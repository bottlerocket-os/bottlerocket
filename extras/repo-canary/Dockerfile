FROM rust:1.38.0 as builder
WORKDIR /opt/build
COPY . .
RUN cargo install --path . --force -q

FROM amazonlinux:2
ARG METADATA_BASE_URL
ENV METADATA_BASE_URL $METADATA_BASE_URL
ARG TARGET_BASE_URL
ENV TARGET_BASE_URL $TARGET_BASE_URL
RUN yum -y update && yum clean all
RUN mkdir -p /usr/share/repo-canary

COPY root.json /usr/share/repo-canary/
COPY --from=builder /opt/build/target/release/repo-canary /usr/bin/

CMD /usr/bin/repo-canary --metadata-base-url $METADATA_BASE_URL --target-base-url $TARGET_BASE_URL --percentage-of-targets-to-retrieve 50
