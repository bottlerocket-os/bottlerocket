.DEFAULT_GOAL := all

TOPDIR := $(strip $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST)))))
SPEC2VAR ?= $(TOPDIR)/bin/spec2var
SPEC2PKG ?= $(TOPDIR)/bin/spec2pkg

SPECS = $(wildcard packages/*/*.spec)
VARS = $(SPECS:.spec=.makevar)
PKGS = $(SPECS:.spec=.makepkg)

OUTPUT ?= $(TOPDIR)/build
OUTVAR := $(shell mkdir -p $(OUTPUT))

ARCHS := x86_64 aarch64

BUILDCTL ?= buildctl --addr tcp://127.0.0.1:1234
BUILDCTL_ARGS := --progress=plain
BUILDCTL_ARGS += --frontend=dockerfile.v0
BUILDCTL_ARGS += --local context=.
BUILDCTL_ARGS += --local dockerfile=.

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

.PHONY: all
all: $(thar-x86_64-ncurses) $(thar-aarch64-glibc)
	@echo BUILT IT ALL

.PHONY: clean
clean:
	@rm -f $(OUTPUT)/*.rpm
