package workgroup

import (
	"context"

	"golang.org/x/sync/errgroup"
)

type workgroup struct {
	ctx   context.Context
	group errgroup.Group
}

func WithContext(ctx context.Context) *workgroup {
	return &workgroup{
		ctx:   ctx,
		group: errgroup.Group{},
	}
}

func (g *workgroup) Work(fn func(context.Context) error) {
	g.group.Go(func() error {
		return fn(g.ctx)
	})
}

func (g *workgroup) Wait() error {
	return g.group.Wait()
}
