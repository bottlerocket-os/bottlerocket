pub mod election;
pub mod pki;
pub mod etcd;
pub mod coordinator;

// Proto modules
pub mod proto {
    pub mod election {
        tonic::include_proto!("platform.bootstrap.election.v1alpha1");
        
        pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("election_descriptor");
    }
    
    pub mod pki {
        tonic::include_proto!("platform.bootstrap.pki.v1alpha1");
        
        pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("pki_descriptor");
    }
    
    pub mod etcd {
        tonic::include_proto!("platform.bootstrap.etcd.v1alpha1");
        
        pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("etcd_descriptor");
    }
}

pub use election::{ElectionService, ElectionState, ElectionConfig, NodeInfo};
pub use pki::{PKIService, Certificate, PKIConfig, CertificateAuthority, CertificateStore, PKIDistributor};
pub use etcd::{EtcdService, EtcdConfig};