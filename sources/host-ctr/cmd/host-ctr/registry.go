package main

import (
	"github.com/pelletier/go-toml"
	"io/ioutil"
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
