package logging

import (
	"sync"

	"github.com/sirupsen/logrus"
)

type Setter func(*logrus.Logger) error

var root = struct {
	logger *logrus.Logger
	mutex  *sync.Mutex
}{
	logger: logrus.New(),
	mutex:  &sync.Mutex{},
}

type Logger interface {
	logrus.FieldLogger
}

func New(component string, setters ...Setter) Logger {
	for _, setter := range setters {
		// no errors handling for now
		_ = Set(setter)
	}
	return root.logger.WithField("component", component)
}

func Set(setter Setter) error {
	root.mutex.Lock()
	err := setter(root.logger)
	root.mutex.Unlock()
	return err
}

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
