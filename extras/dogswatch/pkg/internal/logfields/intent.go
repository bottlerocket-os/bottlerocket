package logfields

import (
	"github.com/amazonlinux/thar/dogswatch/pkg/intent"

	"github.com/sirupsen/logrus"
)

func Intent(i *intent.Intent) logrus.Fields {
	return logrus.Fields{
		"node":   i.GetName(),
		"intent": i.DisplayString(),
	}
}
