module host-ctr

go 1.12

require (
	github.com/aws/aws-sdk-go v1.28.9
	github.com/awslabs/amazon-ecr-containerd-resolver v0.0.0-20200131205711-bda55ee680cd
	github.com/containerd/containerd v1.3.4
	github.com/containerd/ttrpc v1.0.1 // indirect
	github.com/google/go-cmp v0.3.0 // indirect
	github.com/imdario/mergo v0.3.9 // indirect
	github.com/opencontainers/runc v1.0.0-rc8
	github.com/opencontainers/runtime-spec v1.0.1
	github.com/opencontainers/selinux v1.5.2
	github.com/pkg/errors v0.9.1
	github.com/sirupsen/logrus v1.4.2
	github.com/stretchr/testify v1.2.2
	go.etcd.io/bbolt v1.3.4 // indirect
	gopkg.in/yaml.v2 v2.3.0 // indirect
)

replace github.com/Sirupsen/logrus => github.com/sirupsen/logrus v1.4.2
