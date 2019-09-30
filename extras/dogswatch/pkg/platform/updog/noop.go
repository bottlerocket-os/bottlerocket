package updog

type NoopUpdate struct{}

func (n *NoopUpdate) Identifier() interface{} {
	return UpdateID("noop")
}
