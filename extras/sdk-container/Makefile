ARCH ?= $(shell uname -m)

VERSION := v0.4
TAG := thar/sdk-$(ARCH):$(VERSION)
ARCHIVE := thar-sdk-$(ARCH)-$(VERSION).tar.gz

$(ARCHIVE) :
	@DOCKER_BUILDKIT=1 docker build . -t $(TAG) --squash --build-arg ARCH=$(ARCH)
	@docker image save $(TAG) | gzip --fast > $(@)

.PHONY: upload clean

upload : $(ARCHIVE)
	@aws s3 cp $(ARCHIVE) s3://thar-upstream-lookaside-cache/$(TAG).tar.gz

clean:
	@rm -f *.tar.gz
