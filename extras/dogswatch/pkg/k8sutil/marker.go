package k8sutil

import (
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/logging"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/marker"

	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
	v1meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	v1 "k8s.io/client-go/kubernetes/typed/core/v1"
)

func PostMetadata(nc v1.NodeInterface, nodeName string, cont marker.Container) error {
	node, err := nc.Get(nodeName, v1meta.GetOptions{})
	if err != nil {
		return errors.WithMessage(err, "unable to get node")
	}
	marker.OverwriteFrom(cont, node)
	if logging.Debuggable {
		l := logging.New("k8sutil")
		l.WithFields(logrus.Fields{
			"node":        nodeName,
			"annotations": node.GetAnnotations(),
			"labels":      node.GetLabels(),
		}).Debug("merged in new metadata")
	}
	_, err = nc.Update(node)
	if err != nil {
		return errors.WithMessage(err, "unable to update node")
	}
	return nil
}
