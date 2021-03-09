module host-ctr

go 1.12

require (
	github.com/aws/aws-sdk-go v1.37.0
	github.com/awslabs/amazon-ecr-containerd-resolver v0.0.0-20200922205237-bbd7175f7bd0
	github.com/containerd/containerd v1.4.3
	github.com/google/uuid v1.2.0 // indirect
	github.com/opencontainers/runtime-spec v1.0.3-0.20200929063507-e6143ca7d51d
	github.com/opencontainers/selinux v1.8.0
	github.com/pkg/errors v0.9.1
	github.com/sirupsen/logrus v1.7.0
	github.com/urfave/cli/v2 v2.3.0
)

replace github.com/Sirupsen/logrus => github.com/sirupsen/logrus v1.6.0
