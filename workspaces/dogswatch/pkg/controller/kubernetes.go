package controller

import (
	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
	v1meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/kubectl/pkg/drain"
)

func (am *ActionManager) cordonNode(nodeName string) error {
	log := am.log.WithField("node", nodeName)
	log.Debug("preparing to cordon")
	node, err := am.kube.CoreV1().Nodes().Get(nodeName, v1meta.GetOptions{})
	if err != nil {
		return errors.WithMessage(err, "unable to retrieve node from api")
	}
	helper := drain.NewCordonHelper(node)
	if helper.UpdateIfRequired(true) {
		log.Debug("cordoning node")
		err, patchErr := helper.PatchOrReplace(am.kube)
		if err != nil {
			return errors.WithMessage(err, "unable to submit node patch")
		}
		if patchErr != nil {
			return errors.WithMessage(err, "unable to generate patch for node")
		}
		return nil
	} else {
		log.Debug("node is already cordoned")
	}
	return nil
}

func (am *ActionManager) drainWorkload(nodeName string) error {
	log := am.log.WithField("node", nodeName)
	log.Debug("draining workload")
	helper := drain.Helper{
		Client:              am.kube,
		Out:                 am.log.WriterLevel(logrus.InfoLevel),
		ErrOut:              am.log.WriterLevel(logrus.ErrorLevel),
		IgnoreAllDaemonSets: true,
		// TODO: implement a more considerate descheduler
		Force: true,
	}
	pods, errs := helper.GetPodsForDeletion(nodeName)
	if len(errs) != 0 {
		for _, e := range errs {
			log.Error(e)
		}
		return errors.New("errors encountered while listing pods for deletion")
	}
	var err error
	npods := len(pods.Pods())
	if npods > 0 {
		log.Debugf("%d pods present, removing workload", npods)
		err = helper.DeleteOrEvictPods(pods.Pods())
		if err == nil {
			log.Debug("workload drained successfully")
		}
	} else {
		log.Debug("no workload present")
	}
	return err
}

// func (am *ActionManager) permitIntentAction(nodeName string, in *intent.Intent) error {
// 	am.kube.CoreV1().Nodes().Patch(nodeName, types.JSONMerge, []byte(""))
// }
