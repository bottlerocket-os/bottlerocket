package main

import (
	"testing"

	"github.com/containerd/containerd/remotes/docker"
	"github.com/stretchr/testify/assert"
)

// Test RegistryHosts with valid endpoints URLs
func TestRegistryHosts(t *testing.T) {
	tests := []struct {
		name     string
		host     string
		config   RegistryConfig
		expected []docker.RegistryHost
	}{
		{
			"HTTP scheme",
			"docker.io",
			RegistryConfig{
				Mirrors: map[string]Mirror{
					"docker.io": {
						Endpoints: []string{"http://198.158.0.0"},
					},
				},
			},
			[]docker.RegistryHost{
				{
					Authorizer:   docker.NewDockerAuthorizer(),
					Host:         "198.158.0.0",
					Scheme:       "http",
					Path:         "/v2",
					Capabilities: docker.HostCapabilityResolve | docker.HostCapabilityPull,
				},
				{
					Authorizer:   docker.NewDockerAuthorizer(),
					Host:         "registry-1.docker.io",
					Scheme:       "https",
					Path:         "/v2",
					Capabilities: docker.HostCapabilityResolve | docker.HostCapabilityPull,
				},
			},
		},
		{
			"No scheme",
			"docker.io",
			RegistryConfig{
				Mirrors: map[string]Mirror{
					"docker.io": {
						Endpoints: []string{"localhost", "198.158.0.0", "127.0.0.1"},
					},
				},
			},
			[]docker.RegistryHost{
				{
					Authorizer:   docker.NewDockerAuthorizer(),
					Host:         "localhost",
					Scheme:       "http",
					Path:         "/v2",
					Capabilities: docker.HostCapabilityResolve | docker.HostCapabilityPull,
				},
				{
					Authorizer:   docker.NewDockerAuthorizer(),
					Host:         "198.158.0.0",
					Scheme:       "https",
					Path:         "/v2",
					Capabilities: docker.HostCapabilityResolve | docker.HostCapabilityPull,
				},
				{
					Authorizer:   docker.NewDockerAuthorizer(),
					Host:         "127.0.0.1",
					Scheme:       "http",
					Path:         "/v2",
					Capabilities: docker.HostCapabilityResolve | docker.HostCapabilityPull,
				},
				{
					Authorizer:   docker.NewDockerAuthorizer(),
					Host:         "registry-1.docker.io",
					Scheme:       "https",
					Path:         "/v2",
					Capabilities: docker.HostCapabilityResolve | docker.HostCapabilityPull,
				},
			},
		},
		{
			"* endpoints",
			"weird.io",
			RegistryConfig{
				Mirrors: map[string]Mirror{
					"docker.io": {
						Endpoints: []string{"notme", "certainly-not-me"},
					},
					"*": {
						Endpoints: []string{"198.158.0.0", "example.com"},
					},
				},
			},
			[]docker.RegistryHost{
				{
					Authorizer:   docker.NewDockerAuthorizer(),
					Host:         "198.158.0.0",
					Scheme:       "https",
					Path:         "/v2",
					Capabilities: docker.HostCapabilityResolve | docker.HostCapabilityPull,
				},
				{
					Authorizer:   docker.NewDockerAuthorizer(),
					Host:         "example.com",
					Scheme:       "https",
					Path:         "/v2",
					Capabilities: docker.HostCapabilityResolve | docker.HostCapabilityPull,
				},
				{
					Authorizer:   docker.NewDockerAuthorizer(),
					Host:         "weird.io",
					Scheme:       "https",
					Path:         "/v2",
					Capabilities: docker.HostCapabilityResolve | docker.HostCapabilityPull,
				},
			},
		},
		{
			"No mirrors",
			"docker.io",
			RegistryConfig{
				Mirrors: map[string]Mirror{},
			},
			[]docker.RegistryHost{
				{
					Authorizer:   docker.NewDockerAuthorizer(),
					Host:         "registry-1.docker.io",
					Scheme:       "https",
					Path:         "/v2",
					Capabilities: docker.HostCapabilityResolve | docker.HostCapabilityPull,
				},
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			f := registryHosts(&tc.config, nil)
			result, err := f(tc.host)
			assert.NoError(t, err)
			assert.Equal(t, tc.expected, result)
		})
	}
}

// Test RegistryHosts with an invalid endpoint URL
func TestBadRegistryHosts(t *testing.T) {
	f := registryHosts(&RegistryConfig{
		Mirrors: map[string]Mirror{
			"docker.io": {
				Endpoints: []string{"$#%#$$#%#$"},
			},
		},
	}, nil)
	_, err := f("docker.io")
	assert.Error(t, err)
}

func TestEcrParserSpecialRegion(t *testing.T) {
	test := struct {
		name           string
		ecrImgURI      string
		expectedString string
	}{

		"Parse ECR repo URL for special-case region",
		"111111111111.dkr.ecr.il-central-1.amazonaws.com/bottlerocket/container:1.2.3",
		"ecr.aws/arn:aws:ecr:il-central-1:111111111111:repository/bottlerocket/container:1.2.3",
	}

	t.Run(test.name, func(t *testing.T) {
		result, err := parseImageURISpecialRegions(test.ecrImgURI)
		assert.NoError(t, err)
		assert.Equal(t, test.expectedString, result)
	})
}

func TestEcrParserUnsupportedSpecialRegions(t *testing.T) {
	test := struct {
		name           string
		ecrImgURI      string
		expectedString string
	}{
		"Unsupported special region",
		"111111111111.dkr.ecr.outer-space.amazonaws.com/bottlerocket/container:1.2.3",
		"",
	}

	t.Run(test.name, func(t *testing.T) {
		_, err := parseImageURISpecialRegions(test.ecrImgURI)
		assert.Error(t, err)
	})
}
