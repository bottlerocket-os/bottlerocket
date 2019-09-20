use crate::copylike::Copylike;
use crate::error::{self, Result};
use crate::key::{keys_for_root, sign_metadata, RootKeys};
use crate::source::KeySource;
use chrono::{DateTime, Utc};
use maplit::hashmap;
use rayon::prelude::*;
use ring::rand::SystemRandom;
use serde::Serialize;
use sha2::{Digest, Sha256};
use snafu::{OptionExt, ResultExt};
use std::collections::HashMap;
use std::fs::File;
use std::num::{NonZeroU64, NonZeroUsize};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use tough_schema::decoded::Decoded;
use tough_schema::{
    Hashes, Role, RoleType, Root, Signed, Snapshot, SnapshotMeta, Target, Targets, Timestamp,
    TimestampMeta,
};
use walkdir::WalkDir;

#[derive(Debug, StructOpt)]
pub(crate) struct CreateArgs {
    /// Copy files into `outdir` instead of symlinking them
    #[structopt(short = "c", long = "copy")]
    copy: bool,
    /// Hardlink files into `outdir` instead of symlinking them
    #[structopt(short = "H", long = "hardlink")]
    hardlink: bool,

    /// Follow symbolic links in `indir`
    #[structopt(short = "f", long = "follow")]
    follow: bool,

    /// Number of target hashing threads to run (default: number of cores)
    #[structopt(short = "j", long = "jobs")]
    jobs: Option<NonZeroUsize>,

    /// Key files to sign with
    #[structopt(short = "k", long = "key")]
    keys: Vec<KeySource>,

    /// Version of snapshot.json file
    #[structopt(long = "snapshot-version")]
    snapshot_version: NonZeroU64,
    /// Expiration of snapshot.json file
    #[structopt(long = "snapshot-expires")]
    snapshot_expires: DateTime<Utc>,

    /// Version of targets.json file
    #[structopt(long = "targets-version")]
    targets_version: NonZeroU64,
    /// Expiration of targets.json file
    #[structopt(long = "targets-expires")]
    targets_expires: DateTime<Utc>,

    /// Version of timestamp.json file
    #[structopt(long = "timestamp-version")]
    timestamp_version: NonZeroU64,
    /// Expiration of timestamp.json file
    #[structopt(long = "timestamp-expires")]
    timestamp_expires: DateTime<Utc>,

    /// Path to root.json file for the repository
    #[structopt(short = "r", long = "root")]
    root: PathBuf,

    /// Directory of targets
    indir: PathBuf,
    /// Repository output directory
    outdir: PathBuf,
}

impl CreateArgs {
    pub(crate) fn run(&self) -> Result<()> {
        if let Some(jobs) = self.jobs {
            rayon::ThreadPoolBuilder::new()
                .num_threads(usize::from(jobs))
                .build_global()
                .context(error::InitializeThreadPool)?;
        }

        let root_buf = std::fs::read(&self.root).context(error::FileRead { path: &self.root })?;
        let root = serde_json::from_slice::<Signed<Root>>(&root_buf)
            .context(error::FileParseJson { path: &self.root })?
            .signed;
        let mut root_sha256 = [0; 32];
        root_sha256.copy_from_slice(Sha256::digest(&root_buf).as_slice());
        let root_length = root_buf.len() as u64;

        CreateProcess {
            args: self,
            keys: keys_for_root(&self.keys, &root)?,
            rng: SystemRandom::new(),
            root,
            root_sha256,
            root_length,
        }
        .run()
    }
}

struct CreateProcess<'a> {
    args: &'a CreateArgs,
    rng: SystemRandom,
    root: Root,
    root_sha256: [u8; 32],
    root_length: u64,
    keys: RootKeys,
}

