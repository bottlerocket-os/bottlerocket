package nodestream

import (
	"context"

	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/logging"
	v1 "k8s.io/api/core/v1"
	"k8s.io/client-go/informers"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/tools/cache"
	"k8s.io/client-go/util/workqueue"
)

var _ cache.ResourceEventHandler = (*informerStream)(nil)

type informerStream struct {
	log logging.Logger

	informer cache.SharedIndexInformer
	handler  Handler

	// TODO: determine if we need to be using a queue. I think we'll be using
	// other synchronization mechanisms elsewhere that may avoid the need.
	workqueue workqueue.RateLimitingInterface
}

func New(log logging.Logger, kube kubernetes.Interface, config Config, handler Handler) *informerStream {
	is := &informerStream{log: log, handler: handler}

	factory := informers.NewSharedInformerFactoryWithOptions(kube, config.resyncPeriod(), informers.WithTweakListOptions(config.selector()))
	informer := factory.Core().V1().Nodes().Informer()
	informer.AddEventHandler(is)

	is.informer = informer
	is.workqueue = workqueue.NewRateLimitingQueue(workqueue.DefaultControllerRateLimiter())

	return is
}

func (is *informerStream) GetInformer() cache.SharedIndexInformer {
	return is.informer
}

func (is *informerStream) Run(ctx context.Context) error {
	is.log.Debug("starting")
	defer is.log.Debug("finished")
	go is.shutdownWithContext(ctx)
	is.informer.Run(ctx.Done())
	return nil
}

func (is *informerStream) shutdownWithContext(ctx context.Context) {
	select {
	case <-ctx.Done():
		is.shutdown()
	}
}

func (is *informerStream) shutdown() {
	is.log.Debug("shutting down")
	defer is.log.Debug("shutdown")
	// Insert an event to unblock de-queue-ing process and shutdown the
	// queue. This causes the worker to exit itself because it *must* be
	// listening on the same context and does not latch on to the queue
	// again (otherwise, this would be a race condition).
	is.workqueue.Add(nil)
	is.workqueue.ShutDown()
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
