use std::path::PathBuf;
use std::process::{exit, Command};

fn main() -> Result<(), std::io::Error> {
    let root_json_path = PathBuf::from(env!("PUBLISH_REPO_ROOT_JSON"));
    println!("cargo:rerun-if-changed={}", root_json_path.display());
    std::env::set_var("BUILDSYS_DEPRECATED_FEATURE_VARIANT_SENSITIVITY", "1");

    let ret = Command::new("buildsys").arg("build-package").status()?;
    if !ret.success() {
        exit(1);
    }
    Ok(())
}
