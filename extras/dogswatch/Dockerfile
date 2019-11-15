# syntax=docker/dockerfile:experimental
FROM golang:1.13 as builder
ENV GOPROXY=direct
COPY ./ /go/src/github.com/amazonlinux/thar/dogswatch/
RUN cd /go/src/github.com/amazonlinux/thar/dogswatch && \
    CGO_ENABLED=0 GOOS=linux go build -o dogswatch . && mv dogswatch /dogswatch

FROM scratch
COPY --from=builder /dogswatch /etc/ssl /
ENTRYPOINT ["/dogswatch"]
CMD ["-help"]
