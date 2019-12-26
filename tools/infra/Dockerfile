FROM amazonlinux:2 as base
RUN yum update -y \
	&& yum groupinstall -y 'Development Tools' \
	&& yum install -y socat procps-ng awscli jq openssh rsync \
	&& amazon-linux-extras install -y docker \
	&& yum clean all \
	&& rm -rf /var/cache/yum /var/cache/amzn2extras

FROM base
ENV PATH="$PATH:/build/env/bin:/build/.cargo/bin"
ENV CARGO_HOME="/build/.cargo"
COPY env/ /build/env/
WORKDIR /build

RUN source setup-rust-builder

ENTRYPOINT ["/build/env/container/entrypoint.sh"]
CMD [ "bash" ]
