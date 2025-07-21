use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tonic::{Request, Response, Status};

use crate::proto::pki::{
    pki_service_server::PkiService as PkiServiceTrait,
    InitializePkiRequest, InitializePkiResponse,
    CertificateRequest, CertificateResponse,
    RenewRequest, 
    GetCaBundleRequest, CaBundleResponse,
    ListCertificatesRequest, ListCertificatesResponse,
    ObserveCertificatesRequest, CertificateEvent,
    PkiConfig as ProtoPkiConfig,
};

use super::ca::{CertificateAuthority, PKIConfig};
use super::store::CertificateStore;

pub struct PKIService {
    ca: Arc<RwLock<CertificateAuthority>>,
    store: Arc<RwLock<CertificateStore>>,
    is_initialized: Arc<RwLock<bool>>,
    event_tx: mpsc::Sender<CertificateEvent>,
}

impl PKIService {
    pub fn new() -> Self {
        let config = PKIConfig::default();
        let ca = Arc::new(RwLock::new(CertificateAuthority::new(config)));
        let store = Arc::new(RwLock::new(CertificateStore::new()));
        let (event_tx, _event_rx) = mpsc::channel(1000);
        
        Self {
            ca,
            store,
            is_initialized: Arc::new(RwLock::new(false)),
            event_tx,
        }
    }
    
    /// Check if we're the leader (only leader can initialize PKI)
    async fn check_leader(&self) -> Result<(), Status> {
        // TODO: Check with election service if we're the leader
        Ok(())
    }
    
    /// Convert proto config to internal config
    fn proto_to_config(proto: Option<ProtoPkiConfig>) -> PKIConfig {
        if let Some(p) = proto {
            PKIConfig {
                organization: p.organization,
                country: p.country,
                locality: p.locality,
                key_size: 4096, // TODO: Parse from key_algorithm
                root_ca_validity_years: p.validity.as_ref().map(|v| v.root_ca_years as u32).unwrap_or(10),
                intermediate_ca_validity_years: p.validity.as_ref().map(|v| v.intermediate_ca_years as u32).unwrap_or(5),
                server_cert_validity_years: p.validity.as_ref().map(|v| v.server_cert_years as u32).unwrap_or(1),
                client_cert_validity_years: p.validity.as_ref().map(|v| v.client_cert_years as u32).unwrap_or(1),
            }
        } else {
            PKIConfig::default()
        }
    }
}

#[tonic::async_trait]
impl PkiServiceTrait for PKIService {
    async fn initialize_pki(
        &self,
        _request: Request<InitializePkiRequest>,
    ) -> Result<Response<InitializePkiResponse>, Status> {
        Err(Status::unimplemented("PKI service not yet implemented"))
    }
    
    async fn request_certificate(
        &self,
        _request: Request<CertificateRequest>,
    ) -> Result<Response<CertificateResponse>, Status> {
        Err(Status::unimplemented("PKI service not yet implemented"))
    }
    
    async fn renew_certificate(
        &self,
        _request: Request<RenewRequest>,
    ) -> Result<Response<CertificateResponse>, Status> {
        Err(Status::unimplemented("PKI service not yet implemented"))
    }
    
    async fn get_ca_bundle(
        &self,
        _request: Request<GetCaBundleRequest>,
    ) -> Result<Response<CaBundleResponse>, Status> {
        Err(Status::unimplemented("PKI service not yet implemented"))
    }
    
    async fn list_certificates(
        &self,
        _request: Request<ListCertificatesRequest>,
    ) -> Result<Response<ListCertificatesResponse>, Status> {
        Err(Status::unimplemented("PKI service not yet implemented"))
    }
    
    type ObserveCertificatesStream = std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<CertificateEvent, Status>> + Send>>;
    
    async fn observe_certificates(
        &self,
        _request: Request<ObserveCertificatesRequest>,
    ) -> Result<Response<Self::ObserveCertificatesStream>, Status> {
        Err(Status::unimplemented("PKI service not yet implemented"))
    }
}