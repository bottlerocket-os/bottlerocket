package main

import (
	"github.com/containerd/containerd/remotes/docker"
	"github.com/pelletier/go-toml"
	"github.com/pkg/errors"
	"io/ioutil"
	"net/url"
	"strings"
)

// Mirror contains the config related to the registry mirror
type Mirror struct {
	Endpoints []string
}

// RegistryConfig contains the config related to a image registry
type RegistryConfig struct {
	Mirrors map[string]Mirror
}

// NewRegistryConfig unmarshalls a registry configuration file and sets up a RegistryConfig
func NewRegistryConfig(registryConfigFile string) (*RegistryConfig, error) {
	raw, err := ioutil.ReadFile(registryConfigFile)
	if err != nil {
		return nil, err
	}

	config := RegistryConfig{}
	return &config, toml.Unmarshal(raw, &config)
}

// registryHosts returns the registry hosts to be used by the resolver.
// Heavily borrowed from containerd CRI plugin's implementation.
// See https://github.com/containerd/containerd/blob/1407cab509ff0d96baa4f0eb6ff9980270e6e620/pkg/cri/server/image_pull.go#L332-L405
// authorizerOverride lets the caller override the generated authorizer with a custom authorizer
// FIXME Replace this once there's a public containerd client interface that supports registry mirrors
func registryHosts(registryConfig *RegistryConfig, authorizerOverride *docker.Authorizer) docker.RegistryHosts {
	return func(host string) ([]docker.RegistryHost, error) {
		var (
			registries []docker.RegistryHost
			endpoints  []string
			authorizer docker.Authorizer
		)
		// Set up endpoints for the registry
		if _, ok := registryConfig.Mirrors[host]; ok {
			endpoints = registryConfig.Mirrors[host].Endpoints
		} else {
			endpoints = registryConfig.Mirrors["*"].Endpoints
		}
		defaultHost, err := docker.DefaultHost(host)
		if err != nil {
			return nil, errors.Wrap(err, "get default host")
		}
		endpoints = append(endpoints, defaultHost)

		for _, endpoint := range endpoints {
			// Prefix the endpoint with an appropriate URL scheme if the endpoint does not have one.
			if !strings.Contains(endpoint, "://") {
				if endpoint == "localhost" || endpoint == "127.0.0.1" || endpoint == "::1" {
					endpoint = "http://" + endpoint
				} else {
					endpoint = "https://" + endpoint
				}
			}
			url, err := url.Parse(endpoint)
			if err != nil {
				return nil, errors.Wrapf(err, "parse registry endpoint %q from mirrors", endpoint)
			}
			if url.Path == "" {
				url.Path = "/v2"
			}
			if authorizerOverride == nil {
				authorizer = docker.NewDockerAuthorizer()
			} else {
				authorizer = *authorizerOverride
			}
			registries = append(registries, docker.RegistryHost{
				Authorizer:   authorizer,
				Host:         url.Host,
				Scheme:       url.Scheme,
				Path:         url.Path,
				Capabilities: docker.HostCapabilityResolve | docker.HostCapabilityPull,
			})
		}
		return registries, nil
	}
}
