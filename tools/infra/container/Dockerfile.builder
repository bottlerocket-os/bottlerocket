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
	&& yum install -y socat procps-ng awscli jq openssh rsync systemd-devel \
	&& amazon-linux-extras enable docker \
	&& yum install -y docker amazon-ecr-credential-helper \
	&& yum clean all \
	&& rm -rf /var/cache/yum /var/cache/amzn2extras
RUN install -D /dev/null /root/.docker/config.json \
	&& echo '{ "credsStore": "ecr-login" }' >> /root/.docker/config.json

FROM base
ENV PATH="$PATH:/build/runtime/bin:/build/scripts:/build/.cargo/bin"
ENV CARGO_HOME="/build/.cargo"
ENV RUNTIME_SCRIPT_LIB="/build/runtime/lib"

COPY scripts /build/scripts
COPY runtime /build/runtime
WORKDIR /build

RUN install-rust && configure-rust && install-crates

COPY builder/entrypoint.sh /build/entrypoint.sh

ENTRYPOINT ["/build/entrypoint.sh"]

CMD [ "bash" ]
