use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use delete::Delete;
use install::Install;
use logs::Logs;
use model::test_manager::TestManager;
use restart_test::RestartTest;
use run::Run;
use status::Status;
use std::path::PathBuf;

mod aws_resources;
mod delete;
mod install;
mod logs;
mod restart_test;
mod run;
mod status;

/// A program for running and controlling Bottlerocket tests in a Kubernetes cluster using
/// https://github.com/bottlerocket-os/bottlerocket-test-system
#[derive(Parser, Debug)]
#[clap(about, long_about = None)]
struct TestsysArgs {
    /// Path to the kubeconfig file for the testsys cluster. Can also be passed with the KUBECONFIG
    /// environment variable.
    #[clap(long)]
    kubeconfig: Option<PathBuf>,

    #[clap(subcommand)]
    command: Command,
}

impl TestsysArgs {
    async fn run(self) -> Result<()> {
        let client = match self.kubeconfig {
            Some(path) => TestManager::new_from_kubeconfig_path(&path)
                .await
                .context(format!(
                    "Unable to create testsys client using kubeconfig '{}'",
                    path.display()
                ))?,
            None => TestManager::new().await.context(
                "Unable to create testsys client using KUBECONFIG variable or default kubeconfig",
            )?,
        };
        match self.command {
            Command::Run(run) => run.run(client).await?,
            Command::Install(install) => install.run(client).await?,
            Command::Delete(delete) => delete.run(client).await?,
            Command::Status(status) => status.run(client).await?,
            Command::Logs(logs) => logs.run(client).await?,
            Command::RestartTest(restart_test) => restart_test.run(client).await?,
        };
        Ok(())
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    Install(Install),
    Run(Box<Run>),
    Delete(Delete),
    Status(Status),
    Logs(Logs),
    RestartTest(RestartTest),
}

#[tokio::main]
async fn main() {
    let args = TestsysArgs::parse();
    println!("{:?}", args);
    if let Err(e) = args.run().await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
