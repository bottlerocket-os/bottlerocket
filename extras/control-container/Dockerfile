FROM amazonlinux:2

RUN yum install -y https://s3.amazonaws.com/ec2-downloads-windows/SSMAgent/latest/linux_amd64/amazon-ssm-agent.rpm shadow-utils

RUN groupadd -g 274 api
RUN useradd -m -G users,api ssm-user

CMD ["/usr/bin/amazon-ssm-agent"]
