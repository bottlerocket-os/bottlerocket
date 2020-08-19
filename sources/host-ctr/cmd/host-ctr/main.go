package main

import (
	"context"
	"flag"
	"io/ioutil"
	"math/rand"
	"os"
	"os/signal"
	"regexp"
	"strings"
	"syscall"
	"time"

	"github.com/awslabs/amazon-ecr-containerd-resolver/ecr"
	"github.com/containerd/containerd"
	"github.com/containerd/containerd/cio"
	"github.com/containerd/containerd/containers"
	"github.com/containerd/containerd/contrib/seccomp"
	"github.com/containerd/containerd/errdefs"
	"github.com/containerd/containerd/log"
	"github.com/containerd/containerd/namespaces"
	"github.com/containerd/containerd/oci"
	"github.com/opencontainers/runc/libcontainer/cgroups"
	runtimespec "github.com/opencontainers/runtime-spec/specs-go"
	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
)

// Expecting to match ECR image names of the form:
//
// Example 1: 777777777777.dkr.ecr.us-west-2.amazonaws.com/my_image:latest
// Example 2: 777777777777.dkr.ecr.cn-north-1.amazonaws.com.cn/my_image:latest
var ecrRegex = regexp.MustCompile(`(^[a-zA-Z0-9][a-zA-Z0-9-_]*)\.dkr\.ecr\.([a-zA-Z0-9][a-zA-Z0-9-_]*)\.amazonaws\.com(\.cn)?.*`)

func init() {
	rand.Seed(time.Now().UnixNano())
	// Dispatch logging output instead of writing all levels' messages to
	// stderr.
	log.L.Logger.SetOutput(ioutil.Discard)
	log.L.Logger.AddHook(&LogSplitHook{os.Stdout, []logrus.Level{
		logrus.WarnLevel, logrus.InfoLevel, logrus.DebugLevel, logrus.TraceLevel}})
	log.L.Logger.AddHook(&LogSplitHook{os.Stderr, []logrus.Level{
		logrus.PanicLevel, logrus.FatalLevel, logrus.ErrorLevel}})
}

func main() {
	os.Exit(_main())
}

