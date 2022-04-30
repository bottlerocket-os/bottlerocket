use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// A program for running and controlling Bottlerocket tests through in a TestSys Kubernetes
/// cluster.
#[derive(Parser, Debug)]
#[clap(about, long_about = None)]
struct TestsysArgs {
    /// Path to the kubeconfig file. Can also be passed with the KUBECONFIG environment variable.
    #[clap(long)]
    kubeconfig: Option<PathBuf>,

    #[clap(subcommand)]
    command: Command,
}

impl TestsysArgs {
    fn run(self) -> std::result::Result<(), ()> {
        match self.command {
            Command::Run(run_command_shim) => match run_command_shim.command {
                RunCommand::Integ(integ_args) => integ_args.run(),
            },
        }
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    Run(RunCommandShim),
}

// This struct appears to be necessary: https://github.com/clap-rs/clap/issues/2005
#[derive(Args, Debug)]
struct RunCommandShim {
    #[clap(subcommand)]
    command: RunCommand,
}

/// Run a test or resource provider.
#[derive(Subcommand, Debug)]
enum RunCommand {
    Integ(IntegArgs),
}

#[derive(Args, Debug)]
struct IntegArgs {
    /// The build ID. This is typically the git commit short sha plus a `-dirty` suffix if there are
    /// uncommitted changes.
    #[clap(long, env = "BUILDSYS_VERSION_BUILD")]
    version_build: String,

    /// The Bottlerocket variant name.
    #[clap(long, env = "BUILDSYS_VARIANT")]
    variant: String,

    /// The Bottlerocket image's architecture..
    #[clap(long, env = "BUILDSYS_ARCH")]
    arch: String,

    /// The Bottlerocket image ID (for `aws` variants) or image filepath (e.v. OVA file). This will
    /// be derived from variant and arch if not provided.
    #[clap(long)]
    image_id: Option<String>,
}

impl IntegArgs {
    fn run(self) -> std::result::Result<(), ()> {
        println!("TODO - run the tests!");
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let args = TestsysArgs::parse();
    println!("{:?}", args);
    if let Err(_) = args.run() {
        std::process::exit(1);
    }
}
