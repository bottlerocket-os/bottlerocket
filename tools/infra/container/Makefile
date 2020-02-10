# SHELL is bash, silly sh.
SHELL = bash
# DOCKERFILES are the detected container images that are being worked
# with. It is expected that NAME be part of the file name, as in
# Dockerfile.NAME, which is used throughout the infrastructure.
DOCKERFILES = $(filter-out %~,$(wildcard Dockerfile.*))
# NAMES are the detected NAMES given the provided Dockerfiles.
NAMES = $(DOCKERFILES:Dockerfile.%=%)
# IMAGE_REPO_PREFIX is prepended to the image's tag. In the case of
# `push', the IMAGE_REPO_PREFIX provides the ECR repository URI prefix
# for each image.
IMAGE_REPO_PREFIX ?= infra/
# IMAGE_TAG provides the registry/image-name:IMAGE_TAG portion of the
# URI tagged to images.
IMAGE_TAG ?= develop
# IMAGE_NAME is the name that the container image is tagged with.
IMAGE_NAME ?= $(IMAGE_REPO_PREFIX)$(NAME):$(IMAGE_TAG)
# ECR_URI_PREFIX is the ECR URI prefix based on the resolved builder
# image URI which, like other container images, is discoverable under
# its in-region SSM parameter - so we can lob off the builder part and
# use it as our model for the pushed repository name.
ECR_URI_PREFIX = $(shell aws ssm get-parameter --name /infra/container/infra/builder --query Parameter.Value --output text | sed 's/builder$$//')
# ECR_NAME_PREFIX provides a prefix to derive the ECR repository-name
# (the attribute) from the images' NAME - the infra/ prefix is
# conventional across automations' consumed images.
ECR_NAME_PREFIX ?= infra/

# This is a reference to the root of the Bottlerocket repo (relative to the current path)
# so later we can run `docker build` with this path. This allows us to
# more easily copy the Bottlerocket repo into a container, which in turn makes it simpler
# to build Rust code that has dependencies on other code in the repo.
# FIXME: when we split this code out we will need to handle this differently
BOTTLEROCKET_SRC = ../../../

.DEFAULT: all
.PHONY: force all release $(NAMES)
force:

all: $(if $(NAME),$(NAME),$(NAMES))

$(NAMES) : NAME = $@
$(NAMES): force
	@echo "Building container image for '$(NAME)'"
	docker build -t $(IMAGE_NAME) -f Dockerfile.$(NAME) $(BOTTLEROCKET_SRC)

# Push images (must explicitly provide IMAGE_TAG=release to be pulled
# by consumers).
push: IMAGE_REPO_PREFIX = $(ECR_URI_PREFIX)
push: IMAGE_TAG = staging
push: all
	@echo "Pushing container images with tag '$(IMAGE_TAG)'"
	@echo "Images: $(foreach NAME,$(NAMES),$(IMAGE_NAME))"
	@$(foreach NAME,$(NAMES),\
		echo "Pushing '$(NAME)' to '$(IMAGE_NAME)'" && \
		aws ecr describe-repositories --repository-names $(ECR_NAME_PREFIX)$(NAME) &> /dev/null \
		&& docker push $(IMAGE_NAME) \
		|| echo "Could not push $(NAME) to ECR repository as $(IMAGE_NAME)";)
