module host-ctr

go 1.12

require (
	github.com/aws/aws-sdk-go v1.28.9
	github.com/awslabs/amazon-ecr-containerd-resolver v0.0.0-20200131205711-bda55ee680cd
	github.com/containerd/containerd v1.2.9
	github.com/opencontainers/runc v1.0.0-rc8
	github.com/opencontainers/runtime-spec v1.0.1
	github.com/pkg/errors v0.0.0-20190227000051-27936f6d90f9
	github.com/sirupsen/logrus v1.4.2
	github.com/stretchr/testify v1.2.2
)

replace github.com/Sirupsen/logrus => github.com/sirupsen/logrus v1.4.2
