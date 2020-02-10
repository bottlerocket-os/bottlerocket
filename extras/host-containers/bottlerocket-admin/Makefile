# IMAGE_NAME is the full name of the container image being built.
IMAGE_NAME ?= $(notdir $(shell pwd -P))$(IMAGE_ARCH_SUFFIX):$(IMAGE_VERSION)$(addprefix -,$(SHORT_SHA))
# IMAGE_VERSION is the semver version that's tagged on the image.
IMAGE_VERSION = $(shell cat VERSION)
# SHORT_SHA is the revision that the container image was built with.
SHORT_SHA ?= $(shell git describe --abbrev=8 --always --dirty='-dev' --exclude '*' || echo "unknown")
# IMAGE_ARCH_SUFFIX is the runtime architecture designator for the container
# image, it is appended to the IMAGE_NAME unless the name is specified.
IMAGE_ARCH_SUFFIX ?= $(addprefix -,$(ARCH))
# DESTDIR is where the release artifacts will be written.
DESTDIR ?= .
# DISTFILE is the path to the dist target's output file - the container image
# tarball.
DISTFILE ?= $(subst /,,$(DESTDIR))/$(subst /,_,$(IMAGE_NAME)).tar.gz

# These values derive ARCH and DOCKER_ARCH which are needed by dependencies in
# image build defaulting to system's architecture when not specified.
#
# UNAME_ARCH is the runtime architecture of the building host.
UNAME_ARCH = $(shell uname -m)
# ARCH is the target architecture which is being built for.
ARCH ?= $(lastword $(subst :, ,$(filter $(UNAME_ARCH):%,x86_64:amd64 aarch64:arm64)))
# DOCKER_ARCH is the docker specific architecture specifier used for building on
# multiarch container images.
DOCKER_ARCH ?= $(lastword $(subst :, ,$(filter $(ARCH):%,amd64:amd64 arm64:arm64v8)))

.PHONY: all build check check-static-bash

# Run all build tasks for this container image.
all: build check

# Create a distribution container image tarball for release.
dist: all
	@mkdir -p $(dir $(DISTFILE))
	docker save $(IMAGE_NAME) | gzip > $(DISTFILE)

# Build the container image.
build:
	docker build \
		--tag $(IMAGE_NAME) \
		--build-arg IMAGE_VERSION="$(IMAGE_VERSION)" \
		--build-arg ARCH="$(ARCH)" \
		--build-arg DOCKER_ARCH="$(DOCKER_ARCH)" \
		-f Dockerfile . >&2

# Run checks against the container image.
check: check-static-bash

# Check that bash can be run without dependency on shared libraries.
check-static-bash:
	docker run --rm --entrypoint /opt/bin/bash \
		-v nolib:/usr/lib -v nolib:/usr/lib64 \
		$(IMAGE_NAME) \
		-c '/usr/bin/bash -c "echo \$$0 must not run" 2>/dev/null && exit 1 || exit 0'

clean:
	rm -f $(DISTFILE)
