
# syntax=docker/dockerfile:experimental
FROM golang:1.13 as builder
ARG BUILD_LDFLAGS
ENV BUILD_LDFLAGS=$BUILD_LDFLAGS
ENV GOPROXY=direct
COPY ./ /go/src/github.com/amazonlinux/thar/dogswatch/
RUN cd /go/src/github.com/amazonlinux/thar/dogswatch && \
    CGO_ENABLED=0 GOOS=linux go build -mod=readonly ${BUILD_LDFLAGS:+-ldflags "$BUILD_LDFLAGS"} \
    -o dogswatch . && mv dogswatch /dogswatch

FROM scratch
COPY --from=builder /dogswatch /etc/ssl /
ENTRYPOINT ["/dogswatch"]
CMD ["-help"]
