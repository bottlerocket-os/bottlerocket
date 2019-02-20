.DEFAULT_GOAL := all

TOPDIR := $(strip $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST)))))
SPEC2VAR ?= $(TOPDIR)/bin/spec2var
SPEC2PKG ?= $(TOPDIR)/bin/spec2pkg

SPECS = $(wildcard packages/*/*.spec)
VARS = $(SPECS:.spec=.makevar)
PKGS = $(SPECS:.spec=.makepkg)

OUTPUT ?= $(TOPDIR)/build
OUTVAR := $(shell mkdir -p $(OUTPUT))

BUILDCTL ?= buildctl --addr tcp://127.0.0.1:1234
BUILDCTL_ARGS := --progress=plain
BUILDCTL_ARGS += --frontend=dockerfile.v0
BUILDCTL_ARGS += --local context=.
BUILDCTL_ARGS += --local dockerfile=.

%.makevar : %.spec $(SPEC2VAR)
	@$(SPEC2VAR) $< > $@

%.makepkg : %.spec $(SPEC2PKG)
	@$(SPEC2PKG) $< > $@

-include $(VARS)
-include $(PKGS)

.PHONY: all
all: $(thar-sdk)
	@echo BUILT IT ALL

.PHONY: clean
clean:
	@rm -r $(OUTPUT)/*.rpm
