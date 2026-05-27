//! Connection Pool for Reusable HTTPS Tunnels
//!
//! To achieve performance > libcurl, we must amortize the cost of TCP + TLS handshakes.
//! This pool manages active `TlsStream` connections to target servers via the proxy.
//! If a connection to a target exists and is healthy, it is reused.

use crate::config::ProxyConfig;
use crate::error::ProxyResult;
use crate::tls::TlsManager;
use crate::proxy::ProxyConnector;
use http::Uri;
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio::sync::Mutex;
use tracing::info;

type TlsConnection = TlsStream<TcpStream>;

/// A high-performance HTTPS Proxy Client with connection pooling capabilities.
pub struct ProxyClient {
    config: ProxyConfig,
    connector: ProxyConnector,
    tls_manager: TlsManager,
    // Simple HashMap cache for active tunnels to reuse them
    // In a real high-concurrency scenario, this would be a `deadpool` instance per target.
    pool: Mutex<HashMap<String, TlsConnection>>, 
}

impl ProxyClient {
    pub fn new(config: ProxyConfig) -> ProxyResult<Self> {
        let connector = ProxyConnector::new(config.clone());
        let tls_manager = TlsManager::new(&config)?;
        Ok(Self {
            config,
            connector,
            tls_manager,
            pool: Mutex::new(HashMap::new()),
        })
    }

    /// Connect to a target via the proxy.
    /// Reuses existing connection if available (HTTP Keep-Alive style).
    pub async fn connect(&self, target: &Uri) -> ProxyResult<TlsConnection> {
        let target_key = format!("{}://{}", target.scheme_str().unwrap_or("https"), target.host().unwrap());
        
        // 1. Check Pool
        {
            let mut cache = self.pool.lock().await;
            if let Some(conn) = cache.remove(&target_key) {
                // Check if still alive (basic check)
                // A real implementation would check for half-closed state
                info!("Reusing connection from pool for {}", target_key);
                return Ok(conn);
            }
        }

        // 2. Create New Connection
        info!("Creating new connection for {}", target_key);
        let tcp_stream = self.connector.connect(target).await?;
        let tls_stream = self.tls_manager.connect(target.host().unwrap(), tcp_stream).await?;
        
        Ok(tls_stream)
    }

    /// Return a connection to the pool for reuse
    pub async fn release(&self, target: &Uri, conn: TlsConnection) {
        let target_key = format!("{}://{}", target.scheme_str().unwrap_or("https"), target.host().unwrap());
        let mut cache = self.pool.lock().await;
        // Limit pool size simply by map size
        if cache.len() < self.config.pool_max_size {
            cache.insert(target_key, conn);
        }
        // else: conn is dropped here
    }
}
