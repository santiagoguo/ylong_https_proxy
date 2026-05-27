use crate::config::ProxyConfig;
use crate::error::{ProxyError, ProxyResult};
use http::Uri;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info};

pub mod io;
pub use io::pipe_streams;

pub struct ProxyConnector {
    config: ProxyConfig,
}

impl ProxyConnector {
    pub fn new(config: ProxyConfig) -> Self {
        Self { config }
    }

    /// Establishes a TCP tunnel to the target via the proxy using HTTP CONNECT
    pub async fn connect(&self, target: &Uri) -> ProxyResult<TcpStream> {
        let proxy_host = self.config.proxy_url.host_str()
            .ok_or_else(|| ProxyError::InvalidUrl("Missing proxy host".into()))?;
        let proxy_port = self.config.proxy_url.port().unwrap_or(8080);

        info!("Connecting to proxy {}:{} -> {}:{}", 
            proxy_host, proxy_port, 
            target.host().unwrap_or("unknown"), 
            target.port().map(|p| p.as_u16()).unwrap_or(443));

        // 1. TCP Connect to Proxy with Timeout
        let mut stream = tokio::time::timeout(
            self.config.connect_timeout,
            TcpStream::connect((proxy_host, proxy_port))
        ).await.map_err(|_| ProxyError::Timeout)??;

        // 2. Perform CONNECT Handshake
        Self::execute_connect(&mut stream, target, &self.config).await?;
        
        info!("HTTPS Proxy tunnel established successfully");
        Ok(stream)
    }

    async fn execute_connect(
        stream: &mut TcpStream, 
        target: &Uri, 
        config: &ProxyConfig
    ) -> ProxyResult<()> {
        let host = target.host().ok_or_else(|| ProxyError::InvalidUrl("Missing target host".into()))?;
        let port = target.port().map(|p| p.as_u16()).unwrap_or(443);

        // Construct CONNECT request
        let mut req = format!("CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n", host, port, host, port);
        
        // Inject Auth if present
        if let crate::config::ProxyAuth::Basic { username, password } = &config.auth {
            let cred = format!("{}:{}", username, password);
            let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, cred);
            req.push_str(&format!("Proxy-Authorization: Basic {}\r\n", encoded));
        }
        req.push_str("\r\n");

        debug!("Sending CONNECT request");
        stream.write_all(req.as_bytes()).await?;

        // Read Response State Machine
        let mut buffer = Vec::with_capacity(512);
        let mut temp = [0u8; 256];
        
        // Read until \r\n\r\n (HTTP Headers end)
        loop {
            let n = stream.read(&mut temp).await?;
            if n == 0 {
                return Err(ProxyError::ConnectionClosed);
            }
            buffer.extend_from_slice(&temp[..n]);
            if let Some(pos) = Self::find_crlf_crlf(&buffer) {
                let headers = &buffer[..pos];
                let end_of_line = headers.iter().position(|&b| b == b'\r' || b == b'\n').unwrap_or(headers.len());
                let status_line = String::from_utf8_lossy(&headers[..end_of_line]);
                
                debug!("Proxy Response: {}", status_line);
                
                if status_line.starts_with("HTTP/1.1 200") || status_line.starts_with("HTTP/1.0 200") {
                    return Ok(());
                } else if status_line.contains("407") {
                    return Err(ProxyError::AuthFailed(status_line.to_string()));
                } else {
                    return Err(ProxyError::InvalidResponse(status_line.to_string()));
                }
            }
        }
    }

    fn find_crlf_crlf(buf: &[u8]) -> Option<usize> {
        buf.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4)
    }
}
