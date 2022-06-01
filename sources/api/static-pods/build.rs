use bottlerocket_variant::Variant;

fn main() {
    let variant = Variant::from_env().unwrap();
    variant.emit_cfgs();
    generate_readme::from_file("src/static_pods.rs").unwrap();
}
