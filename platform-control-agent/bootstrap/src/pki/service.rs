use std::sync::Arc;
use std::pin::Pin;
use tokio::sync::{RwLock, mpsc, broadcast};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{Request, Response, Status};
use tracing::{info, error, warn, debug};

use crate::proto::pki::{
    pki_service_server::PkiService as PkiServiceTrait,
    InitializePkiRequest, InitializePkiResponse,
    CertificateRequest, CertificateResponse,
    RenewRequest, 
    GetCaBundleRequest, CaBundleResponse,
    ListCertificatesRequest, ListCertificatesResponse,
    ObserveCertificatesRequest, CertificateEvent,
    PkiConfig as ProtoPkiConfig,
    CertificateEventType, CertificateType, CertificateStatus,
};

use crate::{
    pki::{
        ca::{CertificateAuthority, PKIConfig, Certificate},
        store::CertificateStore,
        distribution::PKIDistributor,
    },
    election::ElectionState,
};

pub struct PKIService {
    ca: Arc<CertificateAuthority>,
    store: Arc<CertificateStore>,
    distributor: Arc<PKIDistributor>,
    election_state: Arc<ElectionState>,
    is_initialized: Arc<RwLock<bool>>,
    event_tx: broadcast::Sender<CertificateEvent>,
}

impl PKIService {
    pub fn new(election_state: Arc<ElectionState>) -> Self {
        let config = PKIConfig::default();
        let ca = Arc::new(CertificateAuthority::new(config));
        let store = Arc::new(CertificateStore::new());
        let distributor = Arc::new(PKIDistributor::new(
            store.clone(),
            election_state.clone(),
        ));
        let (event_tx, _event_rx) = broadcast::channel(1000);
        
        Self {
            ca,
            store,
            distributor,
            election_state,
            is_initialized: Arc::new(RwLock::new(false)),
            event_tx,
        }
    }
    
