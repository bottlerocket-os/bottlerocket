module host-ctr

go 1.12

require (
	github.com/awslabs/amazon-ecr-containerd-resolver v0.0.0-20200702001206-7094584cd367
	github.com/containerd/cgroups v0.0.0-20200702150254-e9676da73edd // indirect
	github.com/containerd/containerd v1.3.6
	github.com/opencontainers/image-spec v1.0.1 // indirect
	github.com/opencontainers/runc v1.0.0-rc8
	github.com/opencontainers/runtime-spec v1.0.2
	github.com/opencontainers/selinux v1.5.2
	github.com/pkg/errors v0.9.1
	github.com/sirupsen/logrus v1.6.0
	go.etcd.io/bbolt v1.3.5 // indirect
	golang.org/x/sync v0.0.0-20200625203802-6e8e738ad208 // indirect
	golang.org/x/sys v0.0.0-20200625212154-ddb9806d33ae // indirect
	google.golang.org/genproto v0.0.0-20200702021140-07506425bd67 // indirect
	google.golang.org/grpc v1.30.0 // indirect
	google.golang.org/protobuf v1.25.0 // indirect
	gopkg.in/yaml.v2 v2.3.0 // indirect
)

replace github.com/Sirupsen/logrus => github.com/sirupsen/logrus v1.6.0
