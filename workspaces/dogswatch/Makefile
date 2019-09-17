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
	go test -v $(GOPKGS)