    /// Start background tasks
    pub async fn start(&self) -> anyhow::Result<()> {
        // Start renewal checker task
        let distributor = self.distributor.clone();
        let event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Check hourly
            loop {
                interval.tick().await;
                if let Ok(expiring) = distributor.check_renewals().await {
                    for node_id in expiring {
                        let msg = format!("Certificate renewal needed for node: {}", node_id);
                        warn!("{}", msg);
                        
                        // Send expiring event
                        let event = CertificateEvent {
                            timestamp: Some(prost_types::Timestamp {
                                seconds: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs() as i64,
                                nanos: 0,
                            }),
                            r#type: CertificateEventType::ExpiringSoon as i32,
                            certificate: None,
                            details: msg,
                        };
                        let _ = event_tx.send(event);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Check if we're the leader (only leader can initialize PKI)
    async fn check_leader(&self) -> Result<(), Status> {
        if !self.election_state.is_leader().await {
            return Err(Status::permission_denied("Only the elected leader can perform PKI operations"));
        }
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
    
    /// Send certificate event
    async fn send_event(&self, event_type: CertificateEventType, cert: Option<Certificate>, details: String) {
        let event = CertificateEvent {
            timestamp: Some(prost_types::Timestamp {
                seconds: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                nanos: 0,
            }),
            r#type: event_type as i32,
            certificate: cert.and_then(|c| c.to_proto().ok()),
            details,
        };
        let _ = self.event_tx.send(event);
    }
}

#[tonic::async_trait]
impl PkiServiceTrait for PKIService {
    async fn initialize_pki(
        &self,
        request: Request<InitializePkiRequest>,
    ) -> Result<Response<InitializePkiResponse>, Status> {
        self.check_leader().await?;
        
        let req = request.into_inner();
        let mut initialized = self.is_initialized.write().await;
        
        if *initialized && !req.force {
            return Err(Status::already_exists("PKI already initialized"));
        }
        
        info!("Initializing PKI system");
        
        // Update CA configuration
        let config = Self::proto_to_config(req.config);
        let mut ca = CertificateAuthority::new(config);
        
        // Generate certificate hierarchy
        match ca.initialize() {
            Ok(_) => {
                info!("PKI hierarchy generated successfully");
                
                // Store certificates
                if let Err(e) = self.store.store_root_ca(ca.get_root_ca().unwrap().clone()).await {
                    let msg = format!("Failed to store root CA: {}", e);
                    error!("{}", msg);
                    return Err(Status::internal(msg));
                }
                
                if let Some(k8s_ca) = ca.get_kubernetes_ca() {
                    if let Err(e) = self.store.store_kubernetes_ca(k8s_ca.clone()).await {
                        let msg = format!("Failed to store Kubernetes CA: {}", e);
                        error!("{}", msg);
                        return Err(Status::internal(msg));
                    }
                }
                
                if let Some(etcd_ca) = ca.get_etcd_ca() {
                    if let Err(e) = self.store.store_etcd_ca(etcd_ca.clone()).await {
                        let msg = format!("Failed to store etcd CA: {}", e);
                        error!("{}", msg);
                        return Err(Status::internal(msg));
                    }
                }
                
                *initialized = true;
                
                // Send initialization event
                self.send_event(
                    CertificateEventType::Issued,
                    ca.get_root_ca().cloned(),
                    "PKI system initialized".to_string(),
                ).await;
                
                let root_fingerprint = ca.get_root_ca()
                    .and_then(|c| c.to_proto().ok())
                    .map(|p| p.fingerprint)
                    .unwrap_or_default();
                
                let mut intermediate_ca_fingerprints = Vec::new();
                if let Some(ca) = ca.get_kubernetes_ca() {
                    if let Ok(proto) = ca.to_proto() {
                        intermediate_ca_fingerprints.push(proto.fingerprint);
                    }
                }
                if let Some(ca) = ca.get_etcd_ca() {
                    if let Ok(proto) = ca.to_proto() {
                        intermediate_ca_fingerprints.push(proto.fingerprint);
                    }
                }
                if let Some(ca) = ca.get_front_proxy_ca() {
                    if let Ok(proto) = ca.to_proto() {
                        intermediate_ca_fingerprints.push(proto.fingerprint);
                    }
                }
                
                Ok(Response::new(InitializePkiResponse {
                    success: true,
                    root_ca_fingerprint: root_fingerprint,
                    intermediate_ca_fingerprints,
                }))
            }
            Err(e) => {
                let msg = format!("Failed to generate PKI hierarchy: {}", e);
                error!("{}", msg);
                Err(Status::internal(msg))
            }
        }
    }
    
    async fn request_certificate(
        &self,
        request: Request<CertificateRequest>,
    ) -> Result<Response<CertificateResponse>, Status> {
        let initialized = self.is_initialized.read().await;
        if !*initialized {
            return Err(Status::failed_precondition("PKI not initialized"));
        }
        
        let req = request.into_inner();
        debug!("Certificate request from node: {}", req.node_id);
        
        // Use distributor to handle the request
        match self.distributor.process_certificate_request(req).await {
            Ok(response) => {
                // Send issued event
                if let Some(cert) = &response.certificate {
                    self.send_event(
                        CertificateEventType::Issued,
                        None,
                        format!("Certificate issued to {}", cert.common_name),
                    ).await;
                }
                
                Ok(Response::new(response))
            }
            Err(e) => {
                let msg = format!("Failed to process certificate request: {}", e);
                error!("{}", msg);
                Err(Status::internal(msg))
            }
        }
    }
    
    async fn renew_certificate(
        &self,
        request: Request<RenewRequest>,
    ) -> Result<Response<CertificateResponse>, Status> {
        let initialized = self.is_initialized.read().await;
        if !*initialized {
            return Err(Status::failed_precondition("PKI not initialized"));
        }
        
        let req = request.into_inner();
        debug!("Certificate renewal request for fingerprint: {}", req.fingerprint);
        
        // TODO: Implement certificate renewal
        // 1. Find existing certificate by fingerprint
        // 2. Verify caller owns the certificate
        // 3. Generate new certificate with same attributes
        // 4. Return new certificate
        
        Err(Status::unimplemented("Certificate renewal not yet implemented"))
    }
    
    async fn get_ca_bundle(
        &self,
        _request: Request<GetCaBundleRequest>,
    ) -> Result<Response<CaBundleResponse>, Status> {
        let initialized = self.is_initialized.read().await;
        if !*initialized {
            return Err(Status::failed_precondition("PKI not initialized"));
        }
        
        // Get CA certificates
        let root_ca = self.store.get_root_ca().await
            .ok_or_else(|| Status::internal("Root CA not found"))?;
        let k8s_ca = self.store.get_kubernetes_ca().await;
        let etcd_ca = self.store.get_etcd_ca().await;
        
        // Convert to PEM
        let root_pem = String::from_utf8(root_ca.cert.to_pem().map_err(|e| 
            Status::internal(format!("Failed to encode root CA: {}", e)))?
        ).map_err(|e| Status::internal(format!("Invalid UTF-8: {}", e)))?;
        
        let mut intermediate_cas = Vec::new();
        let mut bundle = root_pem.clone();
        
        if let Some(ca) = k8s_ca {
            let pem = String::from_utf8(ca.cert.to_pem().map_err(|e| 
                Status::internal(format!("Failed to encode Kubernetes CA: {}", e)))?
            ).map_err(|e| Status::internal(format!("Invalid UTF-8: {}", e)))?;
            intermediate_cas.push(pem.clone());
            bundle.push_str(&pem);
        }
        
        if let Some(ca) = etcd_ca {
            let pem = String::from_utf8(ca.cert.to_pem().map_err(|e| 
                Status::internal(format!("Failed to encode etcd CA: {}", e)))?
            ).map_err(|e| Status::internal(format!("Invalid UTF-8: {}", e)))?;
            intermediate_cas.push(pem.clone());
            bundle.push_str(&pem);
        }
        
        Ok(Response::new(CaBundleResponse {
            root_ca: root_pem,
            intermediate_cas,
            bundle,
        }))
    }
    
    async fn list_certificates(
        &self,
        request: Request<ListCertificatesRequest>,
    ) -> Result<Response<ListCertificatesResponse>, Status> {
        let initialized = self.is_initialized.read().await;
        if !*initialized {
            return Err(Status::failed_precondition("PKI not initialized"));
        }
        
        let req = request.into_inner();
        let mut certificates = Vec::new();
        
        // Get stored certificates based on type filter
        if req.r#type == 0 || req.r#type == CertificateType::RootCa as i32 {
            if let Some(cert) = self.store.get_root_ca().await {
                if let Ok(proto) = cert.to_proto() {
                    certificates.push(proto);
                }
            }
        }
        
        if req.r#type == 0 || req.r#type == CertificateType::IntermediateCa as i32 {
            if let Some(cert) = self.store.get_kubernetes_ca().await {
                if let Ok(proto) = cert.to_proto() {
                    certificates.push(proto);
                }
            }
            if let Some(cert) = self.store.get_etcd_ca().await {
                if let Ok(proto) = cert.to_proto() {
                    certificates.push(proto);
                }
            }
        }
        
        // Filter by status
        if !req.include_expired {
            certificates.retain(|c| c.status != CertificateStatus::Expired as i32);
        }
        if !req.include_revoked {
            certificates.retain(|c| c.status != CertificateStatus::Revoked as i32);
        }
        
        Ok(Response::new(ListCertificatesResponse { certificates }))
    }
    
    type ObserveCertificatesStream = Pin<Box<dyn tokio_stream::Stream<Item = Result<CertificateEvent, Status>> + Send>>;
    
    async fn observe_certificates(
        &self,
        request: Request<ObserveCertificatesRequest>,
    ) -> Result<Response<Self::ObserveCertificatesStream>, Status> {
        let req = request.into_inner();
        let filter_types: Vec<i32> = req.types;
        
        // Create a new receiver
        let (tx, rx) = mpsc::channel(100);
        let mut event_rx = self.event_tx.subscribe();
        
        // Spawn task to filter and forward events
        tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                // Filter by event type if specified
                if filter_types.is_empty() || filter_types.contains(&event.r#type) {
                    if tx.send(Ok(event)).await.is_err() {
                        break; // Client disconnected
                    }
                }
            }
        });
        
        let stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream)))
    }
}

/// Helper to implement PKIServiceImpl for backward compatibility
pub struct PKIServiceImpl {
    inner: PKIService,
}

impl PKIServiceImpl {
    pub fn new(election_state: Arc<ElectionState>) -> Self {
        Self {
            inner: PKIService::new(election_state),
        }
    }
    
    pub fn into_service(self) -> PKIService {
        self.inner
    }
}