use anyhow::Result;
use tonic::transport::Channel;

use crate::proto::pki::{
    pki_service_client::PkiServiceClient,
    CertificateRequest, CertificateResponse,
    GetCaBundleRequest, CaBundleResponse,
};

/// PKI client for nodes to request certificates
pub struct PKIClient {
    client: PkiServiceClient<Channel>,
}

impl PKIClient {
    /// Connect to PKI service
    pub async fn connect(addr: String) -> Result<Self> {
        let client = PkiServiceClient::connect(addr).await?;
        Ok(Self { client })
    }
    
    /// Request a certificate
    pub async fn request_certificate(
        &mut self,
        request: CertificateRequest,
    ) -> Result<CertificateResponse> {
        let response = self.client.request_certificate(request).await?;
        Ok(response.into_inner())
    }
    
    /// Get CA bundle for certificate validation
    pub async fn get_ca_bundle(&mut self) -> Result<CaBundleResponse> {
        let response = self.client.get_ca_bundle(GetCaBundleRequest {}).await?;
        Ok(response.into_inner())
    }
}