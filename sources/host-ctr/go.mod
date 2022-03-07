module host-ctr

go 1.16

require (
	github.com/Microsoft/hcsshim v0.9.2 // indirect
	github.com/aws/aws-sdk-go v1.42.45
	github.com/awslabs/amazon-ecr-containerd-resolver v0.0.0-20211009021844-db7e6868925f
	github.com/containerd/cgroups v1.0.3 // indirect
	github.com/containerd/containerd v1.5.9
	github.com/containerd/continuity v0.2.2 // indirect
	github.com/cpuguy83/go-md2man/v2 v2.0.1 // indirect
	github.com/golang/groupcache v0.0.0-20210331224755-41bb18bfe9da // indirect
	github.com/klauspost/compress v1.14.2 // indirect
	github.com/opencontainers/runc v1.1.0 // indirect
	github.com/opencontainers/runtime-spec v1.0.3-0.20211214071223-8958f93039ab
	github.com/pelletier/go-toml v1.9.4
	github.com/pkg/errors v0.9.1
	github.com/sirupsen/logrus v1.8.1
	github.com/stretchr/testify v1.7.0
	github.com/urfave/cli/v2 v2.3.0
	go.opencensus.io v0.23.0 // indirect
	golang.org/x/net v0.0.0-20220127200216-cd36cc0744dd // indirect
	golang.org/x/sys v0.0.0-20220128215802-99c3d69c2c27 // indirect
	google.golang.org/genproto v0.0.0-20220202230416-2a053f022f0d // indirect
	google.golang.org/grpc v1.44.0 // indirect
	k8s.io/cri-api v0.20.6
)

replace github.com/Sirupsen/logrus => github.com/sirupsen/logrus v1.6.0
