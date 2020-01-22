use bttr::Release;
use serde_json::json;
use snafu::ResultExt;
use std::collections::HashMap;
use structopt::StructOpt;

type Result<T> = std::result::Result<T, error::Error>;

#[derive(StructOpt, Debug)]
pub(crate) struct CLI {
    #[structopt(short = "c", long = "release-config", env = "BUILDSYS_RELEASE_CONFIG")]
    #[structopt(parse(try_from_str = Release::from_toml_file))]
    release: Release,

    #[structopt(subcommand)]
    dispatch: Dispatch,

    #[structopt(long = "json-output")]
    json: bool,
}

#[derive(StructOpt, Debug)]
pub(crate) enum Dispatch {
    Show(show::Show),
}

pub(crate) mod show {
    use structopt::StructOpt;
    
    #[derive(StructOpt, Debug)]
    pub enum Show {
	Build(BuildOp),
	Release(ReleaseOp),
    }

    #[derive(Debug, StructOpt)]
    pub struct BuildOp {
	#[structopt(subcommand)]
	pub show_build: Build,
	#[structopt(long = "arch", env = "ARCH")]
	pub arch: String,
	#[structopt(long = "variant", env = "VARIANT")]
	pub variant: String,
	#[structopt(long = "name-suffix", env = "IMAGE_NAME_SUFFIX")]
	pub suffix: Option<String>,
    }

    #[derive(StructOpt, Debug)]
    pub enum Build {
	ImageName,
	FilesystemImageFiles,
	DiskImageFiles,
    }


    #[derive(Debug, StructOpt)]
    pub struct ReleaseOp {
	#[structopt(subcommand)]
	pub show_release: Release,
    }

    #[derive(StructOpt, Debug)]
    pub enum Release {
	Version,
	Migrations,
    }
}

// better - barely-enough tooling to examine release
// bttr - buildsys toml to release
// bttr - big tangent to release.toml
// bttr - be that to release

pub fn main() -> Result<()> {
    let cli = CLI::from_args();

    match cli.dispatch {
	Dispatch::Show(show_command) => {
	    match show_command {
		show::Show::Release(show::ReleaseOp { show_release }) => {
		    match show_release {
			show::Release::Version => println!(
			    "{}",
			    match cli.json {
				true => json!({"version": cli.release.version}).to_string(),
				false => cli.release.version.to_string(),
			    }
			),
			show::Release::Migrations => {
			    let mgs = cli.release.migration_names();
			    let mg_crates = cli
				.release
				.migration_crates()
				.context(error::MigrationHandleError {})?;
			    let (by_crate, by_name): (HashMap<String, String>, HashMap<String, String>) = {
				let mut crate_indexed = HashMap::new();
				let mut name_indexed = HashMap::new();

				for (mname, cname) in mgs.iter().zip(mg_crates.iter()) {
				    crate_indexed.insert(cname.clone(), mname.clone());
				    name_indexed.insert(mname.clone(), cname.clone());
				}
				(crate_indexed, name_indexed)
			    };
			    match cli.json {
				true => println!(
				    "{}",
				    json!({"migration": { "names": mgs, "crates": mg_crates, "index": {"crate": by_crate, "name": by_name} }})
					.to_string()
				),
				false => {
				    for (name, crate_name) in mgs.iter().zip(mg_crates.iter()) {
					println!("{}\t{}", name, crate_name);
				    }
				}
			    }
			}
		    }
		},

		show::Show::Build(show::BuildOp{
		    show_build,
		    arch,
		    variant,
		    suffix,
		}) => {
		    let build = cli.release.as_build(arch, variant, suffix);
		    let image_name = build.image_name();

		    match show_build {
			show::Build::ImageName => {
			    println!(
				"{}",
				match cli.json {
				    true => json!({ "name": image_name }).to_string(),
				    false => image_name,
				}
			    );
			}

			show::Build::FilesystemImageFiles => {
			    let mut files = Vec::with_capacity(bttr::FILESYSTEM_FILE_SUFFICES.len());
			    for x in bttr::FILESYSTEM_FILE_SUFFICES.iter() {
				files.push(format!(
				    "{image_name}-{suffix}",
				    image_name = image_name,
				    suffix = x
				));
			    }
			    println!(
				"{}",
				match cli.json {
				    true => json!({ "files": files }).to_string(),
				    false => files.join("\n"),
				}
			    );
			}

			show::Build::DiskImageFiles => {
			    let mut files = Vec::with_capacity(bttr::DISK_IMAGE_FILE_SUFFICES.len());
			    for x in bttr::DISK_IMAGE_FILE_SUFFICES.iter() {
				files.push(format!(
				    "{image_name}-{suffix}",
				    image_name = image_name,
				    suffix = x
				));
			    }
			    println!(
				"{}",
				match cli.json {
				    true => json!({ "files": files }).to_string(),
				    false => files.join("\n"),
				}
			    );
			}
		    };
		}
	    }
	}
    }
    Ok(())
}
pub mod error {
    use snafu::Snafu;
    use std::path::PathBuf;

    #[derive(Snafu, Debug)]
    pub enum Error {
	#[snafu(visibility(pub))]
	#[snafu(display("could not parse config file {:?}: {}", file.as_os_str(), source))]
	ConfigParseError {
	    file: PathBuf,
	    source: toml::de::Error,
	},

	#[snafu(visibility(pub))]
	#[snafu(display("could not read config file {:?}: {}", file.as_os_str(), source))]
	ConfigReadError {
	    file: PathBuf,
	    source: std::io::Error,
	},

	#[snafu(visibility(pub))]
	#[snafu(display("error processing release migrations: {}", source))]
	MigrationHandleError { source: bttr::error::Error },
	
	#[snafu(visibility(pub))]
	#[snafu(display("command usage error: {}", message))]
	CommandUsageError { message: String },
    }
}
