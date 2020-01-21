IMAGE_VERSION=`cat VERSION`

DOCKER_IMAGE := thar-admin
DOCKER_IMAGE_REF_RELEASE := $(DOCKER_IMAGE):$(ADMIN_CTR_VERSION)
SHORT_SHA ?= $(shell git rev-parse --short=8 HEAD)
DOCKER_IMAGE_REF := $(DOCKER_IMAGE):$(SHORT_SHA)

container:
	docker build --network=host \
		--tag $(DOCKER_IMAGE_REF) \
		--build-arg IMAGE_VERSION="$(IMAGE_VERSION)" \
		--build-arg BUILD_LDFLAGS='' \
		.

container-simple-test:
	docker run --rm ${DOCKER_IMAGE_REF} cat /etc/motd

release-container: container
	docker tag $(DOCKER_IMAGE_REF) $(DOCKER_IMAGE_REF_RELEASE)
