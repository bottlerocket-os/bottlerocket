package main

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

// Test ecrImageNameToRef with a valid ECR image URI
func TestECRImageNameToRefValid(t *testing.T) {
	tests := []struct {
		name      string
		imageName string
		expected  string
	}{
		{"Standard", "777777777777.dkr.ecr.us-west-2.amazonaws.com/my_image:latest", "ecr.aws/arn:aws:ecr:us-west-2:777777777777:repository/my_image:latest"},
		{"Standard: Digests", "777777777777.dkr.ecr.us-west-2.amazonaws.com/my_image@sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", "ecr.aws/arn:aws:ecr:us-west-2:777777777777:repository/my_image@sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"},
		{"AWS CN partition", "777777777777.dkr.ecr.cn-north-1.amazonaws.com.cn/my_image:latest", "ecr.aws/arn:aws-cn:ecr:cn-north-1:777777777777:repository/my_image:latest"},
		{"AWS Gov Cloud West", "777777777777.dkr.ecr.us-gov-west-1.amazonaws.com/my_image:latest", "ecr.aws/arn:aws-us-gov:ecr:us-gov-west-1:777777777777:repository/my_image:latest"},
		{"AWS Gov Cloud East", "777777777777.dkr.ecr.us-gov-east-1.amazonaws.com/my_image:latest", "ecr.aws/arn:aws-us-gov:ecr:us-gov-east-1:777777777777:repository/my_image:latest"},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			result, err := ecrImageNameToRef(tc.imageName)
			assert.NoError(t, err, "failed to convert image name into ref")
			assert.Equal(t, tc.expected, result)
		})
	}
}

func TestECRImageNameToRefInvalid(t *testing.T) {
	tests := []struct {
		name      string
		imageName string
	}{
		{"empty", ""},
		{"missing name or tag/digest", "777777777777.dkr.ecr.us-west-2.amazonaws.com/"},
		{"empty name and tag/digest", "777777777777.dkr.ecr.us-west-2.amazonaws.com/:"},
		{"no account", "dkr.ecr.us-west-2.amazonaws.com"},
		{"no region", "777777777777.dkr.ecr.amazonaws.com/"},
		{"not an ecr image", "docker.io/library/hello-world"},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			_, err := ecrImageNameToRef(tc.imageName)
			assert.Error(t, err)
		})
	}
}
