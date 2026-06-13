//! Proxy Protocol Abstraction Layer
//!
//! Defines the `ProxyProtocol` trait to ensure extensibility for new proxy protocols.
//! Implement this trait to add support for additional protocols such as SOCKS4, SOCKS5,
//! HTTP CONNECT (already implemented), or custom proxy protocols.

use crate::error::ProxyResult;
use async_trait::async_trait;
use http::Uri;
use tokio::net::TcpStream;

/// Represents the type of proxy protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyProtocolType {
    HttpConnect,
    Socks4,
    Socks5,
    Custom(&'static str),
}

impl std::fmt::Display for ProxyProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyProtocolType::HttpConnect => write!(f, "HTTP CONNECT"),
            ProxyProtocolType::Socks4 => write!(f, "SOCKS4"),
            ProxyProtocolType::Socks5 => write!(f, "SOCKS5"),
            ProxyProtocolType::Custom(name) => write!(f, "{name}"),
        }
    }
}

/// Trait for proxy protocol implementations.
///
/// Each proxy protocol (HTTP CONNECT, SOCKS4, SOCKS5, etc.) should implement this trait
/// to provide a standardized way to establish a TCP tunnel to the target server.
///
/// # Example: Adding SOCKS5 Support
///
/// ```ignore
/// use async_trait::async_trait;
/// use ylong_https_proxy::proxy::protocol::{ProxyProtocol, ProxyProtocolType};
///
/// pub struct Socks5Proxy {
///     config: ProxyConfig,
/// }
///
/// #[async_trait]
/// impl ProxyProtocol for Socks5Proxy {
///     fn protocol_type(&self) -> ProxyProtocolType {
///         ProxyProtocolType::Socks5
///     }
///
///     async fn establish_tunnel(&self, target: &Uri) -> ProxyResult<TcpStream> {
///         // SOCKS5 handshake implementation
///         // ...
///     }
/// }
/// ```
#[async_trait]
pub trait ProxyProtocol: Send + Sync {
    /// Returns the type of this proxy protocol.
    fn protocol_type(&self) -> ProxyProtocolType;

    /// Establishes a TCP tunnel to the target server through the proxy.
    ///
    /// After this method returns successfully, the returned `TcpStream`
    /// is ready for TLS upgrade or direct HTTP communication.
    async fn establish_tunnel(&self, target: &Uri) -> ProxyResult<TcpStream>;
}

/// Protocol Registry for managing multiple proxy protocol implementations.
///
/// This allows users to register custom protocol handlers and select which
/// protocol to use for each connection request.
pub struct ProtocolRegistry {
    protocols: Vec<Box<dyn ProxyProtocol>>,
}

impl ProtocolRegistry {
    pub fn new() -> Self {
        Self {
            protocols: Vec::new(),
        }
    }

    /// Register a new proxy protocol handler.
    pub fn register(&mut self, protocol: Box<dyn ProxyProtocol>) {
        self.protocols.push(protocol);
    }

    /// Find a protocol handler by type.
    pub fn find(&self, protocol_type: ProxyProtocolType) -> Option<&dyn ProxyProtocol> {
        self.protocols
            .iter()
            .find(|p| p.protocol_type() == protocol_type)
            .map(|p| p.as_ref())
    }

    /// Get all registered protocol types.
    pub fn registered_types(&self) -> Vec<ProxyProtocolType> {
        self.protocols
            .iter()
            .map(|p| p.protocol_type())
            .collect()
    }
}

impl Default for ProtocolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
