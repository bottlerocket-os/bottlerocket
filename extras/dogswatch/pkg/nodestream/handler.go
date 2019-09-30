package nodestream

import v1 "k8s.io/api/core/v1"

type Handler interface {
	OnAdd(*v1.Node)
	OnUpdate(*v1.Node, *v1.Node)
	OnDelete(*v1.Node)
}

type HandlerFuncs struct {
	OnAddFunc    func(*v1.Node)
	OnUpdateFunc func(*v1.Node, *v1.Node)
	OnDeleteFunc func(*v1.Node)
}

func (fn *HandlerFuncs) OnAdd(n *v1.Node) {
	if fn.OnAddFunc != nil {
		fn.OnAddFunc(n)
	}
}
func (fn *HandlerFuncs) OnUpdate(nOld *v1.Node, nNew *v1.Node) {
	if fn.OnUpdateFunc != nil {
		fn.OnUpdateFunc(nOld, nNew)
	}
}
func (fn *HandlerFuncs) OnDelete(n *v1.Node) {
	if fn.OnDeleteFunc != nil {
		fn.OnDeleteFunc(n)
	}
}
