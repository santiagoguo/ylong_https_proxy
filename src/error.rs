use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid proxy URL: {0}")]
    InvalidUrl(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("TLS handshake failed: {0}")]
    Tls(#[from] rustls::Error),

    #[error("Connection refused or closed by proxy")]
    ConnectionClosed,

    #[error("Operation timed out")]
    Timeout,

    #[error("Invalid HTTP response from proxy: {0}")]
    InvalidResponse(String),

    #[error("Connection pool exhausted")]
    PoolExhausted,
}

pub type ProxyResult<T> = Result<T, ProxyError>;
