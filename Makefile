.DEFAULT_GOAL := all

OS := thar
TOPDIR := $(strip $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST)))))
SPEC2VAR ?= $(TOPDIR)/bin/spec2var
SPEC2PKG ?= $(TOPDIR)/bin/spec2pkg

SPECS = $(wildcard packages/*/*.spec)
VARS = $(SPECS:.spec=.makevar)
PKGS = $(SPECS:.spec=.makepkg)

OUTPUT ?= $(TOPDIR)/build
OUTVAR := $(shell mkdir -p $(OUTPUT))
DATE := $(shell date --rfc-3339=date)

ARCHS := x86_64 aarch64

BUILDKITD_ADDR ?= tcp://127.0.0.1:1234
BUILDCTL ?= buildctl --addr $(BUILDKITD_ADDR)
BUILDCTL_ARGS := --progress=plain
BUILDCTL_ARGS += --frontend=dockerfile.v0
BUILDCTL_ARGS += --local context=.
BUILDCTL_ARGS += --local dockerfile=.

DOCKER ?= docker

define build_rpm
	$(eval HASH:= $(shell sha1sum $3 /dev/null | sha1sum - | awk '{printf $$1}'))
	$(eval RPMS:= $(shell echo $3 | tr ' ' '\n' | awk '/.rpm$$/' | tr '\n' ' '))
	@$(BUILDCTL) build \
		--frontend-opt target=rpm \
		--frontend-opt build-arg:PACKAGE=$(1) \
		--frontend-opt build-arg:ARCH=$(2) \
		--frontend-opt build-arg:HASH=$(HASH) \
		--frontend-opt build-arg:RPMS="$(RPMS)" \
		--frontend-opt build-arg:DATE=$(DATE) \
		--exporter=local \
		--exporter-opt output=$(OUTPUT) \
		$(BUILDCTL_ARGS)
endef

define build_fs
	$(eval HASH:= $(shell sha1sum $(2) /dev/null | sha1sum - | awk '{print $$1}'))
	@$(BUILDCTL) build \
		--frontend-opt target=fs \
		--frontend-opt build-arg:PACKAGE=$(OS)-$(1)-release \
		--frontend-opt build-arg:ARCH=$(1) \
		--frontend-opt build-arg:HASH=$(HASH) \
		--frontend-opt build-arg:DATE=$(DATE) \
		--exporter=docker \
		--exporter-opt name=$(OS):$(1) \
		--exporter-opt output=build/$(OS)-$(1).tar \
		$(BUILDCTL_ARGS) ; \
	$(DOCKER) load < build/$(OS)-$(1).tar
endef

empty :=
space := $(empty) $(empty)
comma := ,
list = $(subst $(space),$(comma),$(1))

%.makevar : %.spec $(SPEC2VAR)
	@set -e; $(SPEC2VAR) --spec=$< --archs=$(call list,$(ARCHS)) > $@

%.makepkg : %.spec $(SPEC2PKG)
	@set -e; $(SPEC2PKG) --spec=$< --archs=$(call list,$(ARCHS)) > $@

-include $(VARS)
-include $(PKGS)

.PHONY: all $(ARCHS)

.SECONDEXPANSION:
$(ARCHS): $$($(OS)-$$(@)-release)
	$(eval PKGS:= $(wildcard $(OUTPUT)/$(OS)-$(@)-*.rpm))
	$(call build_fs,$@,$(PKGS))

all: $(ARCHS)

.PHONY: clean
clean:
	@rm -f $(OUTPUT)/*.rpm

include $(TOPDIR)/hack/rules.mk
