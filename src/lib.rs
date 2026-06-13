//! YLong HTTPS Proxy
//!
//! High-performance HTTPS proxy implementation for ylong_http.
//! Built with OpenSSL for TLS, supporting:
//! - HTTP CONNECT tunneling
//! - TLS 1.2/1.3 with server verification
//! - Mutual TLS (bidirectional authentication) with client certificates
//! - Custom cipher suites and TLS version constraints
//! - Extensible proxy protocol architecture
//! - Sharded connection pooling
//! - Zero-copy forwarding

pub mod config;
pub mod error;
pub mod pool;
pub mod proxy;
pub mod tls;

pub use config::{ProxyConfig, ProxyAuth, TlsConfig, TlsVerifyMode, TlsVersion};
pub use error::{ProxyError, ProxyResult};
pub use pool::ProxyClient;
pub use proxy::{
    ProxyConnector, ProxyProtocol, ProxyProtocolType, ProtocolRegistry,
};
pub use tls::TlsManager;
