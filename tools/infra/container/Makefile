DOCKERFILES = $(filter-out %~,$(wildcard Dockerfile.*))
NAMES = $(DOCKERFILES:Dockerfile.%=%)

IMAGE_REPO_PREFIX ?= infra/
DEFAULT_IMAGE_TAG = develop
IMAGE_NAME ?= $(IMAGE_REPO_PREFIX)$(NAME):$(if $(IMAGE_TAG),$(IMAGE_TAG),$(DEFAULT_IMAGE_TAG))

.PHONY: force all release $(NAMES)
.DEFAULT: all
force:

all: $(NAMES)

release : IMAGE_TAG ?= latest
release: $(if $(NAME),$(NAME),$(NAMES))

$(NAMES) : NAME = $@
$(NAMES): force
	@echo "Building container image for '$(NAME)'"
	docker build -t $(IMAGE_NAME) -f Dockerfile.$(NAME) .
