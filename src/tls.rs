//! YLong HTTPS Proxy Module
//! Implements CONNECT method and TLS tunneling for ylong_http

use std::sync::Arc;
use url::Url;
use rustls::pki_types::ServerName;

/// Configuration for the HTTPS Proxy
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// Proxy server URL (e.g., http://proxy.example.com:8080)
    pub proxy_url: Url,
    /// Basic Auth credentials (username, password)
    pub credentials: Option<(String, String)>,
    /// Custom CA certificates for verification
    pub ca_certs: Vec<Vec<u8>>,
}

impl ProxyConfig {
    pub fn new(proxy_url: Url) -> Self {
        Self {
            proxy_url,
            credentials: None,
            ca_certs: Vec::new(),
        }
    }

    pub fn with_auth(mut self, username: String, password: String) -> Self {
        self.credentials = Some((username, password));
        self
    }

    pub fn with_ca_cert(mut self, cert: Vec<u8>) -> Self {
        self.ca_certs.push(cert);
        self
    }
}

/// Handles the TLS part of the HTTPS Proxy connection
pub struct TlsConnector {
    config: Arc<rustls::ClientConfig>,
}

impl TlsConnector {
    pub fn new(proxy_config: &ProxyConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let root_store = rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        };

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Self {
            config: Arc::new(config),
        })
    }

    /// Perform TLS Handshake over the existing TCP stream
    pub async fn connect<S>(
        &self,
        domain: &str,
        stream: S,
    ) -> Result<tokio_rustls::client::TlsStream<S>, Box<dyn std::error::Error>>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        let server_name = ServerName::try_from(domain)?.to_owned();
        let connector = tokio_rustls::TlsConnector::from(self.config.clone());
        let stream = connector.connect(server_name, stream).await?;
        Ok(stream)
    }
}
