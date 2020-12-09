package main

import (
	"io"

	"github.com/sirupsen/logrus"
)

// LogSplitHook is expected to implement the correct logrus interface.
var _ logrus.Hook = (*LogSplitHook)(nil)

// LogSplitHook directs matched levels to its configured output.
type LogSplitHook struct {
	output io.Writer
	levels []logrus.Level
}

// Fire is invoked when logrus tries to log any message.
func (hook *LogSplitHook) Fire(entry *logrus.Entry) error {
	line, err := entry.String()
	if err != nil {
		return err
	}
	for _, level := range hook.levels {
		if level == entry.Level {
			_, err := hook.output.Write([]byte(line))
			return err
		}
	}
	return nil
}

// Levels returns the log levels this hook is being applied to
func (hook *LogSplitHook) Levels() []logrus.Level {
	return hook.levels
}
