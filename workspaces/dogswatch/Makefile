GOBIN = ./bin/

build: gogenerate $(GOBIN)
	cd $(GOBIN) && \
	go build -v -x github.com/amazonlinux/thar/dogswatch && \
	go build -v -x  github.com/amazonlinux/thar/dogswatch/cmd/...

gogenerate:
	go generate -v github.com/amazonlinux/thar/dogswatch/...

$(GOBIN):
	mkdir -p $(GOBIN)
