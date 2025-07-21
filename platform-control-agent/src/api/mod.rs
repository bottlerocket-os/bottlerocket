// Re-export generated protobuf code
pub use platform::machine::v1alpha1::*;

// Include generated code
pub mod platform {
    pub mod machine {
        pub mod v1alpha1 {
            tonic::include_proto!("platform.machine.v1alpha1");
            
            pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("machine_descriptor");
        }
    }
}