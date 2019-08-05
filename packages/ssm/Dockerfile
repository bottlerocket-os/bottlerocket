FROM amazonlinux:2

RUN mkdir -p /var/lib/amazon/ssm
RUN mkdir -p /etc/amazon/ssm

ADD etc/amazon/* /etc/amazon/ssm/
ADD usr/bin/amazon-ssm-agent /usr/bin/
ADD usr/bin/ssm-cli /usr/bin/
ADD usr/bin/ssm-document-worker /usr/bin/
ADD usr/bin/ssm-session-worker /usr/bin/
ADD usr/bin/ssm-session-logger /usr/bin/

RUN chmod 755 /usr/bin/amazon-ssm-agent
RUN chmod 755 /usr/bin/ssm-cli
RUN chmod 755 /usr/bin/ssm-document-worker
RUN chmod 755 /usr/bin/ssm-session-worker
RUN chmod 755 /usr/bin/ssm-session-logger
