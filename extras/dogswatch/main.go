package main

import (
	"context"
	"flag"
	"os"
	"syscall"
	"time"

	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/agent"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/bottlerocket"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/controller"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/k8sutil"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/logging"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/platform/updog"
	"github.com/amazonlinux/bottlerocket/dogswatch/pkg/sigcontext"
	"github.com/pkg/errors"
	"k8s.io/client-go/kubernetes"
)

var (
	flagAgent          = flag.Bool("agent", false, "Run agent component")
	flagController     = flag.Bool("controller", false, "Run controller component")
	flagSkipMitigation = flag.Bool("skip-mitigation", false, "Skip applying mitigations")
	flagLogDebug       = flag.Bool("debug", false, "")
	flagNodeName       = flag.String("nodeName", "", "nodeName of the Node that this process is running on")
)

func main() {
	flag.Parse()

	if *flagLogDebug {
		logging.Set(logging.Level("debug"))
	}

	log := logging.New("main")

	// "debuggable" builds at runtime produce extensive logging output compared
	// to release builds with the debug flag enabled. This requires building and
	// using a distinct build in the deployment in order to use.
	if logging.Debuggable {
		log.Info("low-level logging.Debuggable is enabled in this build")
		log.Warn("logging.Debuggable produces large volumes of logs")
		delay := 3 * time.Second
		log.WithField("delay", delay).Warn("delaying start due to logging.Debuggable build")
		time.Sleep(delay)
		log.Info("starting logging.Debuggable enabled build")
	}

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
		log.Error("no component specified to run, provide either -agent or -controller")
		flag.Usage()
		os.Exit(1)
	case *flagController:
		err = runController(ctx, kube, *flagNodeName)
		if err != nil {
			log.WithError(err).Fatalf("controller stopped")
		}
	case *flagAgent:
		if !*flagSkipMitigation {
			log.Info("checking for necessary mitigations")
			err := bottlerocket.ApplyMitigations()
			if err != nil {
				log.WithError(err).Fatalf("unable to perform mitigations")
			}
		}
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
