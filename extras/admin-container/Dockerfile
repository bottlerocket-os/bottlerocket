FROM fedora:30 as builder
RUN sed -i 's/^enabled=.*/enabled=0/' /etc/yum.repos.d/*modular*.repo
RUN dnf group install -y "C Development Tools and Libraries"
RUN dnf install -y glibc-static patch

ARG bash_version=5.0
ARG bash_patch_level=11

WORKDIR /opt/build
RUN curl -L https://ftp.gnu.org/gnu/bash/bash-${bash_version}.tar.gz | tar -xz
WORKDIR /opt/build/bash-${bash_version}
RUN for patch_level in $(seq ${bash_patch_level}); do \
        curl -L https://ftp.gnu.org/gnu/bash/bash-${bash_version}-patches/bash${bash_version//.}-$(printf '%03d' $patch_level) | patch -p0; \
    done
RUN CFLAGS="-Os -DHAVE_DLOPEN=0" ./configure \
        --enable-static-link \
        --without-bash-malloc \
    || { cat config.log; exit 1; }
RUN make -j`nproc`
RUN cp bash /opt/bash

FROM amazonlinux:2

RUN yum -y update && yum -y install openssh-server sudo util-linux && yum clean all
RUN rm -f /etc/motd /etc/issue

COPY --from=builder /opt/bash /opt/bin/

ADD --chown=root:root ec2-user.sudoers /etc/sudoers.d/ec2-user
ADD --chown=root:root motd /etc/
ADD start_admin_sshd.sh /usr/sbin/
ADD ./sshd_config /etc/ssh/
ADD ./sheltie /usr/bin/

RUN chmod 440 /etc/sudoers.d/ec2-user
RUN chmod +x /usr/sbin/start_admin_sshd.sh
RUN chmod +x /usr/bin/sheltie
RUN groupadd -g 274 api
RUN useradd -m -G users,api ec2-user

CMD ["/usr/sbin/start_admin_sshd.sh"]
ENTRYPOINT ["/bin/bash", "-c"]
