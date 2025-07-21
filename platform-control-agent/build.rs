use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get OUT_DIR from cargo
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    let proto_files = &["src/api/machine.proto"];
    let proto_includes = &["src/api"];
    
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .file_descriptor_set_path(out_dir.join("machine_descriptor.bin"))
        .compile_protos(proto_files, proto_includes)?;
    
    // Print for debugging
    println!("cargo:warning=Generated proto files in: {:?}", out_dir);
    
    Ok(())
}