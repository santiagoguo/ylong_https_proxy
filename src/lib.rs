//! YLong HTTPS Proxy
//! A high-performance HTTPS proxy implementation for ylong_http.

mod proxy;
mod tls;

pub use proxy::establish_tunnel;
pub use tls::TlsConnector;
pub use crate::ProxyConfig;

use http::Uri;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

/// A unified HTTPS Proxy Client
pub struct HttpsProxyClient {
    config: crate::ProxyConfig,
    tls_connector: TlsConnector,
}

impl HttpsProxyClient {
    /// Create a new client from proxy config
    pub fn new(config: crate::ProxyConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let tls_connector = TlsConnector::new(&config)?;
        Ok(Self { config, tls_connector })
    }

    /// Connect to target via Proxy and perform TLS Handshake
    /// Returns a secure TlsStream ready for HTTPS requests
    pub async fn connect(
        &self,
        target: &Uri,
    ) -> Result<TlsStream<TcpStream>, Box<dyn std::error::Error>> {
        // Step 1: Establish CONNECT tunnel
        let stream = establish_tunnel(&self.config, target).await?;
        
        // Step 2: Perform TLS Handshake
        let domain = target.host().ok_or("Target URI missing host")?;
        let tls_stream = self.tls_connector.connect(domain, stream).await?;
        
        Ok(tls_stream)
    }
}
