package main

import (
	"context"
	"flag"
	"os"
	"syscall"

	"github.com/amazonlinux/thar/dogswatch/pkg/agent"
	"github.com/amazonlinux/thar/dogswatch/pkg/controller"
	"github.com/amazonlinux/thar/dogswatch/pkg/k8sutil"
	"github.com/amazonlinux/thar/dogswatch/pkg/logging"
	"github.com/amazonlinux/thar/dogswatch/pkg/platform/updog"
	"github.com/amazonlinux/thar/dogswatch/pkg/sigcontext"
	"github.com/pkg/errors"
	"k8s.io/client-go/kubernetes"
)

var (
	flagAgent      = flag.Bool("agent", false, "Run agent component")
	flagController = flag.Bool("controller", false, "Run controller component")
	flagLogDebug   = flag.Bool("debug", false, "")
	flagNodeName   = flag.String("nodeName", "", "nodeName of the Node that this process is running on")
)

func main() {
	flag.Parse()

	if *flagLogDebug {
		logging.Set(logging.Level("debug"))
	}

	log := logging.New("main")

	kube, err := k8sutil.DefaultKubernetesClient()
	if err != nil {
		log.WithError(err).Fatalf("kubernetes client")
	}

	ctx, cancel := sigcontext.WithSignalCancel(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer cancel()

	switch {
	case *flagNodeName == "":
		log.Errorf("nodeName to operate under must be provided")
		os.Exit(1)
	case *flagController && *flagAgent:
		log.Error("cannot run both agent and controller")
		os.Exit(1)
	case (!*flagController && !*flagAgent):
		log.Error("no component specified to run, provide one")
		flag.Usage()
		os.Exit(1)
	case *flagController:
		err = runController(ctx, kube, *flagNodeName)
		if err != nil {
			log.WithError(err).Fatalf("controller stopped")
		}
	case *flagAgent:
		err = runAgent(ctx, kube, *flagNodeName)
		if err != nil {
			log.WithError(err).Fatalf("agent stopped")
		}
	}
	log.Info("bark bark! 🐕")
}

func runController(ctx context.Context, kube kubernetes.Interface, nodeName string) error {
	log := logging.New("controller")
	c, err := controller.New(log, kube, nodeName)
	if err != nil {
		return errors.WithMessage(err, "initialization error")
	}
	return errors.WithMessage(c.Run(ctx), "run error")
}

func runAgent(ctx context.Context, kube kubernetes.Interface, nodeName string) error {
	log := logging.New("agent")
	platform, err := updog.New()
	if err != nil {
		return errors.WithMessage(err, "could not setup platform for agent")
	}
	a, err := agent.New(log, kube, platform, nodeName)
	if err != nil {
		return err
	}

	return errors.WithMessage(a.Run(ctx), "run error")
}
