FROM golang:1.13 as builder
ENV GO111MODULE=on
ENV GOPROXY=off
COPY ./ /go/src/github.com/amazonlinux/thar/dogswatch/
RUN cd /go/src/github.com/amazonlinux/thar/dogswatch && \
    go install -mod=vendor ./...

FROM golang:1.13
COPY --from=builder /go/bin/dogswatch /dogswatch
ENTRYPOINT ["/dogswatch"]
CMD ["-help"]
