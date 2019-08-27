.DEFAULT_GOAL := all

OS := thar
RECIPE ?= aws-eks-ami
TOPDIR := $(strip $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST)))))
DEP4SPEC ?= $(TOPDIR)/bin/dep4spec
SPEC2VAR ?= $(TOPDIR)/bin/spec2var
SPEC2PKG ?= $(TOPDIR)/bin/spec2pkg
FETCH_UPSTREAM ?= $(TOPDIR)/bin/fetch-upstream
UPLOAD_SOURCES ?= $(TOPDIR)/bin/upload-sources
export ALLOW_ARBITRARY_SOURCE_URL ?= true

SPECS = $(wildcard packages/*/*.spec)
DEPS = $(SPECS:.spec=.makedep)
VARS = $(SPECS:.spec=.makevar)
PKGS = $(SPECS:.spec=.makepkg)

OUTPUT ?= $(TOPDIR)/build
OUTVAR := $(shell mkdir -p $(OUTPUT))
DATE := $(shell date --rfc-3339=date)

ARCH ?= $(shell uname -m)

DOCKER ?= docker

BUILDKIT_VER = v0.4.0
BUILDKITD_ADDR ?= tcp://127.0.0.1:1234
BUILDCTL_DOCKER_RUN = $(DOCKER) run --rm -t --entrypoint /usr/bin/buildctl --user $(shell id -u):$(shell id -g) --volume $(TOPDIR):$(TOPDIR) --workdir $(TOPDIR) --network host moby/buildkit:$(BUILDKIT_VER)
BUILDCTL ?= $(BUILDCTL_DOCKER_RUN) --addr $(BUILDKITD_ADDR)
BUILDCTL_ARGS := --progress=plain
BUILDCTL_ARGS += --frontend=dockerfile.v0
BUILDCTL_ARGS += --local context=.
BUILDCTL_ARGS += --local dockerfile=.

define build_rpm
	$(eval HASH:= $(shell sha1sum $3 /dev/null | sha1sum - | awk '{printf $$1}'))
	$(eval RPMS:= $(shell echo $3 | tr ' ' '\n' | awk '/.rpm$$/' | tr '\n' ' '))
	@$(BUILDCTL) build \
		--opt target=rpm \
		--opt build-arg:PACKAGE=$(1) \
		--opt build-arg:ARCH=$(2) \
		--opt build-arg:HASH=$(HASH) \
		--opt build-arg:RPMS="$(RPMS)" \
		--opt build-arg:DATE=$(DATE) \
		--output type=local,dest=$(OUTPUT) \
		$(BUILDCTL_ARGS)
endef

define build_image
	$(eval HASH:= $(shell sha1sum $(2) /dev/null | sha1sum - | awk '{print $$1}'))
	@$(BUILDCTL) build \
		--opt target=image \
		--opt build-arg:PACKAGE=$(OS)-$(1)-$(RECIPE) \
		--opt build-arg:ARCH=$(1) \
		--opt build-arg:HASH=$(HASH) \
		--opt build-arg:DATE=$(DATE) \
		--output type=local,dest=$(OUTPUT) \
		$(BUILDCTL_ARGS)
	lz4 -d -f $(OUTPUT)/$(OS)-$(1).img.lz4 $(OUTPUT)/$(OS)-$(1).img \
		&& rm -f $(OUTPUT)/$(OS)-$(1).img.lz4
endef

# `makedep` files are a hook to provide additional dependencies when
# building `makevar` and `makepkg` files. The intended use case is
# to generate source files that must be in place before parsing the
# spec file.
%.makedep : %.spec $(DEP4SPEC)
	@$(DEP4SPEC) --spec=$< --arch=$(ARCH) > $@.tmp
	@mv $@.tmp $@

# `makevar` files generate variables that the `makepkg` files for
# other packages can refer to. All `makevar` files must be evaluated
# before any `makepkg` files, or else empty values could be added to
# the dependency list.
%.makevar : %.spec %.makedep $(SPEC2VAR)
	@$(SPEC2VAR) --spec=$< --arch=$(ARCH) > $@.tmp
	@mv $@.tmp $@

# `makepkg` files define the package outputs obtained by building
# the spec file, as well as the dependencies needed to build that
# package.
%.makepkg : %.spec %.makedep %.makevar $(SPEC2PKG)
	@$(SPEC2PKG) --spec=$< --arch=$(ARCH) > $@.tmp
	@mv $@.tmp $@

include $(DEPS)
include $(VARS)
include $(PKGS)

.PHONY: all $(ARCH)

.SECONDEXPANSION:
$(ARCH): $$($(OS)-$(ARCH)-$(RECIPE))
	$(eval PKGS:= $(wildcard $(OUTPUT)/$(OS)-$(ARCH)-*.rpm))
	$(call build_image,$@,$(PKGS))

all: $(ARCH)

.PHONY: clean
clean:
	@rm -f $(OUTPUT)/*.rpm $(OUTPUT)/*.tar $(OUTPUT)/*.lz4 $(OUTPUT)/*.img
	@find $(TOPDIR) -name '*.make*' -delete

.PHONY: sources
sources: $(SOURCES)

.PHONY: upload-sources
upload-sources: $(SOURCES)
	@$(UPLOAD_SOURCES) $^

include $(TOPDIR)/hack/rules.mk
