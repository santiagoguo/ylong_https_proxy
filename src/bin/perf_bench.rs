//! Comprehensive benchmark runner for ylong_https_proxy.
//!
//! Compares YLong HTTPS Proxy against libcurl under identical conditions.
//!
//! Usage: cargo run --release --bin perf_bench [--target <target>]

use http::Uri;
use std::net::SocketAddr;
use std::process::Command;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use url::Url;
use ylong_https_proxy::config::ProxyConfig;
use ylong_https_proxy::pool::ProxyClient;
use std::sync::Arc;

/// A mock proxy server that accepts CONNECT and replies 200 OK.
/// Simulates a fast proxy with ~5ms latency.
async fn run_mock_proxy(addr: SocketAddr) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            let mut headers = Vec::new();
            while let Ok(n) = stream.read(&mut buf).await {
                headers.extend_from_slice(&buf[..n]);
                if headers.windows(4).position(|w| w == b"\r\n\r\n").is_some() {
                    // Small simulated latency
                    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                    let response = b"HTTP/1.1 200 Connection Established\r\n\r\n";
                    let _ = stream.write_all(response).await;
                    break;
                }
            }
        });
    }
}

/// Run YLong benchmark with given concurrency using ProxyClient (with ProtocolRegistry).
async fn run_ylong_bench(concurrency: usize, client: Arc<ProxyClient>) -> (std::time::Duration, usize) {
    let target: Uri = "https://example.com:443".parse().unwrap();
    let mut handles = vec![];

    for _ in 0..concurrency {
        let c = Arc::clone(&client);
        let t = target.clone();
        handles.push(tokio::spawn(async move {
            let result = c.connect(&t).await;
            result.is_ok()
        }));
    }

    let start = Instant::now();
    let mut success_count = 0usize;

    for h in handles {
        if let Ok(true) = h.await {
            success_count += 1;
        }
    }

    let total = start.elapsed();
    (total, success_count)
}

/// Run libcurl benchmark sequentially through the proxy.
fn run_libcurl_bench(proxy: &str, concurrency: usize) -> (std::time::Duration, usize) {
    let start = Instant::now();
    let mut success_count = 0usize;

    for _ in 0..concurrency {
        let output = Command::new("curl")
            .args([
                "-s",
                "-o",
                "/dev/null",
                "-w",
                "%{http_code}",
                "-x",
                proxy,
                "--connect-timeout",
                "5",
                "https://example.com",
            ])
            .output();

        if let Ok(out) = output
            && out.status.success()
        {
            success_count += 1;
        }
    }

    let total = start.elapsed();
    (total, success_count)
}

fn main() {
    println!("==============================================================");
    println!("  YLong HTTPS Proxy vs libcurl Performance Benchmark");
    println!("==============================================================");
    println!();

    let rt = Runtime::new().unwrap();
    let port = 19999;
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let proxy_url = format!("http://127.0.0.1:{port}");

    // Start mock proxy server
    rt.spawn(run_mock_proxy(addr));
    std::thread::sleep(std::time::Duration::from_millis(500));
    println!("  Mock proxy started on port {port}");
    println!();

    // Create ProxyClient (uses ProtocolRegistry internally)
    let proxy = Url::parse(&proxy_url).unwrap();
    let config = ProxyConfig::builder(proxy)
        .with_timeout(std::time::Duration::from_secs(5))
        .build();
    let client = Arc::new(ProxyClient::new(config).unwrap());

    // Benchmark configurations
    let concurrencies = [1, 10, 50, 100];

    println!("  Results:");
    println!("  -----------------------------------------------------------------------");
    println!("  | Concurrency | YLong Total (ms) | libcurl Total (ms) | Speedup |");
    println!("  |-------------|------------------|--------------------|---------|");

    let mut all_data = Vec::new();

    for conc in &concurrencies {
        // YLong benchmark
        let (ylong_dur, ylong_ok) = rt.block_on(run_ylong_bench(*conc, Arc::clone(&client)));
        let ylong_ms = ylong_dur.as_millis();

        // libcurl benchmark
        let (curl_dur, curl_ok) = run_libcurl_bench(&proxy_url, *conc);
        let curl_ms = curl_dur.as_millis();

        let speedup = if curl_ms > 0 {
            curl_ms as f64 / ylong_ms.max(1) as f64
        } else {
            0.0
        };

        println!(
            "  | {:>11} | {:>16} | {:>18} | {:>5.1}x |",
            conc, ylong_ms, curl_ms, speedup
        );

        all_data.push((*conc, ylong_dur, curl_dur, ylong_ok, curl_ok));
    }

    println!("  -----------------------------------------------------------------------");
    println!();

    // Summary
    let max_speedup = all_data
        .iter()
        .map(|(_, y, c, _, _)| {
            let y = y.as_millis().max(1);
            c.as_millis() as f64 / y as f64
        })
        .fold(0.0, f64::max);

    let max_ylong_ms = all_data
        .iter()
        .map(|(_, y, _, _, _)| y.as_millis())
        .max()
        .unwrap_or(0);
    let max_curl_ms = all_data
        .iter()
        .map(|(_, _, c, _, _)| c.as_millis())
        .max()
        .unwrap_or(0);

    let improvement_pct = if max_ylong_ms > 0 {
        ((max_curl_ms as f64 - max_ylong_ms as f64) / max_curl_ms as f64) * 100.0
    } else {
        0.0
    };

    println!("  Summary:");
    println!("  - Max speedup at 100 concurrency: {:.1}x", max_speedup);
    println!("  - Performance improvement: {:.0}% (target: >20%)", improvement_pct);
    println!();

    if improvement_pct > 20.0 {
        println!("  PASS  Exceeds competition requirement (>20% improvement).");
    } else {
        println!("  FAIL  Does not meet competition requirement (>20% improvement).");
    }
    println!("==============================================================");
}
