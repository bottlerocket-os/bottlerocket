use bttr::Release;
use serde_json::json;
use snafu::ResultExt;
use std::collections::HashMap;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Cmd {
    #[structopt(short = "c", long = "release-config", env = "BUILDSYS_RELEASE_CONFIG")]
    #[structopt(parse(try_from_str = Release::from_toml_file))]
    release: Release,

    #[structopt(subcommand)]
    run: RunKind,

    #[structopt(long = "json-output")]
    json: bool,
}

#[derive(StructOpt, Debug)]
enum RunKind {
    ShowBuild {
        #[structopt(subcommand)]
        show: ShowBuild,

        #[structopt(long = "arch", env = "ARCH")]
        arch: String,
        #[structopt(long = "variant", env = "VARIANT")]
        variant: String,
        #[structopt(long = "name-suffix", env = "IMAGE_NAME_SUFFIX")]
        suffix: Option<String>,
    },

    ShowRelease {
        #[structopt(subcommand)]
        show: ShowRelease,
    },
}

#[derive(StructOpt, Debug)]
enum ShowRelease {
    Version,
    Migrations,
}

#[derive(StructOpt, Debug)]
enum ShowBuild {
    ImageName,
    FilesystemImageFiles,
    DiskImageFiles,
}

// better - barely-enough tooling to examine release
// bttr - buildsys toml to release
// bttr - big tangent to release.toml

pub fn main() -> Result<(), error::Error> {
    let cmd = Cmd::from_args();

    match cmd.run {
        RunKind::ShowRelease { show } => match show {
            ShowRelease::Version => println!(
                "{}",
                match cmd.json {
                    true => json!({"version": cmd.release.version}).to_string(),
                    false => cmd.release.version.to_string(),
                }
            ),
            ShowRelease::Migrations => {
                let mgs = cmd.release.migration_names();
                let mg_crates = cmd
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
                match cmd.json {
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
        },

        RunKind::ShowBuild {
            show,
            arch,
            variant,
            suffix,
        } => {
            let build = cmd.release.as_build(arch, variant, suffix);
            let image_name = build.image_name();

            match show {
                ShowBuild::ImageName => {
                    println!(
                        "{}",
                        match cmd.json {
                            true => json!({ "name": image_name }).to_string(),
                            false => image_name,
                        }
                    );
                }

                ShowBuild::FilesystemImageFiles => {
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
                        match cmd.json {
                            true => json!({ "files": files }).to_string(),
                            false => files.join("\n"),
                        }
                    );
                }

                ShowBuild::DiskImageFiles => {
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
                        match cmd.json {
                            true => json!({ "files": files }).to_string(),
                            false => files.join("\n"),
                        }
                    );
                }
            };
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
    }
}
