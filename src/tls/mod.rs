use crate::config::ProxyConfig;
use crate::error::{ProxyError, ProxyResult};
use rustls::pki_types::{CertificateDer, ServerName};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tracing::info;

/// Manages TLS configuration and handshakes
pub struct TlsManager {
    client_config: Arc<rustls::ClientConfig>,
}

impl TlsManager {
    /// Create a new TLS manager with default system roots or custom CAs
    pub fn new(config: &ProxyConfig) -> ProxyResult<Self> {
        // Explicitly install the Ring crypto provider
        let _ = rustls::crypto::ring::default_provider().install_default();
        let mut root_store = rustls::RootCertStore::empty();

        // Add system roots (macOS Keychain / Windows Store)
        let native_certs = rustls_native_certs::load_native_certs();
        for cert in native_certs.certs {
            root_store.add(cert).map_err(|e| ProxyError::InvalidResponse(format!("Failed to add native cert: {:?}", e)))?;
        }

        // Add custom CA certs if provided
        for cert_bytes in &config.ca_certs {
            let cert = CertificateDer::from(cert_bytes.clone());
            root_store.add(cert).map_err(|e| ProxyError::InvalidResponse(format!("Failed to add custom CA cert: {:?}", e)))?;
        }

        let client_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Self {
            client_config: Arc::new(client_config),
        })
    }

    /// Upgrade a raw TCP stream to a TLS stream
    pub async fn connect(&self, domain: &str, stream: TcpStream) -> ProxyResult<TlsStream<TcpStream>> {
        info!("Starting TLS handshake for {}", domain);
        
        let server_name = ServerName::try_from(domain.to_string())
            .map_err(|_| ProxyError::InvalidUrl(format!("Invalid domain: {}", domain)))?
            .to_owned();

        let connector = tokio_rustls::TlsConnector::from(self.client_config.clone());
        let tls_stream = connector.connect(server_name, stream).await?;
        
        info!("TLS handshake successful for {}", domain);
        Ok(tls_stream)
    }
}
