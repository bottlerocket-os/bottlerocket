GOPKG = github.com/amazonlinux/thar/dogswatch
GOPKGS = $(GOPKG) $(GOPKG)/pkg/... $(GOPKG)/cmd/...
GOBIN = ./bin/

build: gogenerate $(GOBIN)
	cd $(GOBIN) && \
	go build -v -x $(GOPKG) && \
	go build -v -x  $(GOPKG)/cmd/...

gogenerate:
	go generate -v $(GOPKGS)

$(GOBIN):
	mkdir -p $(GOBIN)

test:
	go test $(GOPKGS)

container: vendor
	docker build --network=host -t dogswatch:$$(git describe --always --dirty) .

vendor: go.sum go.mod
	go mod vendor
	touch vendor/

deploy:
	sed 's/@containerRef@/dogswatch:$(shell git describe --always --dirty)/g' ./deployment.yaml \
		| kubectl apply -f -
