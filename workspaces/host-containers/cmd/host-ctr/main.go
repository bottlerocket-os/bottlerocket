package main

import (
	"context"
	"flag"
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
	"github.com/containerd/containerd/log"
	"github.com/containerd/containerd/namespaces"
	"github.com/containerd/containerd/oci"
	cgroups "github.com/opencontainers/runc/libcontainer/cgroups"
	runtimespec "github.com/opencontainers/runtime-spec/specs-go"
	"github.com/pkg/errors"
)

func main() {
	os.Exit(_main())
}

func _main() int {
	// Parse command-line arguments
	targetCtr, source := "", ""
	superpowered := false

	flag.StringVar(&targetCtr, "ctr-id", "", "The ID of the container to be started")
	flag.StringVar(&source, "source", "", "The image to be pulled")
	flag.BoolVar(&superpowered, "superpowered", false, "Specifies whether to launch the container in `superpowered` mode or not")
	flag.Parse()

	if targetCtr == "" || source == "" {
		flag.Usage()
		return 2
	}

	ctx := namespaces.NamespaceFromEnv(context.Background())

	// Set up channel on which to send signal notifications.
	// We must use a buffered channel or risk missing the signal
	// if we're not ready to receive when the signal is sent.
	c := make(chan os.Signal, 1)
	signal.Notify(c, syscall.SIGINT, syscall.SIGTERM)

	// Set up containerd client
	// Use host containers' containerd socket
	client, err := containerd.New("/run/host-containerd/containerd.sock")
	if err != nil {
		log.G(ctx).WithError(err).Error("Failed to connect to containerd")
		return 1
	}
	defer client.Close()

	img, err := pullImage(ctx, source, client)
	if err != nil {
		log.G(ctx).WithField("source", source).Error(err)
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
	defer container.Delete(ctx, containerd.WithSnapshotCleanup)

	// Create the container task
	task, err := container.NewTask(ctx, cio.NewCreator(cio.WithStdio))
	if err != nil {
		log.G(ctx).WithError(err).Error("Failed to create container task")
		return 1
	}
	defer task.Delete(ctx)

	// Wait before calling start in case the container task finishes too quickly
	exitStatusC, err := task.Wait(ctx)
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
	select {
	case s := <-c:
		log.G(ctx).Info("Received signal: ", s)
		// SIGTERM the container task and get its exit status
		if err := task.Kill(ctx, syscall.SIGTERM); err != nil {
			log.G(ctx).WithError(err).Error("Failed to send SIGTERM to container")
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
				killCtx, cancel := context.WithTimeout(ctx, sigkillTimeout)
				defer cancel()
				return task.Kill(killCtx, syscall.SIGKILL)
			}
			if killCtrTask() != nil {
				log.G(ctx).WithError(err).Error("Failed to SIGKILL container process, timed out")
				return 1
			}

			status = <-exitStatusC
		}
	case status = <-exitStatusC:
		// Container task exited on its own
	}
	code, _, err := status.Result()
	if err != nil {
		log.G(ctx).WithError(err).Error("Failed to get container task exit status")
		return 1
	}
	log.G(ctx).WithField("code", code).Info("Container task exited")
	return int(code)
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
	if match := ecrRegex.MatchString(source); match {
		var err error
		source, err = ecrImageNameToRef(source)
		if err != nil {
			return nil, err
		}
	}

	// Pull the image from ECR
	img, err := client.Pull(ctx, source,
		withDynamicResolver(ctx, source),
		containerd.WithSchema1Conversion)
	if err != nil {
		return nil, errors.Wrap(err, "Failed to pull ctr image")
	}
	log.G(ctx).WithField("img", img.Name()).Info("Pulled successfully")
	log.G(ctx).WithField("img", img.Name()).Info("Unpacking...")
	if err := img.Unpack(ctx, containerd.DefaultSnapshotter); err != nil {
		return nil, errors.Wrap(err, "Failed to unpack image")
	}
	return img, nil
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
		log.G(ctx).WithField("ref", ref).Info("Pulling from Amazon ECR")
		c.Resolver = resolver
		return nil
	}
}

// Transform an ECR image name into a reference resolvable by the Amazon ECR Containerd Resolver
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
	// Separate out <name>:<tag>
	tokens := strings.Split(input, "/")
	if len(tokens) != 2 {
		return "", errors.New("No specified name and tag or digest")
	}
	fullImageId := tokens[1]
	matchDigest, _ := regexp.MatchString(`^[a-zA-Z0-9-_]+@sha256:[A-Fa-f0-9]{64}$`, fullImageId)
	matchTag, _ := regexp.MatchString(`^[a-zA-Z0-9-_]+:[a-zA-Z0-9.-_]{1,128}$`, fullImageId)
	if !matchDigest && !matchTag {
		return "", errors.New("Malformed name and tag or digest")
	}
	// Build the ARN for the reference
	ecrARN := &arn.ARN{
		Partition: partition,
		Service:   "ecr",
		Region:    region,
		AccountID: account,
		Resource:  "repository/" + fullImageId,
	}
	return ref + ecrARN.String(), nil
}
