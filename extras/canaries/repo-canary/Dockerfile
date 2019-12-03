FROM rust:1.38.0 as builder
WORKDIR /opt/build
COPY . .
RUN cargo install --path .

FROM amazonlinux:2
RUN yum -y update && yum clean all
RUN mkdir -p /usr/share/repo-canary

COPY root.json /usr/share/repo-canary/
COPY --from=builder /opt/build/target/release/repo-canary /usr/bin/

ENTRYPOINT ["/usr/bin/repo-canary"]
