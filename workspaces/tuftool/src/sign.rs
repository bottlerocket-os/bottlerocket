use crate::error::Result;
use crate::key::{keys_for_root, sign_metadata_inner};
use crate::source::KeySource;
use crate::{load_file, write_file};
use ring::rand::SystemRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use structopt::StructOpt;
use tough::schema::{RoleType, Root, Signed};

#[derive(Debug, StructOpt)]
pub(crate) struct SignArgs {
    /// Path to root.json file for the repository
    #[structopt(short = "r", long = "root")]
    root: PathBuf,

    /// Key files to sign with
    #[structopt(short = "k", long = "key")]
    keys: Vec<KeySource>,

    /// Metadata file to sign
    metadata_file: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct PartialRole {
    #[serde(rename = "_type")]
    type_: RoleType,

    #[serde(flatten)]
    args: HashMap<String, serde_json::Value>,
}

impl SignArgs {
    pub(crate) fn run(&self) -> Result<()> {
        let root: Signed<Root> = load_file(&self.root)?;
        let keys = keys_for_root(&self.keys, &root.signed)?;
        let mut metadata: Signed<PartialRole> = load_file(&self.metadata_file)?;
        sign_metadata_inner(
            &root.signed,
            &keys,
            metadata.signed.type_,
            &mut metadata,
            &SystemRandom::new(),
        )?;
        write_file(&self.metadata_file, &metadata)
    }
}
