//! HTTPS Proxy implementation using HTTP CONNECT method

use hyper::Uri;
use std::io::{Error, ErrorKind};
use tokio::net::TcpStream;

use crate::ProxyConfig;

/// Sends an HTTP CONNECT request to the proxy server and upgrades the connection
pub async fn establish_tunnel(
    proxy_config: &ProxyConfig,
    target: &Uri,
) -> Result<TcpStream, Error> {
    // 1. Resolve proxy address
    let proxy_host = proxy_config.proxy_url.host_str().ok_or_else(|| {
        Error::new(ErrorKind::InvalidInput, "Proxy URL missing host")
    })?;
    let proxy_port = proxy_config.proxy_url.port().map(|p| p.as_u16()).unwrap_or(8080);

    // 2. Connect to proxy via TCP
    let mut stream = TcpStream::connect((proxy_host, proxy_port)).await?;

    // 3. Prepare target host for CONNECT
    let target_host = target.host().ok_or_else(|| {
        Error::new(ErrorKind::InvalidInput, "Target URI missing host")
    })?;
    let target_port = target.port().map(|p| p.as_u16()).unwrap_or(443);

    // 4. Construct CONNECT request
    let mut connect_request = format!("CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n", 
        target_host, target_port, target_host, target_port);

    // 5. Add Proxy-Authorization if credentials provided
    if let Some((user, pass)) = &proxy_config.credentials {
        let credentials = base64::encode(format!("{}:{}", user, pass));
        connect_request.push_str(&format!("Proxy-Authorization: Basic {}\r\n", credentials));
    }

    connect_request.push_str("\r\n");

    // 6. Send CONNECT request
    stream.write_all(connect_request.as_bytes()).await?;

    // 7. Read proxy response
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    let response = String::from_utf8_lossy(&buffer[..n]);

    // 8. Check for 200 Connection Established
    if response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200") {
        Ok(stream)
    } else {
        Err(Error::new(
            ErrorKind::ConnectionRefused,
            format!("Proxy connection failed: {}", response),
        ))
    }
}
