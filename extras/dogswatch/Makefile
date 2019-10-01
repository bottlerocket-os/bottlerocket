GOPKG = github.com/amazonlinux/thar/dogswatch
GOPKGS = $(GOPKG) $(GOPKG)/pkg/... $(GOPKG)/cmd/...
GOBIN = ./bin/
DOCKER_IMAGE := dogswatch
DOCKER_IMAGE_REF := $(DOCKER_IMAGE):$(shell git describe --always --dirty)

build: $(GOBIN)
	cd $(GOBIN) && \
	go build -v -x $(GOPKG) && \
	go build -v -x  $(GOPKG)/cmd/...

$(GOBIN):
	mkdir -p $(GOBIN)

test:
	go test $(GOPKGS)

container: vendor
	DOCKER_BUILDKIT=1 docker build --network=host -t $(DOCKER_IMAGE_REF) .

load: container
	kind load docker-image $(DOCKER_IMAGE)

vendor: go.sum go.mod
	CGO_ENABLED=0 GOOS=linux go mod vendor
	touch vendor/

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
	kubectl get nodes -o json | jq -C -S '.items| map({(.metadata.name): (.metadata.labels * .metadata.annotations)})'
