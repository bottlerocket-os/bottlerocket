# This is the NEXT version tag for the Dogswatch container image.
DOGSWATCH_VERSION=v0.1.2

GOPKG = github.com/amazonlinux/thar/dogswatch
GOPKGS = $(GOPKG) $(GOPKG)/pkg/... $(GOPKG)/cmd/...
GOBIN = ./bin/
DOCKER_IMAGE := dogswatch
DOCKER_IMAGE_REF_RELEASE := $(DOCKER_IMAGE):$(DOGSWATCH_VERSION)
DOCKER_IMAGE_REF := $(DOCKER_IMAGE):$(shell git rev-parse --short=8 HEAD)
DEBUG_LDFLAGS := -X $(GOPKG)/pkg/logging.DebugEnable=true

build: $(GOBIN)
	cd $(GOBIN) && \
	go build -v -x $(GOPKG) && \
	go build -v -x  $(GOPKG)/cmd/...

$(GOBIN):
	mkdir -p $(GOBIN)

test:
	go test -ldflags '$(DEBUG_LDFLAGS)' $(GOPKGS)

container:
	docker build --network=host \
		--tag $(DOCKER_IMAGE_REF)\
		--build-arg BUILD_LDFLAGS='' \
		.

debug-container:
	docker build --network=host \
		--tag $(DOCKER_IMAGE_REF)\
		--build-arg BUILD_LDFLAGS='$(DEBUG_LDFLAGS)' \
		.


release-container:
	docker build --network=host \
		--tag $(DOCKER_IMAGE_REF_RELEASE) \
		--tag $(DOCKER_IMAGE):latest \
		--build-arg BUILD_LDFLAGS='' \
		.

load: container
	kind load docker-image $(DOCKER_IMAGE)

deploy:
	sed 's,@containerRef@,$(DOCKER_IMAGE_REF),g' ./dev/deployment.yaml \
		| kubectl apply -f -

rollout: deploy
	kubectl -n thar rollout restart deployment/dogswatch-controller
	kubectl -n thar rollout restart daemonset/dogswatch-agent

rollout-kind: load rollout

cluster:
	kind create cluster --config ./dev/cluster.yaml

dashboard:
	kubectl apply -f ./dev/dashboard.yaml
	@echo 'Visit dashboard at: http://localhost:8001/api/v1/namespaces/kube-system/services/https:kubernetes-dashboard:/proxy/'
	kubectl proxy

get-nodes-status:
	kubectl get nodes -o json | jq -C -S '.items | map(.metadata|{(.name): (.annotations*.labels|to_entries|map(select(.key|startswith("thar")))|from_entries)}) | add'
