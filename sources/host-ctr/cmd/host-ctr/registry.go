package main

import (
	"net"
	"net/http"
	"net/url"
	"os"
	"strings"
	"time"

	"github.com/containerd/containerd/pkg/cri/server"
	"github.com/containerd/containerd/remotes/docker"
	"github.com/pelletier/go-toml"
	"github.com/pkg/errors"
	runtime "k8s.io/cri-api/pkg/apis/runtime/v1"
)

// Mirror contains the config related to the registry mirror
type Mirror struct {
	Endpoints []string `toml:"endpoints,omitempty"`
}

// Credential contains a registry credential
type Credential struct {
	Username      string `toml:"username,omitempty"`
	Password      string `toml:"password,omitempty"`
	Auth          string `toml:"auth,omitempty"`
	IdentityToken string `toml:"identitytoken,omitempty"`
}

// RegistryConfig contains the config related to a image registry
type RegistryConfig struct {
	Mirrors     map[string]Mirror     `toml:"mirrors,omitempty"`
	Credentials map[string]Credential `toml:"creds,omitempty"`
}

// NewRegistryConfig unmarshalls a registry configuration file and sets up a RegistryConfig
func NewRegistryConfig(registryConfigFile string) (*RegistryConfig, error) {
	raw, err := os.ReadFile(registryConfigFile)
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
			authConfig runtime.AuthConfig
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
			var authorizer docker.Authorizer
			if authorizerOverride == nil {
				// Set up auth for pulling from registry
				var authOpts []docker.AuthorizerOpt
				if _, ok := registryConfig.Credentials[defaultHost]; ok {
					// Convert registry credentials config to runtime auth config, so it can be parsed by `ParseAuth`
					authConfig.Username = registryConfig.Credentials[defaultHost].Username
					authConfig.Password = registryConfig.Credentials[defaultHost].Password
					authConfig.Auth = registryConfig.Credentials[defaultHost].Auth
					authConfig.IdentityToken = registryConfig.Credentials[defaultHost].IdentityToken
					authOpts = append(authOpts, docker.WithAuthClient(&http.Client{
						Transport: newTransport(),
					}))
					authOpts = append(authOpts, docker.WithAuthCreds(func(host string) (string, string, error) {
						return server.ParseAuth(&authConfig, host)
					}))
				}
				authorizer = docker.NewDockerAuthorizer(authOpts...)
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

// newTransport is borrowed from containerd CRI plugin
// See https://github.com/containerd/containerd/blob/1407cab509ff0d96baa4f0eb6ff9980270e6e620/pkg/cri/server/image_pull.go#L466-L481
// FIXME Replace this once containerd creates a library that shares this code with ctr
func newTransport() *http.Transport {
	return &http.Transport{
		Proxy: http.ProxyFromEnvironment,
		DialContext: (&net.Dialer{
			Timeout:       30 * time.Second,
			KeepAlive:     30 * time.Second,
			FallbackDelay: 300 * time.Millisecond,
		}).DialContext,
		MaxIdleConns:          10,
		IdleConnTimeout:       30 * time.Second,
		TLSHandshakeTimeout:   10 * time.Second,
		ExpectContinueTimeout: 5 * time.Second,
	}
}
