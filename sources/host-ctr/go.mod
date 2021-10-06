module host-ctr

go 1.12

require (
	github.com/aws/aws-sdk-go v1.40.56
	github.com/awslabs/amazon-ecr-containerd-resolver v0.0.0-20210811170403-63c50e4c3911
	github.com/containerd/containerd v1.5.7
	github.com/opencontainers/runtime-spec v1.0.3-0.20210326190908-1c3f411f0417
	github.com/opencontainers/selinux v1.8.5 // indirect
	github.com/pelletier/go-toml v1.9.4
	github.com/pkg/errors v0.9.1
	github.com/sirupsen/logrus v1.8.1
	github.com/stretchr/testify v1.7.0
	github.com/urfave/cli/v2 v2.3.0
)

replace github.com/Sirupsen/logrus => github.com/sirupsen/logrus v1.6.0
