.PHONY: buildkitd

buildkitd:
	@echo "Starting buildkitd server on $(BUILDKITD_ADDR)"
	docker run --network=host \
		--volume /var/run/docker.sock:/var/run/docker.sock:ro \
		--rm --privileged moby/buildkit:v0.3.3 \
		--addr $(BUILDKITD_ADDR) \
		--oci-worker true
