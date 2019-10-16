use crate::error::{self, Result};
use snafu::{OptionExt, ResultExt};
use std::fs::{File, OpenOptions};
use std::io::{self};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use tempdir::TempDir;
use tough::{Limits, Repository, Settings};
use url::Url;

#[derive(Debug, StructOpt)]
pub(crate) struct DownloadArgs {
    /// Path to root.json file for the repository
    #[structopt(short = "r", long = "root")]
    root: Option<PathBuf>,

    /// Remote root.json version number
    #[structopt(short = "v", long = "root-version")]
    root_version: Option<NonZeroU64>,

    /// TUF repository metadata base URL
    #[structopt(short = "m", long = "metadata-url")]
    metadata_base_url: String,

    /// TUF repository target base URL
    #[structopt(short = "t", long = "target-url")]
    target_base_url: String,

    /// Allow downloading the root.json file (unsafe)
    #[structopt(long)]
    allow_root_download: bool,

    /// Output directory of targets
    indir: PathBuf,
}

fn root_warning<P: AsRef<Path>>(path: P) {
    #[rustfmt::skip]
    eprintln!("\
=================================================================
WARNING: Downloading root.json to {}
This is unsafe and will not establish trust, use only for testing
=================================================================",
    path.as_ref().display());
}

impl DownloadArgs {
    pub(crate) fn run(&self) -> Result<()> {
        // use local root.json or download from repository
        let root_path = if let Some(path) = &self.root {
            PathBuf::from(path)
        } else if self.allow_root_download {
            let name = if let Some(version) = self.root_version {
                format!("{}.root.json", version)
            } else {
                String::from("1.root.json")
            };
            let path = std::env::current_dir()
                .context(error::CurrentDir)?
                .join(&name);
            let url = Url::parse(&self.metadata_base_url)
                .context(error::UrlParse {
                    url: &self.metadata_base_url,
                })?
                .join(&name)
                .context(error::UrlParse {
                    url: &self.metadata_base_url,
                })?;

            root_warning(&path);

            let mut f = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&path)
                .context(error::OpenFile { path: &path })?;
            reqwest::get(url.as_str())
                .context(error::ReqwestGet)?
                .copy_to(&mut f)
                .context(error::ReqwestCopy)?;
            path
        } else {
            eprintln!("No root.json available");
            std::process::exit(1);
        };

        // load repository
        let repo_dir = TempDir::new("tuf").context(error::TempDir)?;
        let repository = Repository::load(Settings {
            root: File::open(&root_path).context(error::OpenRoot { path: &root_path })?,
            datastore: repo_dir.path(),
            metadata_base_url: &self.metadata_base_url,
            target_base_url: &self.target_base_url,
            limits: Limits {
                ..tough::Limits::default()
            },
        })
        .context(error::Metadata)?;

        // copy all available targets
        println!("Downloading targets to {:?}", &self.indir);
        for target in repository.targets().keys() {
            let path = PathBuf::from(&self.indir).join(target);
            println!("\t-> {}", &target);
            let mut reader = repository
                .read_target(target)
                .context(error::Metadata)?
                .context(error::TargetNotFound { target })?;
            let mut f = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&path)
                .context(error::OpenFile { path: &path })?;
            io::copy(&mut reader, &mut f).context(error::WriteTarget)?;
        }
        Ok(())
    }
}
