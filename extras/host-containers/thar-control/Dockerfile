# DOCKER_ARCH is the multiarch variant that is used as the base image.
ARG DOCKER_ARCH
FROM $DOCKER_ARCH/amazonlinux:2

# IMAGE_VERSION is the assigned version of inputs for this image.
ARG IMAGE_VERSION
ENV IMAGE_VERSION=$IMAGE_VERSION
# IMAGE_VERSION is the assigned version of inputs for this image.
ARG SSM_AGENT_VERSION
ENV SSM_AGENT_VERSION=$SSM_AGENT_VERSION
# ARCH is the normative target architecture for the image.
ARG ARCH
ENV ARCH=$ARCH

# Validation
RUN : \
    "${IMAGE_VERSION:?IMAGE_VERSION is required to build}" \
    "${ARCH:?ARCH is required to build}" \
    "${SSM_AGENT_VERSION:?SSM Agent version required to build}"

LABEL "org.opencontainers.image.version"="$IMAGE_VERSION"

# Install the arch specific build of SSM agent *and confirm that it installed* -
# yum will allow architecture-mismatched packages to not install and consider
# the run successful.
RUN yum install -y "https://s3.amazonaws.com/ec2-downloads-windows/SSMAgent/${SSM_AGENT_VERSION}/linux_${ARCH}/amazon-ssm-agent.rpm" \
    shadow-utils \
    && rm -rf /var/cache/yum

# Add motd explaining the control container.
RUN rm -f /etc/motd /etc/issue
ADD --chown=root:root motd /etc/
# Add bashrc that shows the motd.
ADD ./bashrc /etc/skel/.bashrc
# SSM starts sessions with 'sh', not 'bash', which for us is a link to bash.
# Furthermore, it starts sh as an interactive shell, but not a login shell.
# In this mode, the only startup file respected is the one pointed to by the
# ENV environment variable.  Point it to our bashrc, which just prints motd.
ENV ENV /etc/skel/.bashrc

# Add our helper to quickly enable the admin container.
ADD ./enable-admin-container /usr/bin/
RUN chmod +x /usr/bin/enable-admin-container

# Create our user in the group that allows API access.
RUN groupadd -g 274 api
RUN useradd -m -G users,api ssm-user

CMD ["/usr/bin/amazon-ssm-agent"]
