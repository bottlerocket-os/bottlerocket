FROM amazonlinux:2 as base
RUN yum update -y \
	&& yum groupinstall -y 'Development Tools' \
	&& yum install -y socat procps-ng awscli jq openssh rsync \
	&& amazon-linux-extras install -y docker \
	&& yum clean all \
	&& rm -rf /var/cache/yum /var/cache/amzn2extras

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
