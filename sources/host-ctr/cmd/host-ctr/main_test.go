package main

import (
	"github.com/containerd/containerd/remotes/docker"
	"github.com/stretchr/testify/assert"
	"testing"
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
			f := registryHosts(&tc.config, docker.NewDockerAuthorizer())
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
	}, docker.NewDockerAuthorizer())
	_, err := f("docker.io")
	assert.Error(t, err)
}
