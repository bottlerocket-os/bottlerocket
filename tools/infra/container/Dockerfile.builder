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

COPY scripts /build/scripts
WORKDIR /build

RUN install-rust && configure-rust && install-crates

COPY runtime /build/runtime
COPY builder/entrypoint.sh /build/entrypoint.sh

ENTRYPOINT ["/build/entrypoint.sh"]

CMD [ "bash" ]
