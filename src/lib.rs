//! YLong HTTPS Proxy
//! High-performance HTTPS proxy implementation for ylong_http.
//! Focus: CONNECT tunneling, TLS upgrade, Zero-copy forwarding, Connection pooling.

pub mod config;
pub mod error;
pub mod pool;
pub mod proxy;
pub mod tls;

pub use config::{ProxyConfig, ProxyAuth};
pub use error::{ProxyError, ProxyResult};
pub use pool::ProxyClient;
pub use proxy::ProxyConnector;
pub use tls::TlsManager;
