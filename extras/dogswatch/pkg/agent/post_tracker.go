package agent

import (
	"container/list"
	"sync"

	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/intent"
)

// postTracker records posted Intents to compare to inbound Intents.
type postTracker struct {
	mu   *sync.RWMutex
	list *list.List
}

func newPostTracker() *postTracker {
	return &postTracker{
		mu:   &sync.RWMutex{},
		list: list.New()}
}

// recordPost retains a record of the posted Intent.
func (p *postTracker) recordPost(in *intent.Intent) {
	p.mu.Lock()
	p.list.PushBack(in.Clone())
	p.mu.Unlock()
}

// matchesPost checks for the presence of a matching tracked posted Intent.
func (p *postTracker) matchesPost(in *intent.Intent) bool {
	p.mu.RLock()
	defer p.mu.RUnlock()
	if p.list.Len() == 0 {
		return false
	}
	for elm := p.list.Front(); elm != nil; elm = elm.Next() {
		if intent.Equivalent(elm.Value.(*intent.Intent), in) {
			return true
		}
	}

	return false
}

// clear removes all tracked posted Intents.
func (p *postTracker) clear() {
	p.mu.Lock()
	p.list.Init()
	p.mu.Unlock()
}
