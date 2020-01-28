# Makefile - utility for dogswatch builds and development
#
# ecr_uri=$(aws ecr describe-repositories --repository dogswatch --query 'repositories[].repositoryUri' --output text)
#
# make container IMAGE_REPO_NAME="$ecr_uri"
#
# IMAGE_NAME is the full name of the container image being built. This may be
# specified to fully control the name of the container image's tag.
IMAGE_NAME = $(IMAGE_REPO_NAME)$(IMAGE_ARCH_SUFFIX):$(IMAGE_VERSION)$(addprefix -,$(SHORT_SHA))
# IMAGE_REPO_NAME is the image's full name in a container image registry. This
# could be an ECR Repository name or a Docker Hub name such as
# `example-org/example-image`. If the repository includes the architecture name,
# IMAGE_ARCH_SUFFIX must be overridden as needed.
IMAGE_REPO_NAME = bottlerocket/$(notdir $(shell pwd -P))
# IMAGE_VERSION is the semver version that's tagged on the image.
IMAGE_VERSION = $(shell cat VERSION)
# SHORT_SHA is the revision that the container image was built with.
SHORT_SHA = $(shell git describe --abbrev=8 --always --dirty='-dev' --exclude '*' 2>/dev/null || echo "unknown")
# IMAGE_ARCH_SUFFIX is the runtime architecture designator for the container
# image, it is appended to the IMAGE_NAME unless the name is specified.
IMAGE_ARCH_SUFFIX = $(addprefix -,$(ARCH))
# DESTDIR is where the release artifacts will be written.
DESTDIR = .
# DISTFILE is the path to the dist target's output file - the container image
# tarball.
DISTFILE = $(DESTDIR:/=)/$(subst /,_,$(IMAGE_NAME)).tar.gz

# These values derive ARCH and DOCKER_ARCH which are needed by dependencies in
# image build defaulting to system's architecture when not specified.
#
# UNAME_ARCH is the runtime architecture of the building host.
UNAME_ARCH = $(shell uname -m)
# ARCH is the target architecture which is being built for.
ARCH = $(lastword $(subst :, ,$(filter $(UNAME_ARCH):%,x86_64:amd64 aarch64:arm64)))
# DOCKER_ARCH is the docker specific architecture specifier used for building on
# multiarch container images.
DOCKER_ARCH = $(lastword $(subst :, ,$(filter $(ARCH):%,amd64:amd64 arm64:arm64v8)))
# Build container images using BuildKit strategy.
export DOCKER_BUILDKIT = 1

.PHONY: all build check

# Go compliation specific to selected build and target system architecture.
GOPKG = github.com/amazonlinux/bottlerocket/dogswatch
GOPKGS = $(GOPKG) $(GOPKG)/pkg/... $(GOPKG)/cmd/...
export GOBIN = $(shell pwd -P)/bin
export GOARCH = $(ARCH)

.DEFAULT_GOAL = build

# Run all build tasks for this daemon & its container image.
all: build test container check

# Build the daemon and tools into GOBIN
build:
	go install -v $(GOPKG) $(GOPKG)/cmd/...

# Run Go tests for daemon and tools.
#
# Tests run only with the native GOARCH of the system.
test: GOARCH=
# Use debuggable build to capture more logging for diagnosing failing tests.
test: GO_LDFLAGS +=-X $(GOPKG)/pkg/logging.DebugEnable=true
test:
	go test -race -ldflags '$(GO_LDFLAGS)' $(GOPKGS)

# Build a container image for daemon and tools.
container:
	docker build \
		--network=host \
		--build-arg GO_LDFLAGS \
		--build-arg GOARCH \
		--build-arg SHORT_SHA='$(SHORT_SHA)' \
		--target="dogswatch" \
		--tag $(IMAGE_NAME) \
		.

# Build and test in a container.
container-test:
	docker build \
		--network=host \
		--build-arg GO_LDFLAGS='$(GO_LDFLAGS)' \
		--build-arg GOARCH='$(GOARCH)' \
		--build-arg SHORT_SHA='$(SHORT_SHA)' \
		--build-arg NOCACHE='$(shell date +"%s")' \
		--target="test" \
		--tag $(IMAGE_NAME)-test \
		.

# Build container image with debug-configured daemon.
debug: GO_LDFLAGS +=-X $(GOPKG)/pkg/logging.DebugEnable=true
debug: IMAGE_NAME := $(IMAGE_NAME)-debug
debug: container

# Create a distribution container image tarball for release.
dist: container check
	@mkdir -p $(dir $(DISTFILE))
	docker save $(IMAGE_NAME) | gzip > '$(DISTFILE)'

# Run checks on the container image.
check: check-executable

# Check that the container's executable works.
check-executable:
	@echo "Running check: $@"
	docker run --rm $(IMAGE_NAME) -help 2>&1 \
		| grep -q '/dogswatch'

# Clean the build artifacts on disk.
clean:
	rm -f -- $(foreach binpkg,$(GOPKG) $(wildcard ./cmd/*),'$(GOBIN)/$(notdir $(binpkg))')
	rm -f -- '$(DISTFILE)'

# Development targets

# Deploy the development stack to the environment selected cluster. This
# requires that the image be present as the default name OR specified & pushed
# to a cluster-reachable location.
deploy-dev:
	sed 's,@containerRef@,$(IMAGE_NAME),g' ./dev/deployment.yaml \
		| kubectl apply -f -

# Rollout updates resources and bounces them to effectively "restart" the
# collective service.
rollout: deploy
	kubectl -n bottlerocket rollout restart deployment/dogswatch-controller
	kubectl -n bottlerocket rollout restart daemonset/dogswatch-agent

# Load the docker image into cluster's container image storage ("kind"
# development cluster specific).
kind-load: container
	kind load docker-image $(IMAGE_NAME)

# Rollout but for "kind" development clusters (see `rollout').
kind-rollout: kind-load rollout

# Cluster creates a "kind" based cluster for development.
kind-cluster:
	@hash kind
	kind create cluster --config ./dev/cluster.yaml

dashboard:
	@echo "Dashboard deployment, as configured here, is unauthenticated & intended"
	@echo "for development clusters that are private or only locally accessible."
	@echo
	@echo "To configure, run, and proxy kubernetes-dashboard, use:"
	@echo
	@echo "  \$$ make unsafe-dashboard"
	@echo
	@exit 1

# Spin up a service for the Kubernetes Dashboard - this can be unsafe as any
# access to ClusterIP bound services will automatically authenticate clients as
# fully-privileged.
unsafe-dashboard:
	kubectl apply -f ./dev/dashboard.yaml
	@echo 'Visit dashboard at: http://localhost:8001/api/v1/namespaces/kube-system/services/https:kubernetes-dashboard:/proxy/'
	kubectl proxy

# Print the Node operational management status.
get-nodes-status:
	kubectl get nodes -o json | jq -C -S '.items | map(.metadata|{(.name): (.annotations*.labels|to_entries|map(select(.key|startswith("bottlerocket")))|from_entries)}) | add'
