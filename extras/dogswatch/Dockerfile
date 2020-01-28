# syntax=docker/dockerfile:experimental
FROM golang:1.13 as build
ARG GO_LDFLAGS=
ARG GOARCH=
ARG SHORT_SHA=
ENV GOPROXY=direct
COPY go.mod go.sum /go/src/github.com/amazonlinux/bottlerocket/dogswatch/
WORKDIR /go/src/github.com/amazonlinux/bottlerocket/dogswatch
RUN go mod download
COPY . /go/src/github.com/amazonlinux/bottlerocket/dogswatch/
RUN make -e build GOBIN=/ CGO_ENABLED=0

# Build minimal container with a static build of dogswatch.
FROM scratch as dogswatch
COPY --from=build /dogswatch /etc/ssl /
ENTRYPOINT ["/dogswatch"]
CMD ["-help"]

FROM build as test
# Accept a cache-busting value to explicitly run tests.
ARG NOCACHE=
RUN NOCACHE=$NOCACHE make -e test

# Make container the output of a plain 'docker build'.
FROM dogswatch
