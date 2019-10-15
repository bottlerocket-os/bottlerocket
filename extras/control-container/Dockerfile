FROM amazonlinux:2

RUN yum install -y https://s3.amazonaws.com/ec2-downloads-windows/SSMAgent/latest/linux_amd64/amazon-ssm-agent.rpm shadow-utils

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
