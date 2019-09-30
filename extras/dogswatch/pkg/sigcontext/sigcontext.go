package sigcontext

import (
	"context"
	"os"
	"os/signal"
	"sync"
)

// WithSignalCancel is a context that will cancel itself when a signal is sent
// to the process. The cancel function returned is responsible for freeing the
// signal handlers used and must be called. If a caller wants to default the
// signal handlers to the go runtime then the cancel must be called as soon as
// the derived context is Done() (ie: a second ^C, SIGINT, will cause the
// process to terminate).
func WithSignalCancel(ctx context.Context, sigs ...os.Signal) (context.Context, context.CancelFunc) {
	sigctx, ctxcancel := context.WithCancel(ctx)

	sigchan := make(chan os.Signal, 1)
	signal.Notify(sigchan, sigs...)

	var once sync.Once
	cancel := func() {
		ctxcancel()
		once.Do(func() {
			signal.Stop(sigchan)
			close(sigchan)
		})
	}

	// Select on the signals coming in. The caller is required to call their
	// provided cancel function to release the signal channel and notificant.
	go func() {
		for {
			select {
			case <-sigctx.Done():
				ctxcancel()
				return
			case _, ok := <-sigchan:
				if !ok {
					continue
				}
				ctxcancel()
			}
		}
	}()

	return sigctx, cancel
}
