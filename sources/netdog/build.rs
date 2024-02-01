use bottlerocket_variant::{Variant, VARIANT_ENV};

/// The name of the environment variable that will be set in the event the variant is being built
/// with the `systemd-networkd` backend.  The variable's value can only ever be 1 (based on
/// buildsys), and will only be emitted if the variant's image features include the build flag.
/// TODO: Remove these variables once systemd-networkd integration development is finished
const NET_BACKEND_OVERRIDE_ENV: &str = "SYSTEMD_NETWORKD";
/// The `cfg` value that will be emitted in the case the `NET_BACKEND_OVERRIDE_ENV` variable is set
const NET_BACKEND_OVERRIDE: &str = "systemd-networkd";
/// The default network backend, used in the case the `NET_BACKEND_OVERRIDE_ENV` isn't set in the
/// build environment.
const DEFAULT_NET_BACKEND: &str = "wicked";

fn main() {
    let variant = match Variant::from_env() {
        Ok(variant) => variant,
        Err(e) => {
            eprintln!(
                "For local builds, you must set the '{}' environment variable so we know \
                which data provider to build. Valid values are the directories in \
                models/src/variants/, for example 'aws-ecs-1': {}",
                VARIANT_ENV, e,
            );
            std::process::exit(1);
        }
    };
    variant.emit_cfgs();
    emit_net_backend_cfgs();

    generate_readme::from_main().unwrap();
}

/// Emit `cfg` values that can be used for conditional compilation based on the network backend.
/// TODO: Remove this function once systemd-networkd integration development is finished
fn emit_net_backend_cfgs() {
    let net_backend = if Some("1".to_string()) == std::env::var(NET_BACKEND_OVERRIDE_ENV).ok() {
        NET_BACKEND_OVERRIDE
    } else {
        DEFAULT_NET_BACKEND
    };
    println!("cargo:rerun-if-env-changed={}", NET_BACKEND_OVERRIDE_ENV);
    println!("cargo:rustc-cfg=net_backend=\"{}\"", net_backend);
}
