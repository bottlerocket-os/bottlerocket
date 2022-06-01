use std::{env, process};

fn main() {
    // The code below emits `cfg` operators to conditionally compile this program based on the
    // current variant.
    // TODO: Replace this approach when the build system supports ideas like "variant
    // tags": https://github.com/bottlerocket-os/bottlerocket/issues/1260
    println!("cargo:rerun-if-env-changed=VARIANT");
    if let Ok(variant) = env::var("VARIANT") {
        if variant.starts_with("aws") {
            println!("cargo:rustc-cfg=bottlerocket_platform=\"aws\"");
        } else if variant.starts_with("vmware") {
            println!("cargo:rustc-cfg=bottlerocket_platform=\"vmware\"");
        } else if variant.starts_with("metal") {
            println!("cargo:rustc-cfg=bottlerocket_platform=\"metal\"");
        } else {
            eprintln!(
            "For local builds, you must set the 'VARIANT' environment variable so we know which data \
            provider to build. Valid values are the directories in models/src/variants/, for \
            example 'aws-ecs-1'."
            );
            process::exit(1);
        }
    }

    generate_readme::from_main().unwrap();
}
