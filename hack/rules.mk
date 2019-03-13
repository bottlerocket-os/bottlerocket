.PHONY: buildkitd

buildkitd:
	@echo "Starting buildkitd server on $(BUILDKITD_ADDR)"
	docker run -t --network=host \
		--volume /var/run/docker.sock:/var/run/docker.sock:ro \
		--rm --privileged moby/buildkit:$(BUILDKIT_VER) \
		--addr $(BUILDKITD_ADDR) \
		--oci-worker true