func _main() int {
	// Parse command-line arguments
	var (
		containerID      string
		source           string
		containerdSocket string
		namespace        string
		superpowered     bool
		pullImageOnly    bool
	)

	flag.StringVar(&containerID, "ctr-id", "", "The ID of the container to be started")
	flag.StringVar(&source, "source", "", "The image to be pulled")
	flag.BoolVar(&superpowered, "superpowered", false, "Specifies whether to launch the container in `superpowered` mode or not")
	flag.BoolVar(&pullImageOnly, "pull-image-only", false, "Only pull and unpack the container image, do not start any container task")
	flag.StringVar(&containerdSocket, "containerd-socket", "/run/host-containerd/containerd.sock", "Specifies the path to the containerd socket. Defaults to `/run/host-containerd/containerd.sock`")
	flag.StringVar(&namespace, "namespace", "default", "Specifies the containerd namespace")

	flag.Parse()

	// Image source must always be provided.
	if source == "" {
		log.L.Error("source image must be provided")
		flag.Usage()
		return 2
	}

	// Container ID must be provided unless the goal is to pull an image.
	if containerID == "" && !pullImageOnly {
		log.L.Error("container ID must be provided")
		flag.Usage()
		return 2
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	ctx = namespaces.WithNamespace(ctx, namespace)
	go func(ctx context.Context, cancel context.CancelFunc) {
		// Set up channel on which to send signal notifications.
		// We must use a buffered channel or risk missing the signal
		// if we're not ready to receive when the signal is sent.
		c := make(chan os.Signal, 1)
		signal.Notify(c, syscall.SIGINT, syscall.SIGTERM)
		for sigrecv := range c {
			log.G(ctx).Info("Received signal: ", sigrecv)
			cancel()
		}
	}(ctx, cancel)

	// Setup containerd client using provided socket.
	client, err := containerd.New(containerdSocket, containerd.WithDefaultNamespace(namespace))
	if err != nil {
		log.G(ctx).
			WithError(err).
			WithField("socket", containerdSocket).
			WithField("namespace", namespace).
			Error("Failed to connect to containerd")
		return 1
	}
	defer client.Close()

	// Parse the source ref if it looks like an ECR ref.
	ref := source
	isECRImage := ecrRegex.MatchString(ref)
	if isECRImage {
		ecrRef, err := ecr.ParseImageURI(ref)
		if err != nil {
			log.G(ctx).WithError(err).WithField("source", source).Error("Failed to parse ECR reference")
			return 1
		}
		ref = ecrRef.Canonical()
		log.G(ctx).
			WithField("source", source).
			WithField("ref", ref).
			Debug("Parsed ECR reference from URI")
	}

	img, err := pullImage(ctx, ref, client)
	if err != nil {
		log.G(ctx).WithField("ref", ref).Error(err)
		return 1
	}

	// When the image is from ECR, the image reference will be converted from
	// its ref format. This is of the form of `"ecr.aws/" + ECR repository ARN +
	// label/digest`.
	//
	// See the resolver for details on this format -
	// https://github.com/awslabs/amazon-ecr-containerd-resolver.
	//
	// If the image was pulled from ECR, add `source` ref pointing to the same
	// image so other clients can locate it using both `source` and the parsed
	// ECR ref.
	if isECRImage {
		// Add additional `source` tag on ECR image for other clients.
		log.G(ctx).
			WithField("ref", ref).
			WithField("source", source).
			Debug("Adding source tag on pulled image")
		if err := tagImage(ctx, ref, source, client); err != nil {
			log.G(ctx).
				WithError(err).
				WithField("source", source).
				WithField("ref", ref).
				Error("Failed to add source tag on pulled image")
			return 1
		}
	}

	// If we're only pulling and unpacking the image, we're done here.
	if pullImageOnly {
		log.G(ctx).Info("Not starting host container, pull-image-only mode specified")
		return 0
	}

	// Clean up target container if it already exists before starting container
	// task.
	if err := deleteCtrIfExists(ctx, client, containerID); err != nil {
		return 1
	}

	// Get the cgroup path of the systemd service
	cgroupPath, err := cgroups.GetOwnCgroup("name=systemd")
	if err != nil {
		log.G(ctx).WithError(err).Error("Failed to discover systemd cgroup path")
		return 1
	}

	// Set up the container spec. See `withSuperpowered` for conditional options
	// set when configured as superpowered.
	ctrOpts := containerd.WithNewSpec(
		oci.WithImageConfig(img),
		oci.WithHostNamespace(runtimespec.NetworkNamespace),
		oci.WithHostHostsFile,
		oci.WithHostResolvconf,
		// Launch the container under the systemd unit's cgroup
		oci.WithCgroup(cgroupPath),
		// Mount in the API socket for the Bottlerocket API server, and the API
		// client used to interact with it
		oci.WithMounts([]runtimespec.Mount{
			{
				Options:     []string{"bind", "rw"},
				Destination: "/run/api.sock",
				Source:      "/run/api.sock",
			},
			// Mount in the apiclient to make API calls to the Bottlerocket API server
			{
				Options:     []string{"bind", "ro"},
				Destination: "/usr/local/bin/apiclient",
				Source:      "/usr/bin/apiclient",
			},
			// Mount in the persistent storage location for this container
			{
				Options:     []string{"rbind", "rw"},
				Destination: "/.bottlerocket/host-containers/" + containerID,
				Source:      "/local/host-containers/" + containerID,
			}}),
		// Mount the rootfs with an SELinux label that makes it writable
		withMountLabel("system_u:object_r:state_t:s0"),
		// Include conditional options for superpowered containers.
		withSuperpowered(superpowered),
	)

	// Create and start the container.
	container, err := client.NewContainer(
		ctx,
		containerID,
		containerd.WithImage(img),
		containerd.WithNewSnapshot(containerID+"-snapshot", img),
		ctrOpts,
	)
	if err != nil {
		log.G(ctx).WithError(err).WithField("img", img.Name).Error("Failed to create container")
		return 1
	}
	defer func() {
		// Clean up the container as program wraps up.
		cleanup, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()
		err := container.Delete(cleanup, containerd.WithSnapshotCleanup)
		if err != nil {
			log.G(cleanup).WithError(err).Error("Failed to cleanup container")
		}
	}()

	// Create the container task
	task, err := container.NewTask(ctx, cio.NewCreator(cio.WithStdio))
	if err != nil {
		log.G(ctx).WithError(err).Error("Failed to create container task")
		return 1
	}
	defer func() {
		// Clean up the container's task as program wraps up.
		cleanup, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()
		_, err := task.Delete(cleanup)
		if err != nil {
			log.G(cleanup).WithError(err).Error("Failed to delete container task")
		}
	}()

	// Call Wait before calling Start to ensure the task's status notifications
	// are received.
	exitStatusC, err := task.Wait(context.TODO())
	if err != nil {
		log.G(ctx).WithError(err).Error("Unexpected error during container task setup.")
		return 1
	}

	// Execute the target container's task.
	if err := task.Start(ctx); err != nil {
		log.G(ctx).WithError(err).Error("Failed to start container task")
		return 1
	}
	log.G(ctx).Info("Successfully started container task")

	// Block until an OS signal (e.g. SIGTERM, SIGINT) is received or the
	// container task finishes and exits on its own.

	// Container task's exit status.
	var status containerd.ExitStatus
	// Context used when stopping and cleaning up the container task
	ctrCtx, cancel := context.WithCancel(context.Background())
	defer cancel()

	select {
	case <-ctx.Done():
		// SIGTERM the container task and get its exit status
		if err := task.Kill(ctrCtx, syscall.SIGTERM); err != nil {
			log.G(ctrCtx).WithError(err).Error("Failed to send SIGTERM to container")
			return 1
		}
		// Wait for 20 seconds and see check if container task exited
		const gracePeriod = 20 * time.Second
		timeout := time.NewTimer(gracePeriod)

		select {
		case status = <-exitStatusC:
			// Container task was able to exit on its own, stop the timer.
			if !timeout.Stop() {
				<-timeout.C
			}
		case <-timeout.C:
			// Container task still hasn't exited, SIGKILL the container task or
			// timeout and bail.

			const sigkillTimeout = 45 * time.Second
			killCtx, cancel := context.WithTimeout(ctrCtx, sigkillTimeout)

			err := task.Kill(killCtx, syscall.SIGKILL)
			cancel()
			if err != nil {
				log.G(ctrCtx).WithError(err).Error("Failed to SIGKILL container process, timed out")
				return 1
			}

			status = <-exitStatusC
		}
	case status = <-exitStatusC:
		// Container task exited on its own
	}
	code, _, err := status.Result()
	if err != nil {
		log.G(ctrCtx).WithError(err).Error("Failed to get container task exit status")
		return 1
	}
	log.G(ctrCtx).WithField("code", code).Info("Container task exited")
	return int(code)
}

// deleteCtrIfExists cleans up an existing container. This involves killing its
// task then deleting it and its snapshot when any exist.
func deleteCtrIfExists(ctx context.Context, client *containerd.Client, targetCtr string) error {
	existingCtr, err := client.LoadContainer(ctx, targetCtr)
	if err != nil {
		if errdefs.IsNotFound(err) {
			log.G(ctx).WithField("ctr-id", targetCtr).Info("No clean up necessary, proceeding")
			return nil
		}
		log.G(ctx).WithField("ctr-id", targetCtr).WithError(err).Error("Failed to retrieve list of containers")
		return err
	}
	if existingCtr != nil {
		log.G(ctx).WithField("ctr-id", targetCtr).Info("Container already exists, deleting")
		// Kill task associated with existing container if it exists
		existingTask, err := existingCtr.Task(ctx, nil)
		if err != nil {
			// No associated task found, proceed to delete existing container
			if errdefs.IsNotFound(err) {
				log.G(ctx).WithField("ctr-id", targetCtr).Info("No task associated with existing container")
			} else {
				log.G(ctx).WithField("ctr-id", targetCtr).WithError(err).Error("Failed to retrieve task associated with existing container")
				return err
			}
		}
		if existingTask != nil {
			_, err := existingTask.Delete(ctx, containerd.WithProcessKill)
			if err != nil {
				log.G(ctx).WithField("ctr-id", targetCtr).WithError(err).Error("Failed to delete existing container task")
				return err
			}
			log.G(ctx).WithField("ctr-id", targetCtr).Info("Killed existing container task")
		}
		if err := existingCtr.Delete(ctx, containerd.WithSnapshotCleanup); err != nil {
			log.G(ctx).WithField("ctr-id", targetCtr).WithError(err).Error("Failed to delete existing container")
			return err
		}
		log.G(ctx).WithField("ctr-id", targetCtr).Info("Deleted existing container")
	}
	return nil
}

// withMountLabel configures the mount with the provided SELinux label.
func withMountLabel(label string) oci.SpecOpts {
	return func(_ context.Context, _ oci.Client, _ *containers.Container, s *runtimespec.Spec) error {
		if s.Linux != nil {
			s.Linux.MountLabel = label
		}
		return nil
	}
}

// withSuperpowered add container options granting administrative privileges
// when it's `superpowered`.
func withSuperpowered(superpowered bool) oci.SpecOpts {
	if !superpowered {
		// Set the `control_t` process label so the host container can
		// interact with the API and modify its local state files.
		return oci.Compose(
			seccomp.WithDefaultProfile(),
			oci.WithSelinuxLabel("system_u:system_r:control_t:s0"),
		)
	}

	return oci.Compose(
		oci.WithHostNamespace(runtimespec.PIDNamespace),
		oci.WithParentCgroupDevices,
		oci.WithPrivileged,
		oci.WithNewPrivileges,
		oci.WithSelinuxLabel("system_u:system_r:super_t:s0"),
		withAllDevicesAllowed,
		oci.WithMounts([]runtimespec.Mount{
			{
				Options:     []string{"rbind", "ro"},
				Destination: "/.bottlerocket/rootfs",
				Source:      "/",
			},
			{
				Options:     []string{"rbind", "ro"},
				Destination: "/lib/modules",
				Source:      "/lib/modules",
			},
			{
				Options:     []string{"rbind", "ro"},
				Destination: "/usr/src/kernels",
				Source:      "/usr/src/kernels",
			},
			{
				Options:     []string{"rbind"},
				Destination: "/sys/kernel/debug",
				Source:      "/sys/kernel/debug",
			}}),
	)
}

// pullImage pulls an image from the specified source.
func pullImage(ctx context.Context, source string, client *containerd.Client) (containerd.Image, error) {
	// Pull the image
	// Retry with exponential backoff when failures occur, maximum retry duration will not exceed 31 seconds
	const maxRetryAttempts = 5
	const intervalMultiplier = 2
	const maxRetryInterval = 30 * time.Second
	const jitterPeakAmplitude = 4000
	const jitterLowerBound = 2000
	var retryInterval = 1 * time.Second
	var retryAttempts = 0
	var img containerd.Image
	for {
		var err error
		img, err = client.Pull(ctx, source,
			withDynamicResolver(ctx, source),
			containerd.WithSchema1Conversion)
		if err == nil {
			log.G(ctx).WithField("img", img.Name()).Info("Pulled successfully")
			break
		}
		if retryAttempts >= maxRetryAttempts {
			return nil, errors.Wrap(err, "retries exhausted")
		}
		// Add a random jitter between 2 - 6 seconds to the retry interval
		retryIntervalWithJitter := retryInterval + time.Duration(rand.Int31n(jitterPeakAmplitude))*time.Millisecond + jitterLowerBound*time.Millisecond
		log.G(ctx).WithError(err).Warnf("Failed to pull image. Waiting %s before retrying...", retryIntervalWithJitter)
		timer := time.NewTimer(retryIntervalWithJitter)
		select {
		case <-timer.C:
			retryInterval *= intervalMultiplier
			if retryInterval > maxRetryInterval {
				retryInterval = maxRetryInterval
			}
			retryAttempts++
		case <-ctx.Done():
			return nil, errors.Wrap(err, "context ended while retrying")
		}
	}

	log.G(ctx).WithField("img", img.Name()).Info("Unpacking...")
	if err := img.Unpack(ctx, containerd.DefaultSnapshotter); err != nil {
		return nil, errors.Wrap(err, "failed to unpack image")
	}

	return img, nil
}

// tagImage adds a tag to the image in containerd's metadata storage.
//
// Image tag logic derived from:
//
// https://github.com/containerd/containerd/blob/d80513ee8a6995bc7889c93e7858ddbbc51f063d/cmd/ctr/commands/images/tag.go#L67-L86
//
func tagImage(ctx context.Context, imageName string, newImageName string, client *containerd.Client) error {
	log.G(ctx).WithField("imageName", newImageName).Info("Tagging image")
	// Retrieve image information
	imageService := client.ImageService()
	image, err := imageService.Get(ctx, imageName)
	if err != nil {
		return err
	}
	// Tag with new image name
	image.Name = newImageName
	// Attempt to create the image first
	if _, err = imageService.Create(ctx, image); err != nil {
		// The image already exists then delete the original and attempt to create the new one
		if errdefs.IsAlreadyExists(err) {
			if err = imageService.Delete(ctx, newImageName); err != nil {
				return err
			}
			if _, err = imageService.Create(ctx, image); err != nil {
				return err
			}
		} else {
			return err
		}
	}
	return nil
}

// withDynamicResolver provides an initialized resolver for use with ref.
func withDynamicResolver(ctx context.Context, ref string) containerd.RemoteOpt {
	if !strings.HasPrefix(ref, "ecr.aws/") {
		// not handled here
		return func(_ *containerd.Client, _ *containerd.RemoteContext) error { return nil }
	}

	return func(_ *containerd.Client, c *containerd.RemoteContext) error {
		// Create the ECR resolver
		resolver, err := ecr.NewResolver()
		if err != nil {
			return errors.Wrap(err, "Failed to create ECR resolver")
		}
		log.G(ctx).WithField("ref", ref).Info("Pulling with Amazon ECR Resolver")
		c.Resolver = resolver
		return nil
	}
}
