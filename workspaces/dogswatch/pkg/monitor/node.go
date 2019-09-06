package monitor

type Node interface {
	Post(state State)
}
