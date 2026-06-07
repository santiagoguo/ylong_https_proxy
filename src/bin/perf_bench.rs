//! Simple manual benchmark runner for ylong_https_proxy
//!
//! Run with: cargo run --release --bin perf_bench
//!
//! This starts a mock proxy, runs the client against it with varying concurrency,
//! and prints performance metrics.

use http::Uri;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use url::Url;
use ylong_https_proxy::config::ProxyConfig;
use ylong_https_proxy::proxy::ProxyConnector;
use std::time::Instant;
use std::sync::Arc;

/// A mock proxy server that accepts CONNECT and replies 200 OK
async fn run_mock_proxy(addr: SocketAddr) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            let mut headers = Vec::new();
            loop {
                let n = match stream.read(&mut buf).await {
                    Ok(n) => n,
                    Err(_) => break,
                };
                headers.extend_from_slice(&buf[..n]);
                if headers.windows(4).position(|w| w == b"\r\n\r\n").is_some() {
                    let response = b"HTTP/1.1 200 Connection Established\r\n\r\n";
                    let _ = stream.write_all(response).await;
                    break;
                }
            }
        });
    }
}

async fn run_bench(concurrency: usize, connector: Arc<ProxyConnector>) {
    let target: Uri = "https://example.com:443".parse().unwrap();
    let mut handles = vec![];
    for _ in 0..concurrency {
        let c = Arc::clone(&connector);
        let t = target.clone();
        handles.push(tokio::spawn(async move {
            let start = Instant::now();
            let _ = c.connect(&t).await;
            start.elapsed()
        }));
    }

    let mut total_time = std::time::Duration::ZERO;
    let mut success_count = 0;

    for h in handles {
        match h.await {
            Ok(duration) => {
                total_time += duration;
                success_count += 1;
            }
            Err(_) => {}
        }
    }

    if success_count > 0 {
        let avg = total_time / success_count;
        println!("  Concurrency: {} | Avg Time: {:?} | Success: {}/{}", 
                 concurrency, avg, success_count, concurrency);
    }
}

fn main() {
    println!("🚀 Starting ylong_https_proxy Performance Benchmark...");
    
    let rt = Runtime::new().unwrap();
    let port = 19999;
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    
    // Start mock server
    rt.spawn(run_mock_proxy(addr));
    std::thread::sleep(std::time::Duration::from_millis(500));
    println!("✅ Mock Proxy started on port {}", port);

    let proxy_url = Url::parse(&format!("http://127.0.0.1:{}", port)).unwrap();
    let config = ProxyConfig::builder(proxy_url)
        .with_timeout(std::time::Duration::from_secs(5))
        .build();
    let connector = Arc::new(ProxyConnector::new(config));

    println!("🏃 Running benchmarks...");
    for conc in [1, 10, 50, 100] {
        rt.block_on(run_bench(conc, Arc::clone(&connector)));
    }
    println!("✅ Benchmark complete.");
}