impl<'a> CreateProcess<'a> {
    fn run(self) -> Result<()> {
        let root_path = self
            .args
            .outdir
            .join("metadata")
            .join(format!("{}.root.json", self.root.version));
        self.copy_action()
            .run(&self.args.root, &root_path)
            .context(error::FileCopy {
                action: self.copy_action(),
                src: &self.args.root,
                dst: root_path,
            })?;

        let (targets_sha256, targets_length) = self.write_metadata(
            Targets {
                spec_version: crate::SPEC_VERSION.to_owned(),
                version: self.args.targets_version,
                expires: self.args.targets_expires,
                targets: self.build_targets()?,
                _extra: HashMap::new(),
            },
            self.args.targets_version,
            "targets.json",
        )?;

        let (snapshot_sha256, snapshot_length) = self.write_metadata(
            Snapshot {
                spec_version: crate::SPEC_VERSION.to_owned(),
                version: self.args.snapshot_version,
                expires: self.args.snapshot_expires,
                meta: hashmap! {
                    "root.json".to_owned() => SnapshotMeta {
                        hashes: Some(Hashes {
                            sha256: self.root_sha256.to_vec().into(),
                            _extra: HashMap::new(),
                        }),
                        length: Some(self.root_length),
                        version: self.root.version,
                        _extra: HashMap::new(),
                    },
                    "targets.json".to_owned() => SnapshotMeta {
                        hashes: Some(Hashes {
                            sha256: targets_sha256.to_vec().into(),
                            _extra: HashMap::new(),
                        }),
                        length: Some(targets_length),
                        version: self.args.targets_version,
                        _extra: HashMap::new(),
                    },
                },
                _extra: HashMap::new(),
            },
            self.args.snapshot_version,
            "snapshot.json",
        )?;

        self.write_metadata(
            Timestamp {
                spec_version: crate::SPEC_VERSION.to_owned(),
                version: self.args.snapshot_version,
                expires: self.args.snapshot_expires,
                meta: hashmap! {
                    "snapshot.json".to_owned() => TimestampMeta {
                        hashes: Hashes {
                            sha256: snapshot_sha256.to_vec().into(),
                            _extra: HashMap::new(),
                        },
                        length: snapshot_length,
                        version: self.args.snapshot_version,
                        _extra: HashMap::new(),
                    }
                },
                _extra: HashMap::new(),
            },
            self.args.timestamp_version,
            "timestamp.json",
        )?;

        Ok(())
    }

    // =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    fn copy_action(&self) -> Copylike {
        match (self.args.copy, self.args.hardlink) {
            (true, _) => Copylike::Copy, // --copy overrides --hardlink
            (false, true) => Copylike::Hardlink,
            (false, false) => Copylike::Symlink,
        }
    }

    fn build_targets(&self) -> Result<HashMap<String, Target>> {
        WalkDir::new(&self.args.indir)
            .follow_links(self.args.follow)
            .into_iter()
            .par_bridge()
            .filter_map(|entry| match entry {
                Ok(entry) => {
                    if entry.file_type().is_file() {
                        Some(self.process_target(entry.path()))
                    } else {
                        None
                    }
                }
                Err(err) => Some(Err(err).context(error::WalkDir)),
            })
            .collect()
    }

    fn process_target(&self, path: &Path) -> Result<(String, Target)> {
        let target_name = path.strip_prefix(&self.args.indir).context(error::Prefix {
            path,
            base: &self.args.indir,
        })?;
        let target_name = target_name
            .to_str()
            .context(error::PathUtf8 { path: target_name })?
            .to_owned();

        let mut file = File::open(path).context(error::FileOpen { path })?;
        let mut digest = Sha256::new();
        let length = std::io::copy(&mut file, &mut digest).context(error::FileRead { path })?;

        let target = Target {
            length,
            hashes: Hashes {
                sha256: Decoded::from(digest.result().as_slice().to_vec()),
                _extra: HashMap::new(),
            },
            custom: HashMap::new(),
            _extra: HashMap::new(),
        };

        let dst = if self.root.consistent_snapshot {
            self.args.outdir.join("targets").join(format!(
                "{}.{}",
                hex::encode(&target.hashes.sha256),
                target_name
            ))
        } else {
            self.args.outdir.join("targets").join(&target_name)
        };
        self.copy_action()
            .run(path, &dst)
            .context(error::FileCopy {
                action: self.copy_action(),
                src: path,
                dst,
            })?;

        Ok((target_name, target))
    }

    // =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

    fn write_metadata<T: Role + Serialize>(
        &self,
        role: T,
        version: NonZeroU64,
        filename: &'static str,
    ) -> Result<([u8; 32], u64)> {
        let metadir = self.args.outdir.join("metadata");
        std::fs::create_dir_all(&metadir).context(error::FileCreate { path: &metadir })?;

        let path = metadir.join(
            if T::TYPE != RoleType::Timestamp && self.root.consistent_snapshot {
                format!("{}.{}", version, filename)
            } else {
                filename.to_owned()
            },
        );

        let mut role = Signed {
            signed: role,
            signatures: Vec::new(),
        };
        self.sign_metadata(&mut role)?;

        let mut buf =
            serde_json::to_vec_pretty(&role).context(error::FileWriteJson { path: &path })?;
        buf.push(b'\n');
        std::fs::write(&path, &buf).context(error::FileCreate { path: &path })?;

        let mut sha256 = [0; 32];
        sha256.copy_from_slice(Sha256::digest(&buf).as_slice());
        Ok((sha256, buf.len() as u64))
    }

    fn sign_metadata<T: Role + Serialize>(&self, role: &mut Signed<T>) -> Result<()> {
        sign_metadata(&self.root, &self.keys, role, &self.rng)
    }
}
