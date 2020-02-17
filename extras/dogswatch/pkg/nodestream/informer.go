package nodestream

import (
	"context"

	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/logging"
	v1 "k8s.io/api/core/v1"
	"k8s.io/client-go/informers"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/tools/cache"
)

var _ cache.ResourceEventHandler = (*informerStream)(nil)

type informerStream struct {
	log logging.Logger

	informer cache.SharedIndexInformer
	handler  Handler
}

func New(log logging.Logger, kube kubernetes.Interface, config Config, handler Handler) *informerStream {
	is := &informerStream{log: log, handler: handler}

	factory := informers.NewSharedInformerFactoryWithOptions(kube, config.resyncPeriod(), informers.WithTweakListOptions(config.selector()))
	informer := factory.Core().V1().Nodes().Informer()
	informer.AddEventHandler(is)

	is.informer = informer

	return is
}

func (is *informerStream) GetInformer() cache.SharedIndexInformer {
	return is.informer
}

func (is *informerStream) Run(ctx context.Context) error {
	is.log.Debug("starting")
	defer is.log.Debug("finished")
	is.informer.Run(ctx.Done())
	return nil
}

func (is *informerStream) OnAdd(obj interface{}) {
	is.log.Debug("resource add event")
	is.handler.OnAdd(obj.(*v1.Node))
}

func (is *informerStream) OnDelete(obj interface{}) {
	is.log.Debug("resource delete event")
	is.handler.OnDelete(obj.(*v1.Node))
}

func (is *informerStream) OnUpdate(oldObj, newObj interface{}) {
	is.log.Debug("resource update event")
	is.handler.OnUpdate(oldObj.(*v1.Node), newObj.(*v1.Node))
}
