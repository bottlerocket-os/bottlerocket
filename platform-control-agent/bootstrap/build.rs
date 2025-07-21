use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);

    // Compile election proto
    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .field_attribute("timestamp", "#[serde(skip)]")
        .field_attribute("last_heartbeat", "#[serde(skip)]")
        .field_attribute("lease_expiry", "#[serde(skip)]")
        .field_attribute("not_before", "#[serde(skip)]")
        .field_attribute("not_after", "#[serde(skip)]")
        .field_attribute("last_check", "#[serde(skip)]")
        .field_attribute("last_backup", "#[serde(skip)]")
        .file_descriptor_set_path(out_dir.join("election_descriptor.bin"))
        .compile_protos(&["proto/election.proto"], &["proto"])?;
    
    // Compile PKI proto
    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .field_attribute("timestamp", "#[serde(skip)]")
        .field_attribute("not_before", "#[serde(skip)]")
        .field_attribute("not_after", "#[serde(skip)]")
        .file_descriptor_set_path(out_dir.join("pki_descriptor.bin"))
        .compile_protos(&["proto/pki.proto"], &["proto"])?;
    
    // Compile etcd proto
    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .field_attribute("timestamp", "#[serde(skip)]")
        .field_attribute("last_check", "#[serde(skip)]")
        .field_attribute("last_backup", "#[serde(skip)]")
        .file_descriptor_set_path(out_dir.join("etcd_descriptor.bin"))
        .compile_protos(&["proto/etcd.proto"], &["proto"])?;

    Ok(())
}