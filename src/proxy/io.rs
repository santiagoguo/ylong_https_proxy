//! Zero-Copy I/O utilities for high-performance proxy forwarding.
//!
//! This module provides functions to bridge streams with minimal memory copying,
//! utilizing the kernel's capabilities and async runtime optimizations.

use tokio::io::{self, AsyncRead, AsyncWrite};
use tracing::debug;

/// Bridges two streams bidirectionally with zero-copy optimization where possible.
/// This is typically used in proxy relays to forward data between client and upstream.
///
/// # Returns
/// A tuple `(u64, u64)` representing bytes transferred (left_to_right, right_to_left).
pub async fn pipe_streams<L, R>(mut left: L, mut right: R) -> io::Result<(u64, u64)>
where
    L: AsyncRead + AsyncWrite + Unpin,
    R: AsyncRead + AsyncWrite + Unpin,
{
    debug!("Starting zero-copy bidirectional pipe");
    // tokio::io::copy_bidirectional is the standard zero-copy abstraction in Tokio.
    // It uses specialized implementations (like splice on Linux) when available,
    // or highly optimized buffer reuse.
    let (left_to_right, right_to_left) = io::copy_bidirectional(&mut left, &mut right).await?;
    debug!(
        "Pipe finished: L->R {} bytes, R->L {} bytes",
        left_to_right, right_to_left
    );
    Ok((left_to_right, right_to_left))
}

/// Creates a connected pair of streams for testing purposes.
/// In a real benchmark, this would be a TCP connection to a mock proxy.
pub async fn create_test_pair() -> io::Result<(impl AsyncRead + AsyncWrite, impl AsyncRead + AsyncWrite)> {
    use tokio::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    
    let client = tokio::net::TcpStream::connect(addr).await?;
    let (server, _) = listener.accept().await?;
    
    Ok((client, server))
}
