package main

import (
	"context"
	"encoding/base64"
	"fmt"
	"io"
	"math/rand"
	"os"
	"os/signal"
	"regexp"
	"strings"
	"syscall"
	"time"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/aws/aws-sdk-go/service/ecrpublic"
	"github.com/awslabs/amazon-ecr-containerd-resolver/ecr"
	"github.com/containerd/containerd"
	"github.com/containerd/containerd/cio"
	"github.com/containerd/containerd/containers"
	"github.com/containerd/containerd/contrib/seccomp"
	"github.com/containerd/containerd/errdefs"
	"github.com/containerd/containerd/log"
	"github.com/containerd/containerd/namespaces"
	"github.com/containerd/containerd/oci"
	"github.com/containerd/containerd/remotes/docker"
	"github.com/containerd/containerd/runtime/v2/runc/options"
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
	log.L.Logger.SetOutput(io.Discard)
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
		registryConfig   string
		cType            string
		useCachedImage   bool
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
				&cli.StringFlag{
					Name:        "registry-config",
					Usage:       "path to image registry configuration",
					Destination: &registryConfig,
				},
				&cli.StringFlag{
					Name:        "container-type",
					Usage:       "specifies one of: [host, bootstrap]",
					Destination: &cType,
					Value:       "host",
				},
				&cli.BoolFlag{
					Name:        "use-cached-image",
					Usage:       "skips registry authentication and image pull if the image already exists in the image store",
					Destination: &useCachedImage,
					Value:       false,
				},
			},
			Action: func(c *cli.Context) error {
				return runCtr(containerdSocket, namespace, containerID, source, superpowered, registryConfig, containerType(cType), useCachedImage)
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
				&cli.StringFlag{
					Name:        "registry-config",
					Usage:       "path to image registry configuration",
					Destination: &registryConfig,
				},
				&cli.BoolFlag{
					Name:        "skip-if-image-exists",
					Usage:       "skips registry authentication and image pull if the image already exists in the image store",
					Destination: &useCachedImage,
					Value:       false,
				},
			},
			Action: func(c *cli.Context) error {
				return pullImageOnly(containerdSocket, namespace, source, registryConfig, useCachedImage)
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

// Used to define valid container types
type containerType string

const (
	host      containerType = "host"
	bootstrap containerType = "bootstrap"
)

// IsValid returns true if an invalid type is valid
func (ct containerType) IsValid() bool {
	switch ct {
	case host, bootstrap:
		return true
	}

	return false
}

// PersistentDir returns the persistent base directory for the container type
func (ct containerType) PersistentDir() string {
	switch ct {
	case host:
		return "host-containers"
	case bootstrap:
		return "bootstrap-containers"
	}

	return ""
}

// Prefix returns the prefix for the container type
func (ct containerType) Prefix() string {
	switch ct {
	case bootstrap:
		return "boot."
	case host:
		return ""
	}

	return ""
}

// SliceContains returns true if a slice contains a string
func SliceContains(s []string, v string) bool {
	for _, n := range s {
		if n == v {
			return true
		}
	}
	return false
}

func runCtr(containerdSocket string, namespace string, containerID string, source string, superpowered bool, registryConfigPath string, cType containerType, useCachedImage bool) error {
	// Check if the containerType provided is valid
	if !cType.IsValid() {
		return errors.New("Invalid container type")
	}

	// Return error if caller tries to setup bootstrap container as superpowered
	if cType == bootstrap && superpowered {
		return errors.New("Bootstrap containers can't be superpowered")
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
			log.G(ctx).Info("received signal: ", sigrecv)
			cancel()
		}
	}(ctx, cancel)

	client, err := newContainerdClient(ctx, containerdSocket, namespace)
	if err != nil {
		return err
	}
	defer client.Close()

	// Check if the image source is an ECR image. If it is, then we need to handle it with the ECR resolver.
	isECRImage := ecrRegex.MatchString(source)
	var img containerd.Image
	if isECRImage {
		img, err = fetchECRImage(ctx, source, client, registryConfigPath, useCachedImage)
		if err != nil {
			return err
		}
	} else {
		img, err = fetchImage(ctx, source, client, registryConfigPath, useCachedImage)
		if err != nil {
			log.G(ctx).WithField("ref", source).Error(err)
			return err
		}
	}

	prefix := cType.Prefix()
	containerName := containerID
	containerID = prefix + containerID
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
		// Set the destination name for the container persistent storage location
		persistentDir := cType.PersistentDir()

		specOpts := []oci.SpecOpts{
			oci.WithImageConfig(img),
			oci.WithHostNamespace(runtimespec.NetworkNamespace),
			oci.WithHostHostsFile,
			oci.WithHostResolvconf,
			// Unmask `/sys/firmware` to provide extra insight into the hardware of the
			// underlying host, such as the number of CPU sockets on aarch64 variants
			withUnmaskedPaths([]string{"/sys/firmware"}),
			// Pass proxy environment variables to this container
			withProxyEnv(),
			// Add a default set of mounts regardless of the container type
			withDefaultMounts(containerName, persistentDir),
			// Mount the container's rootfs with an SELinux label that makes it writable
			withMountLabel("system_u:object_r:secret_t:s0"),
		}

		// Select the set of specOpts based on the container type
		switch {
		case superpowered:
			specOpts = append(specOpts, withSuperpowered())
		case cType == bootstrap:
			specOpts = append(specOpts, withBootstrap())
		default:
			specOpts = append(specOpts, withDefault())
		}

		ctrOpts := containerd.WithNewSpec(specOpts...)

		// Create the container.
		container, err = client.NewContainer(
			ctx,
			containerID,
			containerd.WithImage(img),
			containerd.WithNewSnapshot(containerID+"-snapshot", img),
			containerd.WithRuntime("io.containerd.runc.v2", &options.Options{
				Root: "/run/host-containerd/runc",
			}),
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

	// Return error if container exists with non-zero status
	if code != 0 {
		return fmt.Errorf("Container %s exited with non-zero status", containerID)
	}

	return nil
}

// pullImageOnly pulls the specified container image
func pullImageOnly(containerdSocket string, namespace string, source string, registryConfigPath string, useCachedImage bool) error {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	ctx = namespaces.WithNamespace(ctx, namespace)

	client, err := newContainerdClient(ctx, containerdSocket, namespace)
	if err != nil {
		return err
	}
	defer client.Close()

	// Check if the image source is an ECR image. If it is, then we need to handle it with the ECR resolver.
	isECRImage := ecrRegex.MatchString(source)
	if isECRImage {
		_, err = fetchECRImage(ctx, source, client, registryConfigPath, useCachedImage)
		if err != nil {
			return err
		}
	} else {
		_, err = fetchImage(ctx, source, client, registryConfigPath, useCachedImage)
		if err != nil {
			log.G(ctx).WithField("ref", source).Error(err)
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

// parseImageURISpecialRegions mimics the parsing in ecr.ParseImageURI but
// constructs the canonical ECR references while skipping certain checks.
// We only do this for special regions that are not yet supported by the aws-go-sdk.
// Referenced source: https://github.com/awslabs/amazon-ecr-containerd-resolver/blob/a5058cf091f4fc573813a032db37a9820952f1f9/ecr/ref.go#L70-L71
func parseImageURISpecialRegions(input string) (ecr.ECRSpec, error) {
	ecrRefPrefixMapping := map[string]string{
		"ca-west-1": "ecr.aws/arn:aws:ecr:ca-west-1:",
	}
	// Matching on account, region
	matches := ecrRegex.FindStringSubmatch(input)
	if len(matches) < 3 {
		return ecr.ECRSpec{}, fmt.Errorf("invalid image URI: %s", input)
	}
	account := matches[1]
	region := matches[2]

	// Need to include the full repository path and the imageID (e.g. /eks/image-name:tag)
	tokens := strings.SplitN(input, "/", 2)
	if len(tokens) != 2 {
		return ecr.ECRSpec{}, fmt.Errorf("invalid image URI: %s", input)
	}
	fullRepoPath := tokens[len(tokens)-1]
	// Run simple checks on the provided repository.
	switch {
	case
		// Must not be empty
		fullRepoPath == "",
		// Must not have a partial/unsupplied label
		strings.HasSuffix(fullRepoPath, ":"),
		// Must not have a partial/unsupplied digest specifier
		strings.HasSuffix(fullRepoPath, "@"):
		return ecr.ECRSpec{}, errors.New("incomplete reference provided")
	}

	// Get the ECR image reference prefix from the AWS region
	ecrRefPrefix, ok := ecrRefPrefixMapping[region]
	if !ok {
		return ecr.ECRSpec{}, fmt.Errorf("%s: %s", "invalid region in internal mapping", region)
	}

	return ecr.ParseRef(fmt.Sprintf("%s%s:repository/%s", ecrRefPrefix, account, fullRepoPath))
}

// fetchECRRef attempts to resolve the ECR reference from an input source string
// by first using the aws-sdk-go's ParseImageURI function. This will fail for
// special regions that are not yet supported. If it fails for any reason,
// attempt to parse again using parseImageURISpecialRegions in this package.
// This uses a special region reference to build the ECR image references.
// If both fail, an error is returned.
func fetchECRRef(ctx context.Context, input string) (ecr.ECRSpec, error) {
	var spec ecr.ECRSpec
	spec, err := ecr.ParseImageURI(input)
	if err == nil {
		return spec, nil
	}
	log.G(ctx).WithError(err).WithField("source", input).Warn("failed to parse ECR reference")

	// The parsing might fail if the AWS region is special, parse again with special handling:
	spec, err = parseImageURISpecialRegions(input)
	if err == nil {
		return spec, nil
	}

	// Return the error for the parseImageURISpecialRegions from this package
	// if a valid ECR ref has not yet been returned
	log.G(ctx).WithError(err).WithField("source", input).Error("failed to parse special ECR reference")
	return ecr.ECRSpec{}, errors.Wrap(err, "could not parse ECR reference for special regions")

}

// fetchECRImage does some additional conversions before resolving the image reference and fetches the image.
func fetchECRImage(ctx context.Context, source string, client *containerd.Client, registryConfigPath string, fetchCachedImageIfExist bool) (containerd.Image, error) {
	ecrRef, err := fetchECRRef(ctx, source)
	if err != nil {
		return nil, err
	}
	ref := ecrRef.Canonical()

	log.G(ctx).
		WithField("ref", ref).
		WithField("source", source).
		Debug("parsed ECR reference from URI")

	img, err := fetchImage(ctx, ref, client, registryConfigPath, fetchCachedImageIfExist)
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

// withSuperpowered adds container options to grant administrative privileges
func withSuperpowered() oci.SpecOpts {
	return oci.Compose(
		withPrivilegedMounts(),
		withRootFsShared(),
		oci.WithHostNamespace(runtimespec.PIDNamespace),
		oci.WithParentCgroupDevices,
		oci.WithPrivileged,
		oci.WithNewPrivileges,
		oci.WithSelinuxLabel("system_u:system_r:super_t:s0-s0:c0.c1023"),
		oci.WithAllDevicesAllowed,
	)
}

// withBootstrap adds container options to grant read-write access to the underlying
// root filesystem, as well as to manage the devices attached to the host
func withBootstrap() oci.SpecOpts {
	return oci.Compose(
		withPrivilegedMounts(),
		withStorageMounts(),
		withRootFsShared(),
		oci.WithSelinuxLabel("system_u:system_r:control_t:s0-s0:c0.c1023"),
		// Bootstrap containers don't require all capabilities. We only add
		// - CAP_SYS_ADMIN: for mounting filesystems
		// - CAP_NET_ADMIN: for managing iptables rules
		// - CAP_SYS_CHROOT: to execute binaries from the root filesystem
		// - CAP_SYS_MODULE: to load kernel modules from the root filesystem
		// managing iptables rules, `CAP_SYS_CH`
		oci.WithAddedCapabilities([]string{"CAP_SYS_ADMIN", "CAP_NET_ADMIN", "CAP_SYS_CHROOT", "CAP_SYS_MODULE"}),
		// `WithDefaultProfile` creates the proper seccomp profile based on the
		// container's capabilities.
		seccomp.WithDefaultProfile(),
		oci.WithAllDevicesAllowed,
		withSwapManagement,
	)
}

// withDefault adds container options for non-privileged containers
func withDefault() oci.SpecOpts {
	return oci.Compose(
		oci.WithSelinuxLabel("system_u:system_r:control_t:s0-s0:c0.c1023"),
		// Non-privileged containers only have access to a subset of the devices
		oci.WithDefaultUnixDevices,
		// No additional capabilities required for non-privileged containers
		seccomp.WithDefaultProfile(),
	)
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

// withDefaultMounts adds the mount configurations required in all container types,
// all default mounts are set up with rprivate propagations
func withDefaultMounts(containerID string, persistentDir string) oci.SpecOpts {
	var mounts = []runtimespec.Mount{
		// Local persistent storage for the container
		{
			Options:     []string{"rbind", "rw"},
			Destination: fmt.Sprintf("/.bottlerocket/%s/%s", persistentDir, containerID),
			Source:      fmt.Sprintf("/local/%s/%s", persistentDir, containerID),
		},
		// Mount in the API socket for the Bottlerocket API server, and the API
		// client used to interact with it
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
		// Cgroup filesystem for this container
		{
			Destination: "/sys/fs/cgroup",
			Type:        "cgroup",
			Source:      "cgroup",
			Options:     []string{"ro", "nosuid", "noexec", "nodev"},
		},
		// Bottlerocket release information for the container
		{
			Options:     []string{"bind", "ro"},
			Destination: "/etc/bottlerocket-release",
			Source:      "/etc/os-release",
		},
		// Bottlerocket RPM inventory available to the container
		{
			Options:     []string{"bind", "ro"},
			Destination: "/var/lib/bottlerocket/inventory/application.json",
			Source:      "/usr/share/bottlerocket/application-inventory.json",
		},
		// Bottlerocket logs
		{
			Options:     []string{"bind", "ro"},
			Destination: "/.bottlerocket/support",
			Source:      "/var/log/support",
			Type:        "bind",
		},
	}

	// The `current` dir was added for easier referencing in Dockerfiles and scripts.
	// If a host container is also named `current`, only add a single `current` mount
	// to the spec.
	if containerID != "current" {
		mounts = append(mounts, runtimespec.Mount{
			Options:     []string{"rbind", "rw"},
			Destination: fmt.Sprintf("/.bottlerocket/%s/current", persistentDir),
			Source:      fmt.Sprintf("/local/%s/%s", persistentDir, containerID),
		})
	}

	// Use withMounts to make sure all mounts have rprivate propagations
	return withMounts(mounts)
}

// withPrivilegedMounts adds options to grant access to the host root filesystem
func withPrivilegedMounts() oci.SpecOpts {
	// Use withMounts to force rprivate when no propagation configurations
	// are set
	return withMounts([]runtimespec.Mount{
		{
			Options:     []string{"rbind", "ro"},
			Destination: "/.bottlerocket/rootfs",
			Source:      "/",
			Type:        "bind",
		},
		{
			Options:     []string{"rbind", "ro"},
			Destination: "/lib/modules",
			Source:      "/lib/modules",
			Type:        "bind",
		},
		{
			Options:     []string{"rbind", "rw"},
			Destination: "/usr/src/kernels",
			Source:      "/usr/src/kernels",
			Type:        "bind",
		},
		{
			Options:     []string{"rbind"},
			Destination: "/sys/firmware",
			Source:      "/sys/firmware",
			Type:        "bind",
		},
		{
			Options:     []string{"rbind"},
			Destination: "/sys/kernel/debug",
			Source:      "/sys/kernel/debug",
			Type:        "bind",
		},
		// Use shared propagations so mounts in this mount point propagate
		// across the peer group
		{
			Options:     []string{"rbind", "rshared"},
			Destination: "/.bottlerocket/rootfs/mnt",
			Source:      "/mnt",
			Type:        "bind",
		},
	})
}

// withStorageMounts adds options to share container storage mounts
func withStorageMounts() oci.SpecOpts {
	var mounts []runtimespec.Mount

	storageDirs := []string{
		"/var/lib/containerd",
		"/var/lib/docker",
		"/var/lib/kubelet",
	}

	for _, storageDir := range storageDirs {
		if _, err := os.Stat(storageDir); !os.IsNotExist(err) {
			mounts = append(mounts, runtimespec.Mount{
				Options:     []string{"rbind", "rshared"},
				Destination: fmt.Sprintf("/.bottlerocket/rootfs/%s", storageDir),
				Source:      storageDir,
				Type:        "bind",
			})
		}
	}

	// No need to call the `withMounts` helper since any mounts will have
	// propagation settings defined in the options.
	return oci.WithMounts(mounts)
}

// withMounts sets the mounts' propagations as rprivate only when the
// mounts' options don't have propagations settings
func withMounts(mounts []runtimespec.Mount) oci.SpecOpts {
	finalMounts := []runtimespec.Mount{}

	for _, mount := range mounts {
		// Only set rprivate when no propagations are configured for
		// the mount
		if !hasPropagation(mount) {
			// Update the local mount copy instead of the original
			mount.Options = append(mount.Options, "rprivate")
		}
		finalMounts = append(finalMounts, mount)
	}

	return oci.WithMounts(finalMounts)
}

// hasPropagation checks if the mount has propagation options
func hasPropagation(mount runtimespec.Mount) bool {
	// Propagations can be shared, rshared, private, rprivate, slave, rslave
	for _, option := range mount.Options {
		switch option {
		case "shared", "rshared", "private", "rprivate", "slave", "rslave":
			return true
		}
	}

	return false
}

// withRootFsShared sets the rootfs mount propagation as `rshared`
func withRootFsShared() oci.SpecOpts {
	return func(_ context.Context, _ oci.Client, _ *containers.Container, s *runtimespec.Spec) error {
		if s.Linux != nil {
			s.Linux.RootfsPropagation = "rshared"
		}
		return nil
	}
}

// withSwapManagement allows the swapon and swapoff syscalls
func withSwapManagement(_ context.Context, _ oci.Client, _ *containers.Container, s *runtimespec.Spec) error {
	if s.Linux != nil && s.Linux.Seccomp != nil && s.Linux.Seccomp.Syscalls != nil {
		s.Linux.Seccomp.Syscalls = append(s.Linux.Seccomp.Syscalls, runtimespec.LinuxSyscall{
			Names:  []string{"swapon", "swapoff"},
			Action: runtimespec.ActAllow,
			Args:   []runtimespec.LinuxSeccompArg{},
		})
	}
	return nil
}

// withProxyEnv reads proxy environment variables and returns a spec option for passing said proxy environment variables
func withProxyEnv() oci.SpecOpts {
	noOp := func(_ context.Context, _ oci.Client, _ *containers.Container, s *runtimespec.Spec) error { return nil }
	httpsProxy, httpsProxySet := os.LookupEnv("HTTPS_PROXY")
	noProxy, noProxySet := os.LookupEnv("NO_PROXY")
	withHTTPSProxy := noOp
	withNoProxy := noOp
	if httpsProxySet {
		withHTTPSProxy = oci.WithEnv([]string{"HTTPS_PROXY=" + httpsProxy, "https_proxy=" + httpsProxy})
	}
	if noProxySet {
		withNoProxy = oci.WithEnv([]string{"NO_PROXY=" + noProxy, "no_proxy=" + noProxy})
	}
	return oci.Compose(withHTTPSProxy, withNoProxy)
}

// fetchImage returns a `containerd.Image` given an image source.
func fetchImage(ctx context.Context, source string, client *containerd.Client, registryConfigPath string, useCachedImage bool) (containerd.Image, error) {
	// Check the containerd image store to see if image exists
	img, err := client.GetImage(ctx, source)
	if err != nil {
		if errdefs.IsNotFound(err) {
			log.G(ctx).WithField("ref", source).Info("Image does not exist, proceeding to pull image from source.")
		} else {
			log.G(ctx).WithField("ref", source).Error(err)
			return nil, err
		}
	}
	if img != nil && useCachedImage {
		log.G(ctx).WithField("ref", source).Info("Image exists, fetching cached image from image store")
		return img, err
	}
	return pullImage(ctx, source, client, registryConfigPath)
}

// pullImage pulls an image from the specified source.
func pullImage(ctx context.Context, source string, client *containerd.Client, registryConfigPath string) (containerd.Image, error) {
	// Handle registry config
	var registryConfig *RegistryConfig
	if registryConfigPath != "" {
		var err error
		registryConfig, err = NewRegistryConfig(registryConfigPath)
		if err != nil {
			log.G(ctx).
				WithError(err).
				WithField("registry-config", registryConfigPath).
				Error("failed to read registry config")
			return nil, err
		}
	}

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
			withDynamicResolver(ctx, source, registryConfig),
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
func withDynamicResolver(ctx context.Context, ref string, registryConfig *RegistryConfig) containerd.RemoteOpt {
	defaultResolver := func(_ *containerd.Client, _ *containerd.RemoteContext) error { return nil }
	if registryConfig != nil {
		defaultResolver = func(_ *containerd.Client, c *containerd.RemoteContext) error {
			resolver := docker.NewResolver(docker.ResolverOptions{
				Hosts: registryHosts(registryConfig, nil),
			})
			c.Resolver = resolver
			return nil
		}
	}

	switch {
	// For private ECR registries, we need to use the Amazon ECR resolver.
	// Currently we're unable to support image registry configuration with the ECR resolver.
	// FIXME Track upstream `amazon-ecr-containerd-resolver` support for image registry configuration.
	case strings.HasPrefix(ref, "ecr.aws/"):
		return func(_ *containerd.Client, c *containerd.RemoteContext) error {
			// Create the Amazon ECR resolver
			resolver, err := ecr.NewResolver()
			if err != nil {
				return errors.Wrap(err, "Failed to create ECR resolver")
			}
			log.G(ctx).WithField("ref", ref).Info("pulling with Amazon ECR Resolver")
			c.Resolver = resolver
			return nil
		}
	// For Amazon ECR Public registries, we should try and fetch credentials before resolving the image reference
	case strings.HasPrefix(ref, "public.ecr.aws/"):
		// ... not if the user has specified their own registry credentials for 'public.ecr.aws'; In that case we use the default resolver.
		if _, found := registryConfig.Credentials["public.ecr.aws"]; found {
			return defaultResolver
		}

		// Try to get credentials for authenticated pulls from ECR Public
		session := session.Must(session.NewSession())
		// The ECR Public API is only available in us-east-1 today
		publicConfig := aws.NewConfig().WithRegion("us-east-1")
		client := ecrpublic.New(session, publicConfig)
		output, err := client.GetAuthorizationToken(&ecrpublic.GetAuthorizationTokenInput{})
		if err != nil {
			log.G(ctx).Warn("ecr-public: failed to get authorization token, falling back to default resolver (unauthenticated pull)")
			return defaultResolver
		}
		if output == nil || output.AuthorizationData == nil {
			log.G(ctx).Warn("ecr-public: missing AuthorizationData in ECR Public GetAuthorizationToken response, falling back to default resolver (unauthenticated pull)")
			return defaultResolver
		}
		authToken, err := base64.StdEncoding.DecodeString(aws.StringValue(output.AuthorizationData.AuthorizationToken))
		if err != nil {
			log.G(ctx).Warn("ecr-public: unable to decode authorization token, falling back to default resolver (unauthenticated pull)")
			return defaultResolver
		}
		tokens := strings.SplitN(string(authToken), ":", 2)
		if len(tokens) != 2 {
			log.G(ctx).Warn("ecr-public: invalid credentials decoded from authorization token, falling back to default resolver (unauthenticated pull)")
			return defaultResolver
		}
		// Use the fetched authorization credentials to resolve the image
		authOpt := docker.WithAuthCreds(func(host string) (string, string, error) {
			// Double-check to make sure the we're doing this for an ECR Public registry
			if host != "public.ecr.aws" {
				return "", "", errors.New("ecr-public: expected image to start with public.ecr.aws")
			}
			return tokens[0], tokens[1], nil
		})
		authorizer := docker.NewDockerAuthorizer(authOpt)
		resolverOpt := docker.ResolverOptions{
			Hosts: registryHosts(registryConfig, &authorizer),
		}

		return func(_ *containerd.Client, c *containerd.RemoteContext) error {
			resolver := docker.NewResolver(resolverOpt)
			log.G(ctx).WithField("ref", ref).Info("pulling from ECR Public")
			c.Resolver = resolver
			return nil
		}
	default:
		// For all other registries
		return defaultResolver
	}
}

// withUnmaskedPaths sets an alternate list of masked paths, less the paths provided
func withUnmaskedPaths(unmaskPaths []string) oci.SpecOpts {
	return func(_ context.Context, _ oci.Client, _ *containers.Container, s *runtimespec.Spec) error {
		if s.Linux != nil && s.Linux.MaskedPaths != nil {
			var maskedPaths []string
			for _, path := range s.Linux.MaskedPaths {
				if SliceContains(unmaskPaths, path) {
					continue
				}
				maskedPaths = append(maskedPaths, path)
			}
			// Replace the default Linux.MaskedPaths with our own
			s.Linux.MaskedPaths = maskedPaths
		}
		return nil
	}
}
