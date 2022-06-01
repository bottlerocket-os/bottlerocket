use std::env;

fn main() {
    // TODO: Replace this approach when the build system supports ideas like "variant
    // tags": https://github.com/bottlerocket-os/bottlerocket/issues/1260
    println!("cargo:rerun-if-env-changed=VARIANT");
    if let Ok(variant) = env::var("VARIANT") {
        if variant.starts_with("aws-k8s") {
            println!("cargo:rustc-cfg=aws_k8s_variant");
        }
    }

    generate_readme::from_main().unwrap();
}
