package k8sutil

import (
	"github.com/pkg/errors"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/rest"
	"k8s.io/client-go/tools/clientcmd"
)

// NewDefaultConfig loads kubeconfig from the environment based on the default
// SDK behavior - that is, this respects `$KUBECONFIG` and would load service
// access tokens if available.
func NewDefaultConfig() (*rest.Config, error) {
	// Load with SDK defaults.
	loadrules := clientcmd.NewDefaultClientConfigLoadingRules()
	overrides := clientcmd.ConfigOverrides{}
	configLoader := clientcmd.
		NewNonInteractiveDeferredLoadingClientConfig(loadrules, &overrides)
	config, loadErr := configLoader.ClientConfig()
	if loadErr != nil {
		return nil, errors.Wrap(loadErr, "could not load kubeconfig with default loader")
	}
	return config, nil
}

func DefaultKubernetesClient() (*kubernetes.Clientset, error) {
	config, configErr := NewDefaultConfig()
	if configErr != nil {
		return nil, configErr
	}

	return kubernetes.NewForConfig(config)
}
