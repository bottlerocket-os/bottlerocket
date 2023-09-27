mod source_path;

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about, long_about = None, version)]
struct Args {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Debug, Parser)]
enum SubCommand {
    /// Get the local source paths that feed into this variant.
    GetSourcePaths(source_path::SourcePathArgs),
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

// Execute the requested subcommand.
fn run(args: Args) -> source_path::Result<()> {
    match args.cmd {
        SubCommand::GetSourcePaths(getsourcepaths) => getsourcepaths.run(),
    }
}
