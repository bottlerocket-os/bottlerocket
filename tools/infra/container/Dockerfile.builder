# Dockerfile.builder - Base build environment container image
#
# The builder image provides an environment in which packages and images may be
# built. This includes the necessary compilers, libraries, services, and
# executable dependencies used in the course of the build process.
#
# Facilitating scripts may be found in the ./runtime and ./scripts directory
# where scripts are generally participants in the build of the environment.
#
FROM amazonlinux:2 as base
RUN yum update -y \
	&& yum groupinstall -y 'Development Tools' \
	&& yum install -y socat procps-ng awscli jq openssh rsync systemd-devel openssl-devel \
	&& amazon-linux-extras enable docker \
	&& yum install -y docker amazon-ecr-credential-helper \
	&& yum clean all \
	&& rm -rf /var/cache/yum /var/cache/amzn2extras
RUN install -D /dev/null /root/.docker/config.json \
	&& echo '{ "credsStore": "ecr-login" }' >> /root/.docker/config.json

FROM base as buildenv
ENV PATH="$PATH:/build/runtime/bin:/build/scripts:/build/.cargo/bin"
ENV CARGO_HOME="/build/.cargo"
ENV RUNTIME_SCRIPT_LIB="/build/runtime/lib"
COPY tools/infra/container/scripts /build/scripts
COPY tools/infra/container/runtime /build/runtime
# FIXME: remove depedency on top level source - #656
COPY bin/amiize.sh /build/runtime/bin/amiize.sh
RUN install-rust && configure-rust && install-crates

FROM buildenv as signing-tool
COPY . /build/src
RUN cd /build/src/tools/update_sign_tuf_repo && \
  cargo build --release

FROM buildenv
COPY --from=signing-tool /build/src/tools/update_sign_tuf_repo/target/release/update_sign_tuf_repo /build/runtime/bin/update_sign_tuf_repo
WORKDIR /build
COPY tools/infra/container/builder/entrypoint.sh /build/entrypoint.sh
ENTRYPOINT ["/build/entrypoint.sh"]
CMD [ "bash" ]
