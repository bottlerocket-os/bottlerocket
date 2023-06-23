use clap::{Parser, Subcommand};
use delete::Delete;
use env_logger::Builder;
use error::Result;
use install::Install;
use log::{debug, error, LevelFilter};
use logs::Logs;
use restart_test::RestartTest;
use run::Run;
use secret::Add;
use status::Status;
use std::path::PathBuf;
use testsys_model::test_manager::TestManager;
use uninstall::Uninstall;

mod aws_ecs;
mod aws_k8s;
mod aws_resources;
mod crds;
mod delete;
mod error;
mod install;
mod logs;
mod metal_k8s;
mod migration;
mod restart_test;
mod run;
mod secret;
mod sonobuoy;
mod status;
mod uninstall;
mod vmware_k8s;

/// A program for running and controlling Bottlerocket tests in a Kubernetes cluster using
/// bottlerocket-test-system
#[derive(Parser, Debug)]
#[clap(about, long_about = None)]
struct TestsysArgs {
    #[arg(global = true, long, default_value = "INFO")]
    /// How much detail to log; from least to most: ERROR, WARN, INFO, DEBUG, TRACE
    log_level: LevelFilter,

    /// Path to the kubeconfig file for the testsys cluster. Can also be passed with the KUBECONFIG
    /// environment variable.
    #[arg(long)]
    kubeconfig: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

impl TestsysArgs {
    async fn run(self) -> Result<()> {
        let client = match self.kubeconfig {
            Some(path) => TestManager::new_from_kubeconfig_path(&path).await?,
            None => TestManager::new().await?,
        };
        match self.command {
            Command::Run(run) => run.run(client).await?,
            Command::Install(install) => install.run(client).await?,
            Command::Delete(delete) => delete.run(client).await?,
            Command::Status(status) => status.run(client).await?,
            Command::Logs(logs) => logs.run(client).await?,
            Command::RestartTest(restart_test) => restart_test.run(client).await?,
            Command::Add(add) => add.run(client).await?,
            Command::Uninstall(uninstall) => uninstall.run(client).await?,
        };
        Ok(())
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    // We need to box some commands because they require significantly more arguments than the other commands.
    Install(Box<Install>),
    Run(Box<Run>),
    Delete(Delete),
    Status(Status),
    Logs(Logs),
    RestartTest(RestartTest),
    Add(Add),
    Uninstall(Uninstall),
}

#[tokio::main]
async fn main() {
    let args = TestsysArgs::parse();
    init_logger(args.log_level);
    debug!("{:?}", args);
    if let Err(e) = args.run().await {
        error!("{}", e);
        std::process::exit(1);
    }
}

/// Initialize the logger with the value passed by `--log-level` (or its default) when the
/// `RUST_LOG` environment variable is not present. If present, the `RUST_LOG` environment variable
/// overrides `--log-level`/`level`.
fn init_logger(level: LevelFilter) {
    match std::env::var(env_logger::DEFAULT_FILTER_ENV).ok() {
        Some(_) => {
            // RUST_LOG exists; env_logger will use it.
            Builder::from_default_env().init();
        }
        None => {
            // RUST_LOG does not exist; use default log level for this crate only.
            Builder::new()
                .filter(Some(env!("CARGO_CRATE_NAME")), level)
                .init();
        }
    }
}
