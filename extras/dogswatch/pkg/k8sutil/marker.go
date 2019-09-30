package k8sutil

import (
	"github.com/amazonlinux/thar/dogswatch/pkg/marker"
	"github.com/pkg/errors"
	v1meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	v1 "k8s.io/client-go/kubernetes/typed/core/v1"
)

func PostMetadata(nc v1.NodeInterface, nodeName string, cont marker.Container) error {
	node, err := nc.Get(nodeName, v1meta.GetOptions{})
	if err != nil {
		return errors.WithMessage(err, "unable to get node")
	}
	marker.OverwriteFrom(cont, node)
	// {
	// 	l := logging.New("k8sutil")
	// 	l.Debugf("annotations: %#v", node.GetAnnotations())
	// 	l.Debugf("labels: %#v", node.GetLabels())
	// }
	node, err = nc.Update(node)
	if err != nil {
		return errors.WithMessage(err, "unable to update node")
	}
	return nil
}
