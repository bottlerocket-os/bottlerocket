# syntax=docker/dockerfile:experimental
FROM golang:1.13 as builder
ENV GO111MODULE=on
ENV GOPROXY=direct
COPY ./ /go/src/github.com/amazonlinux/thar/dogswatch/
RUN --mount=type=cache,target=/root/.cache/go-build \
    --mount=type=cache,target=/go/pkg \
    cd /go/src/github.com/amazonlinux/thar/dogswatch && \
    CGO_ENABLED=0 GOOS=linux go build -o dogswatch . && mv dogswatch /dogswatch

FROM scratch
COPY --from=builder /dogswatch /etc/ssl /
ENTRYPOINT ["/dogswatch"]
CMD ["-help"]
