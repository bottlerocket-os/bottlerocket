package main

import (
	"context"
	"flag"
	"math/rand"
	"os"
	"os/signal"
	"regexp"
	"strings"
	"syscall"
	"time"

	"github.com/aws/aws-sdk-go/aws/arn"
	"github.com/awslabs/amazon-ecr-containerd-resolver/ecr"
	"github.com/containerd/containerd"
	"github.com/containerd/containerd/cio"
	"github.com/containerd/containerd/errdefs"
	"github.com/containerd/containerd/log"
	"github.com/containerd/containerd/namespaces"
	"github.com/containerd/containerd/oci"
	"github.com/opencontainers/runc/libcontainer/cgroups"
	runtimespec "github.com/opencontainers/runtime-spec/specs-go"
	"github.com/pkg/errors"
)

func init() {
	rand.Seed(time.Now().UnixNano())
}

func main() {
	os.Exit(_main())
}

func _main() int {
	// Parse command-line arguments
	var (
		targetCtr        string
		source           string
		containerdSocket string
		namespace        string
		superpowered     bool
		pullImageOnly    bool
	)
	flag.StringVar(&targetCtr, "ctr-id", "", "The ID of the container to be started")
	flag.StringVar(&source, "source", "", "The image to be pulled")
	flag.BoolVar(&superpowered, "superpowered", false, "Specifies whether to launch the container in `superpowered` mode or not")
	flag.BoolVar(&pullImageOnly, "pull-image-only", false, "Only pull and unpack the container image, do not start any container task")
	flag.StringVar(&containerdSocket, "containerd-socket", "/run/host-containerd/containerd.sock", "Specifies the path to the containerd socket. Defaults to `/run/host-containerd/containerd.sock`")
	flag.StringVar(&namespace, "namespace", "default", "Specifies the containerd namespace")
	flag.Parse()

	if source == "" || (targetCtr == "" && !pullImageOnly) {
		flag.Usage()
		return 2
	}

	ctx, cancel := context.WithCancel(context.Background())
	ctx = namespaces.WithNamespace(ctx, namespace)
	defer cancel()

	go func(ctx context.Context, cancel context.CancelFunc) {
		// Set up channel on which to send signal notifications.
		// We must use a buffered channel or risk missing the signal
		// if we're not ready to receive when the signal is sent.
		c := make(chan os.Signal, 1)
		signal.Notify(c, syscall.SIGINT, syscall.SIGTERM)
		for {
			select {
			case s := <-c:
				log.G(ctx).Info("Received signal: ", s)
				cancel()
			}
		}
	}(ctx, cancel)

	// Set up containerd client
	// Use host containers' containerd socket
	client, err := containerd.New(containerdSocket, containerd.WithDefaultNamespace(namespace))
	if err != nil {
		log.G(ctx).WithError(err).WithFields(map[string]interface{}{"containerdSocket": containerdSocket, "namespace": namespace}).Error("Failed to connect to containerd")
		return 1
	}
	defer client.Close()

	// Check if the image is from ECR, if it is, convert the image name into a resolvable reference
	ref := source
	match := ecrRegex.MatchString(source)
	if match {
		var err error
		ref, err = ecrImageNameToRef(source)
		if err != nil {
			log.G(ctx).WithError(err).WithField("source", source)
			return 1
		}
	}

	img, err := pullImage(ctx, ref, client)
	if err != nil {
		log.G(ctx).WithField("ref", ref).Error(err)
		return 1
	}

	// If the image is from ECR, the image reference will be converted into the form of
	// `"ecr.aws/" + the ARN of the image repository + label/digest`.
	// We tag the image with its original image name so other services can discover this image by its original image reference.
	// After the tag operation, this image should be addressable by both its original image reference and its ECR resolver resolvable reference.
	if match {
		// Include original tag on ECR image for other consumers.
		if err := tagImage(ctx, ref, source, client); err != nil {
			log.G(ctx).WithError(err).WithField("source", source).Error("Failed to tag an image with original image name")
			return 1
		}
	}

	// If we're only pulling and unpacking the image, we're done here
	if pullImageOnly {
		return 0
	}

	// Clean up target container if it already exists before starting container task
	if err := deleteCtrIfExists(ctx, client, targetCtr); err != nil {
		return 1
	}

	// Get the cgroup path of the systemd service
	cgroupPath, err := cgroups.GetOwnCgroup("name=systemd")
	if err != nil {
		log.G(ctx).WithError(err).Error("Failed to connect to containerd")
		return 1
	}

	// Set up the container specifications depending on the type of container and whether it's superpowered or not
	ctrOpts := containerd.WithNewSpec(
		oci.WithImageConfig(img),
		oci.WithHostNamespace(runtimespec.NetworkNamespace),
		oci.WithHostHostsFile,
		oci.WithHostResolvconf,
		// Launch the container under the systemd unit's cgroup
		oci.WithCgroup(cgroupPath),
		// Mount in the API socket for the Thar API server, and the API client used to interact with it
		oci.WithMounts([]runtimespec.Mount{
			{
				Options:     []string{"bind", "rw"},
				Destination: "/run/api.sock",
				Source:      "/run/api.sock",
			},
			// Mount in the apiclient to make API calls to the Thar API server
			{
				Options:     []string{"bind", "ro"},
				Destination: "/usr/local/bin/apiclient",
				Source:      "/usr/bin/apiclient",
			},
			// Mount in the persistent storage location for this container
			{
				Options:     []string{"rbind", "rw"},
				Destination: "/.thar/host-containers/" + targetCtr,
				Source:      "/local/host-containers/" + targetCtr,
			}}),
		withSuperpowered(superpowered),
	)

	// Create and start the container via containerd
	container, err := client.NewContainer(
		ctx,
		targetCtr,
		containerd.WithImage(img),
		containerd.WithNewSnapshot(targetCtr+"-snapshot", img),
		ctrOpts,
	)
	if err != nil {
		log.G(ctx).WithError(err).WithField("img", img.Name).Error("Failed to create container")
		return 1
	}
	defer container.Delete(context.TODO(), containerd.WithSnapshotCleanup)

	// Create the container task
	task, err := container.NewTask(ctx, cio.NewCreator(cio.WithStdio))
	if err != nil {
		log.G(ctx).WithError(err).Error("Failed to create container task")
		return 1
	}
	defer task.Delete(context.TODO())

	// Wait before calling start in case the container task finishes too quickly
	exitStatusC, err := task.Wait(context.TODO())
	if err != nil {
		log.G(ctx).WithError(err).Error("Unexpected error during container task setup.")
		return 1
	}

	// Call start on the task to execute the target container
	if err := task.Start(ctx); err != nil {
		log.G(ctx).WithError(err).Error("Failed to start container task")
		return 1
	}
	log.G(ctx).Info("Successfully started container task")

	// Block until an OS signal (e.g. SIGTERM, SIGINT) is received or the container task finishes and exits on its own.
	var status containerd.ExitStatus
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
		force := make(chan struct{})
		timeout := time.AfterFunc(20*time.Second, func() {
			close(force)
		})
		select {
		case status = <-exitStatusC:
			// Container task was able to exit on its own
			timeout.Stop()
		case <-force:
			// Container task still hasn't exited, SIGKILL the container task
			// Create a deadline of 45 seconds
			killCtrTask := func() error {
				const sigkillTimeout = 45 * time.Second
				killCtx, cancel := context.WithTimeout(ctrCtx, sigkillTimeout)
				defer cancel()
				return task.Kill(killCtx, syscall.SIGKILL)
			}
			if killCtrTask() != nil {
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

// Check if container already exists, if it does, kill its task then delete it and clean up its snapshot
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

// Add container options depending on whether it's `superpowered` or not
func withSuperpowered(superpowered bool) oci.SpecOpts {
	if !superpowered {
		return oci.Compose()
	}
	return oci.Compose(
		oci.WithHostNamespace(runtimespec.PIDNamespace),
		oci.WithParentCgroupDevices,
		oci.WithPrivileged,
		oci.WithNewPrivileges,
		oci.WithMounts([]runtimespec.Mount{
			{
				Options:     []string{"rbind", "ro"},
				Destination: "/.thar/rootfs",
				Source:      "/",
			}}),
	)
}

// Expecting to match ECR image names of the form:
// Example 1: 777777777777.dkr.ecr.us-west-2.amazonaws.com/my_image:latest
// Example 2: 777777777777.dkr.ecr.cn-north-1.amazonaws.com.cn/my_image:latest
var ecrRegex = regexp.MustCompile(`(^[a-zA-Z0-9][a-zA-Z0-9-_]*)\.dkr\.ecr\.([a-zA-Z0-9][a-zA-Z0-9-_]*)\.amazonaws\.com(\.cn)?.*`)

// Pulls image from specified source
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
	log.G(ctx).WithField("img", img.Name()).Info("Pulled successfully")
	log.G(ctx).WithField("img", img.Name()).Info("Unpacking...")
	if err := img.Unpack(ctx, containerd.DefaultSnapshotter); err != nil {
		return nil, errors.Wrap(err, "failed to unpack image")
	}
	return img, nil
}

// Image tag logic derived from:
// https://github.com/containerd/containerd/blob/d80513ee8a6995bc7889c93e7858ddbbc51f063d/cmd/ctr/commands/images/tag.go#L67-L86
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

// Return the resolver appropriate for the specified image reference
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

// Transform an ECR image name into a reference resolvable by the Amazon ECR containerd Resolver
// e.g. ecr.aws/arn:<partition>:ecr:<region>:<account>:repository/<name>:<tag>
func ecrImageNameToRef(input string) (string, error) {
	ref := "ecr.aws/"
	partition := "aws"
	if strings.HasPrefix(input, "https://") {
		input = strings.TrimPrefix(input, "https://")
	}
	// Matching on account, region and TLD
	err := errors.New("Invalid ECR image name")
	matches := ecrRegex.FindStringSubmatch(input)
	if len(matches) < 3 {
		return "", err
	}
	tld := matches[3]
	region := matches[2]
	account := matches[1]
	// If `.cn` TLD, partition should be "aws-cn"
	// If US gov cloud regions, partition should be "aws-us-gov"
	// If both of them match, the image source is invalid
	isCnEndpoint := tld == ".cn"
	isGovCloudEndpoint := (region == "us-gov-west-1" || region == "us-gov-east-1")
	if isCnEndpoint && isGovCloudEndpoint {
		return "", err
	} else if isCnEndpoint {
		partition = "aws-cn"
	} else if isGovCloudEndpoint {
		partition = "aws-us-gov"
	}
	// Separate out <name>:<tag> for checking validity
	tokens := strings.Split(input, "/")
	if len(tokens) < 2 {
		return "", errors.New("No specified name and tag or digest")
	}
	fullImageId := tokens[len(tokens)-1]
	matchDigest, _ := regexp.MatchString(`^[a-zA-Z0-9-_]+@sha256:[A-Fa-f0-9]{64}$`, fullImageId)
	matchTag, _ := regexp.MatchString(`^[a-zA-Z0-9-_]+:[a-zA-Z0-9\.\-_]{1,128}$`, fullImageId)
	if !matchDigest && !matchTag {
		return "", errors.New("Malformed name and tag or digest")
	}
	// Need to include the full repository path and the imageID (e.g. /eks/image-name:tag)
	tokens = strings.SplitN(input, "/", 2)
	fullPath := tokens[len(tokens)-1]
	// Build the ARN for the reference
	ecrARN := &arn.ARN{
		Partition: partition,
		Service:   "ecr",
		Region:    region,
		AccountID: account,
		Resource:  "repository/" + fullPath,
	}
	return ref + ecrARN.String(), nil
}
