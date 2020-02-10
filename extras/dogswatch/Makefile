# This is the NEXT version tag for the Dogswatch container image.
DOGSWATCH_VERSION=`cat VERSION`

GOPKG = github.com/amazonlinux/bottlerocket/dogswatch
GOPKGS = $(GOPKG) $(GOPKG)/pkg/... $(GOPKG)/cmd/...
GOBIN = ./bin/
DOCKER_IMAGE := dogswatch
DOCKER_IMAGE_REF_RELEASE := $(DOCKER_IMAGE):$(DOGSWATCH_VERSION)
SHORT_SHA ?= $(shell git rev-parse --short=8 HEAD)
DOCKER_IMAGE_REF := $(DOCKER_IMAGE):$(SHORT_SHA)
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
		--tag $(DOCKER_IMAGE_REF) \
		--build-arg BUILD_LDFLAGS='' \
		.

container-simple-test:
	docker run --rm $(DOCKER_IMAGE_REF) -help 2>&1 | grep -C 10 'dogswatch'

debug-container:
	docker build --network=host \
		--tag $(DOCKER_IMAGE_REF)\
		--build-arg BUILD_LDFLAGS='$(DEBUG_LDFLAGS)' \
		.

release-container: container
	docker tag $(DOCKER_IMAGE_REF) $(DOCKER_IMAGE_REF_RELEASE)

load: container
	kind load docker-image $(DOCKER_IMAGE)

deploy:
	sed 's,@containerRef@,$(DOCKER_IMAGE_REF),g' ./dev/deployment.yaml \
		| kubectl apply -f -

rollout: deploy
	kubectl -n bottlerocket rollout restart deployment/dogswatch-controller
	kubectl -n bottlerocket rollout restart daemonset/dogswatch-agent

rollout-kind: load rollout

cluster:
	kind create cluster --config ./dev/cluster.yaml

dashboard:
	kubectl apply -f ./dev/dashboard.yaml
	@echo 'Visit dashboard at: http://localhost:8001/api/v1/namespaces/kube-system/services/https:kubernetes-dashboard:/proxy/'
	kubectl proxy

get-nodes-status:
	kubectl get nodes -o json | jq -C -S '.items | map(.metadata|{(.name): (.annotations*.labels|to_entries|map(select(.key|startswith("bottlerocket")))|from_entries)}) | add'
