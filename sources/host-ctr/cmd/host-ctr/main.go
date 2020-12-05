package main

import (
	"context"
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
	"github.com/urfave/cli/v2"
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
	app := App()
	if err := app.Run(os.Args); err != nil {
		log.L.Fatalf("%v", err)
	}
}

// App sets up a cli.App with `host-ctr`'s flags and subcommands
func App() *cli.App {
	// Command-line arguments
	var (
		containerID      string
		source           string
		containerdSocket string
		namespace        string
		superpowered     bool
	)

	app := cli.NewApp()
	app.Name = "host-ctr"
	app.Usage = "manage host containers"

	// Global options
	app.Flags = []cli.Flag{
		&cli.StringFlag{
			Name:        "containerd-socket",
			Aliases:     []string{"s"},
			Usage:       "path to the containerd socket",
			Value:       "/run/host-containerd/containerd.sock",
			Destination: &containerdSocket,
		},
		&cli.StringFlag{
			Name:        "namespace",
			Aliases:     []string{"n"},
			Usage:       "the containerd namespace to operate in",
			Value:       "default",
			Destination: &namespace,
		},
	}

	// Subcommands
	app.Commands = []*cli.Command{
		{
			Name:  "run",
			Usage: "run host container with the specified image",
			Flags: []cli.Flag{
				&cli.StringFlag{
					Name:        "source",
					Usage:       "the image source",
					Destination: &source,
					Required:    true,
				},
				&cli.StringFlag{
					Name:        "container-id",
					Usage:       "the id of the container to manage",
					Destination: &containerID,
					Required:    true,
				},
				&cli.BoolFlag{
					Name:        "superpowered",
					Usage:       "specifies whether to create the container with `superpowered` privileges",
					Destination: &superpowered,
					Value:       false,
				},
			},
			Action: func(c *cli.Context) error {
				return runCtr(containerdSocket, namespace, containerID, source, superpowered)
			},
		},
		{
			Name:        "pull-image",
			Usage:       "pull the specified container image",
			Description: "pull the specified container image to make it available in the containerd image store",
			Flags: []cli.Flag{
				&cli.StringFlag{
					Name:        "source",
					Usage:       "the image source",
					Destination: &source,
					Required:    true,
				},
			},
			Action: func(c *cli.Context) error {
				return pullImageOnly(containerdSocket, namespace, source)
			},
		},
		{
			Name:  "clean-up",
			Usage: "delete specified container's resources if it exists",
			Flags: []cli.Flag{
				&cli.StringFlag{
					Name:        "container-id",
					Usage:       "the id of the container to clean up",
					Destination: &containerID,
					Required:    true,
				},
			},
			Action: func(c *cli.Context) error {
				return cleanUp(containerdSocket, namespace, containerID)
			},
		},
	}

	return app
}

