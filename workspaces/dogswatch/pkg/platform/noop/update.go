package noop

type Update struct{}

func (u *Update) Identifier() interface{} {
	return "noop"
}
