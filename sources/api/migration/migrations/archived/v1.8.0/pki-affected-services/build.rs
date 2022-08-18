use bottlerocket_variant::Variant;

fn main() {
    let variant = Variant::from_env().unwrap();
    variant.emit_cfgs();
}