func runCtr(containerdSocket string, namespace string, containerID string, source string, superpowered bool) error {
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
			log.G(ctx).Info("received signal: ", sigrecv)
			cancel()
		}
	}(ctx, cancel)

	client, err := newContainerdClient(ctx, containerdSocket, namespace)
	if err != nil {
		return err
	}
	defer client.Close()

	// Parse the source ref if it looks like an ECR ref.
	isECRImage := ecrRegex.MatchString(source)
	var img containerd.Image
	if isECRImage {
		img, err = pullECRImage(ctx, source, client)
		if err != nil {
			return err
		}
	} else {
		img, err = pullImage(ctx, source, client)
		if err != nil {
			log.G(ctx).WithField("ref", source).Error(err)
			return err
		}
	}

	// Check if the target container already exists. If it does, take over the helm to manage it.
	container, err := client.LoadContainer(ctx, containerID)
	if err != nil {
		if errdefs.IsNotFound(err) {
			log.G(ctx).WithField("ctr-id", containerID).Info("Container does not exist, proceeding to create it")
		} else {
			log.G(ctx).WithField("ctr-id", containerID).WithError(err).Error("failed to retrieve list of containers")
			return err
		}
	}

	// If the container doesn't already exist, create it
	if container == nil {
		// Get the cgroup path of the systemd service
		cgroupPath, err := cgroups.GetOwnCgroup("name=systemd")
		if err != nil {
			log.G(ctx).WithError(err).Error("failed to discover systemd cgroup path")
			return err
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

		// Create the container.
		container, err = client.NewContainer(
			ctx,
			containerID,
			containerd.WithImage(img),
			containerd.WithNewSnapshot(containerID+"-snapshot", img),
			ctrOpts,
		)
		if err != nil {
			log.G(ctx).WithError(err).WithField("img", img.Name).Error("failed to create container")
			return err
		}
	}
	defer func() {
		// Clean up the container as program wraps up.
		cleanup, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()
		err := container.Delete(cleanup, containerd.WithSnapshotCleanup)
		if err != nil {
			log.G(cleanup).WithError(err).Error("failed to cleanup container")
		}
	}()

	// Check if the container task already exists. If it does, try to manage it.
	task, err := container.Task(ctx, cio.NewAttach(cio.WithStdio))
	if err != nil {
		if errdefs.IsNotFound(err) {
			log.G(ctx).WithField("container-id", containerID).Info("container task does not exist, proceeding to create it")
		} else {
			log.G(ctx).WithField("container-id", containerID).WithError(err).Error("failed to retrieve container task")
			return err
		}
	}
	// If the container doesn't already exist, create it
	taskAlreadyRunning := false
	if task == nil {
		// Create the container task
		task, err = container.NewTask(ctx, cio.NewCreator(cio.WithStdio))
		if err != nil {
			log.G(ctx).WithError(err).Error("failed to create container task")
			return err
		}
	} else {
		// Check the container task process status and see if it's already running.
		taskStatus, err := task.Status(ctx)
		if err != nil {
			log.G(ctx).WithError(err).Error("failed to retrieve container task status")
			return err
		}
		log.G(ctx).WithField("task status", taskStatus.Status).Info("found existing container task")

		// If the task isn't running (it's in some weird state like `Paused`), we should replace it with a new task.
		if taskStatus.Status != containerd.Running {
			_, err := task.Delete(ctx, containerd.WithProcessKill)
			if err != nil {
				log.G(ctx).WithError(err).Error("failed to delete existing container task")
				return err
			}
			log.G(ctx).Info("killed existing container task to replace it with a new task")
			// Recreate the container task
			task, err = container.NewTask(ctx, cio.NewCreator(cio.WithStdio))
			if err != nil {
				log.G(ctx).WithError(err).Error("failed to create container task")
				return err
			}
		} else {
			log.G(ctx).Info("container task is still running, proceeding to monitor it")
			taskAlreadyRunning = true
		}
	}
	defer func() {
		// Clean up the container's task as program wraps up.
		cleanup, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()
		_, err := task.Delete(cleanup)
		if err != nil {
			log.G(cleanup).WithError(err).Error("failed to delete container task")
		}
	}()

	// Call `Wait` to ensure the task's status notifications are received.
	exitStatusC, err := task.Wait(context.TODO())
	if err != nil {
		log.G(ctx).WithError(err).Error("unexpected error during container task setup")
		return err
	}
	if !taskAlreadyRunning {
		// Execute the target container's task.
		if err := task.Start(ctx); err != nil {
			log.G(ctx).WithError(err).Error("failed to start container task")
			return err
		}
		log.G(ctx).Info("successfully started container task")
	}

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
			log.G(ctrCtx).WithError(err).Error("failed to send SIGTERM to container")
			return err
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
				log.G(ctrCtx).WithError(err).Error("failed to SIGKILL container process, timed out")
				return err
			}

			status = <-exitStatusC
		}
	case status = <-exitStatusC:
		// Container task exited on its own
	}
	code, _, err := status.Result()
	if err != nil {
		log.G(ctrCtx).WithError(err).Error("failed to get container task exit status")
		return err
	}
	log.G(ctrCtx).WithField("code", code).Info("container task exited")

	return nil
}

// pullImageOnly pulls the specified container image
func pullImageOnly(containerdSocket string, namespace string, source string) error {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	ctx = namespaces.WithNamespace(ctx, namespace)

	client, err := newContainerdClient(ctx, containerdSocket, namespace)
	if err != nil {
		return err
	}
	defer client.Close()

	// Parse the source ref if it looks like an ECR ref.
	isECRImage := ecrRegex.MatchString(source)
	ref := source
	if isECRImage {
		_, err = pullECRImage(ctx, source, client)
		if err != nil {
			return err
		}
	} else {
		_, err = pullImage(ctx, ref, client)
		if err != nil {
			log.G(ctx).WithField("ref", ref).Error(err)
			return err
		}
	}

	return nil
}

