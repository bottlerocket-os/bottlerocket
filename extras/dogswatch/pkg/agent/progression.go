package agent

import "github.com/amazonlinux/bottlerocket/dogswatch/pkg/platform"

type progression struct {
	target platform.Update
}

func (p *progression) SetTarget(t platform.Update) {
	p.target = t
}

func (p *progression) GetTarget() platform.Update {
	return p.target
}

func (p *progression) Reset() {
	p.target = nil
	return
}

func (p *progression) Valid() bool {
	return p.target != nil
}
