package logging

import (
	"io"
	"sync"

	"github.com/sirupsen/logrus"
)

const (
	SubComponentField = "subcomponent"
	ComponentField    = "component"
)

type Setter func(*logrus.Logger) error

var root = struct {
	logger *logrus.Logger
	mutex  *sync.Mutex
}{
	logger: func() *logrus.Logger {
		l := logrus.New()

		l.SetFormatter(&logrus.TextFormatter{
			FullTimestamp: true,
		})

		return l
	}(),
	mutex: &sync.Mutex{},
}

// Logger is a top level component logging facade.
type Logger interface {
	logrus.FieldLogger

	Writer() *io.PipeWriter
	WriterLevel(logrus.Level) *io.PipeWriter
}

// SubLogger is a sub component logging facade for subsystems that do not have
// their own top level logging owner.
type SubLogger interface {
	logrus.FieldLogger
}

// New creates a top level Logger for a given component.
func New(component string, setters ...Setter) Logger {
	for _, setter := range setters {
		// no errors handling for now
		_ = Set(setter)
	}
	return root.logger.WithField(ComponentField, component)
}

// Set mutates the underlying root logging implementation.
func Set(setter Setter) error {
	root.mutex.Lock()
	err := setter(root.logger)
	root.mutex.Unlock()
	return err
}

// Level sets the level of the root logger.
func Level(lvl string) Setter {
	l, err := logrus.ParseLevel(lvl)
	if err != nil {
		root.logger.WithError(err).Errorf("unable to parse provided level %q", lvl)
		l = logrus.DebugLevel
	}
	return func(r *logrus.Logger) error {
		r.SetLevel(l)
		return nil
	}
}
