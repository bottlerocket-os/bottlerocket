package testoutput

import (
	"io"
	"os"
	"testing"

	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/logging"
	"github.com/sirupsen/logrus"
)

// New returns a writer that writes strings (assuming lines) to the testing
// logger.
func New(t testing.TB) io.Writer {
	return &testoutput{t}
}

// Logger wraps a logger at the call point to collect its downstream calls.
func Logger(t testing.TB, logger logging.Logger) logging.Logger {
	l := logger.WithFields(logrus.Fields{})
	l.Logger.SetOutput(New(t))
	l.Logger.SetLevel(logrus.DebugLevel)
	return l
}

// Setter may be given to logging to configure the output to be sent to the
// testing facade to be interlaced with test output. You should not use parallel
// tests with this set as they would conflict in that they'd write to the wrong
// test or write to the Revert'd output if they aren't synchronous.
func Setter(t testing.TB) func(*logrus.Logger) error {
	return func(l *logrus.Logger) error {
		l.SetOutput(New(t))
		l.SetLevel(logrus.DebugLevel)
		return nil
	}
}

// Revert restores the logger output to write to stderr.
func Revert() func(*logrus.Logger) error {
	return func(l *logrus.Logger) error {
		l.SetOutput(os.Stderr)
		return nil
	}
}

type testoutput struct {
	t testing.TB
}

func (l *testoutput) Write(p []byte) (n int, err error) {
	l.t.Logf("%s", p)
	return len(p), nil
}
