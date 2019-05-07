.DEFAULT_GOAL := all

OS := thar
TOPDIR := $(strip $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST)))))
DEP4SPEC ?= $(TOPDIR)/bin/dep4spec
SPEC2VAR ?= $(TOPDIR)/bin/spec2var
SPEC2PKG ?= $(TOPDIR)/bin/spec2pkg

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
		--opt target=builder \
		--opt build-arg:PACKAGE=$(OS)-$(1)-release \
		--opt build-arg:ARCH=$(1) \
		--opt build-arg:HASH=$(HASH) \
		--opt build-arg:DATE=$(DATE) \
		--output type=docker,name=$(OS)-builder:$(1),dest=build/$(OS)-$(1)-builder.tar \
		$(BUILDCTL_ARGS)
	@$(DOCKER) load < build/$(OS)-$(1)-builder.tar
	@$(DOCKER) run -t -v /dev:/dev -v $(OUTPUT):/local/output \
		$(OS)-builder:$(1) \
			--disk-image-name=$(OS)-$(1).img \
			--boot-image-name=$(OS)-$(1)-boot.ext4.lz4 \
			--verity-image-name=$(OS)-$(1)-root.verity.lz4 \
			--root-image-name=$(OS)-$(1)-root.ext4.lz4 \
			--package-dir=/local/rpms \
			--output-dir=/local/output
endef

# `makedep` files are a hook to provide additional dependencies when
# building `makevar` and `makepkg` files. The intended use case is
# to generate source files that must be in place before parsing the
# spec file.
%.makedep : %.spec $(DEP4SPEC)
	@$(DEP4SPEC) --spec=$< > $@.tmp
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
$(ARCH): $$($(OS)-$(ARCH)-release)
	$(eval PKGS:= $(wildcard $(OUTPUT)/$(OS)-$(ARCH)-*.rpm))
	$(call build_image,$@,$(PKGS))

all: $(ARCH)

.PHONY: clean
clean:
	@rm -f $(OUTPUT)/*.rpm

include $(TOPDIR)/hack/rules.mk