// cleanUp checks if the specified container exists and attempts to clean it up
func cleanUp(containerdSocket string, namespace string, containerID string) error {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	ctx = namespaces.WithNamespace(ctx, namespace)

	client, err := newContainerdClient(ctx, containerdSocket, namespace)
	if err != nil {
		return err
	}
	defer client.Close()

	container, err := client.LoadContainer(ctx, containerID)
	if err != nil {
		if errdefs.IsNotFound(err) {
			log.G(ctx).WithField("container-id", containerID).Info("container does not exist, no clean up necessary")
			return nil
		}
		log.G(ctx).WithField("container-id", containerID).WithError(err).Error("failed to retrieve list of containers")
		return err
	}
	if container != nil {
		log.G(ctx).WithField("container-id", containerID).Info("container exists, deleting")
		// Kill task associated with existing container if it exists
		task, err := container.Task(ctx, nil)
		if err != nil {
			// No associated task found, proceed to delete existing container
			if errdefs.IsNotFound(err) {
				log.G(ctx).WithField("container-id", containerID).Info("no task associated with existing container")
			} else {
				log.G(ctx).WithField("container-id", containerID).WithError(err).Error("failed to retrieve task associated with existing container")
				return err
			}
		}
		if task != nil {
			_, err := task.Delete(ctx, containerd.WithProcessKill)
			if err != nil {
				log.G(ctx).WithField("container-id", containerID).WithError(err).Error("failed to delete existing container task")
				return err
			}
			log.G(ctx).WithField("container-id", containerID).Info("killed existing container task")
		}
		if err := container.Delete(ctx, containerd.WithSnapshotCleanup); err != nil {
			log.G(ctx).WithField("container-id", containerID).WithError(err).Error("failed to delete existing container")
			return err
		}
		log.G(ctx).WithField("container-id", containerID).Info("deleted existing container")
	}

	return nil
}

// pullECRImage does some additional conversions before resolving the image reference and pulls the image.
func pullECRImage(ctx context.Context, source string, client *containerd.Client) (containerd.Image, error) {
	ref := source
	ecrRef, err := ecr.ParseImageURI(ref)
	if err != nil {
		log.G(ctx).WithError(err).WithField("source", source).Error("failed to parse ECR reference")
		return nil, err
	}

	ref = ecrRef.Canonical()
	log.G(ctx).
		WithField("ref", ref).
		WithField("source", source).
		Debug("parsed ECR reference from URI")

	img, err := pullImage(ctx, ref, client)
	if err != nil {
		log.G(ctx).WithField("ref", ref).Error(err)
		return nil, err
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
	log.G(ctx).
		WithField("ref", ref).
		WithField("source", source).
		Debug("adding source tag on pulled image")
	if err := tagImage(ctx, ref, source, client); err != nil {
		log.G(ctx).
			WithError(err).
			WithField("source", source).
			WithField("ref", ref).
			Error("failed to add source tag on pulled image")
		return nil, err
	}

	return img, nil
}

// newContainerdClient creates a new containerd client connected to the specified containerd socket.
func newContainerdClient(ctx context.Context, containerdSocket string, namespace string) (*containerd.Client, error) {
	client, err := containerd.New(containerdSocket, containerd.WithDefaultNamespace(namespace))
	if err != nil {
		log.G(ctx).
			WithError(err).
			WithField("socket", containerdSocket).
			WithField("namespace", namespace).
			Error("failed to connect to containerd")
		return nil, err
	}

	return client, nil
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
				Options:     []string{"rbind", "rw"},
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
			log.G(ctx).WithField("img", img.Name()).Info("pulled image successfully")
			break
		}
		if retryAttempts >= maxRetryAttempts {
			return nil, errors.Wrap(err, "retries exhausted")
		}
		// Add a random jitter between 2 - 6 seconds to the retry interval
		retryIntervalWithJitter := retryInterval + time.Duration(rand.Int31n(jitterPeakAmplitude))*time.Millisecond + jitterLowerBound*time.Millisecond
		log.G(ctx).WithError(err).Warnf("failed to pull image. waiting %s before retrying...", retryIntervalWithJitter)
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

	log.G(ctx).WithField("img", img.Name()).Info("unpacking image...")
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
	log.G(ctx).WithField("img", newImageName).Info("tagging image")
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
		log.G(ctx).WithField("ref", ref).Info("pulling with Amazon ECR Resolver")
		c.Resolver = resolver
		return nil
	}
}
