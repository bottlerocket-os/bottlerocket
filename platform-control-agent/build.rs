use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get OUT_DIR from cargo
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(&["src/api/machine.proto"], &["src/api"])?;
    
    // Print for debugging
    println!("cargo:warning=Generated proto files in: {:?}", out_dir);
    
    Ok(())
}