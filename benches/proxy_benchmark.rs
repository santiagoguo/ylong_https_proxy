//! Benchmark: ylong_https_proxy connection establishment
//!
//! Measures overhead of `ProxyConnector::connect()` under various concurrency levels
//! using a mock TCP proxy server.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use futures::future::join_all;
use http::Uri;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use url::Url;
use ylong_https_proxy::config::ProxyConfig;
use ylong_https_proxy::proxy::ProxyConnector;

/// A mock proxy server that accepts CONNECT and replies 200 OK
async fn run_mock_proxy(addr: SocketAddr) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            // Read until we get \r\n\r\n (HTTP headers end)
            let mut headers = Vec::new();
            loop {
                let n = match stream.read(&mut buf).await {
                    Ok(n) => n,
                    Err(_) => break,
                };
                headers.extend_from_slice(&buf[..n]);
                if let Some(_pos) = headers.windows(4).position(|w| w == b"\r\n\r\n") {
                    let response = b"HTTP/1.1 200 Connection Established\r\n\r\n";
                    let _ = stream.write_all(response).await;
                    break;
                }
            }
        });
    }
}

fn proxy_connect_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Start mock proxy on a random port
    // We use a channel to signal that the server is ready
    let (tx, rx) = std::sync::mpsc::channel::<SocketAddr>();
    
    // We can't easily pass the channel to the async task and block the main thread.
    // Let's just pick a fixed port.
    let port = 19998;
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    
    rt.spawn(run_mock_proxy(addr));
    
    // Give server a moment to bind
    std::thread::sleep(std::time::Duration::from_millis(200));

    let proxy_url = Url::parse(&format!("http://127.0.0.1:{}", port)).unwrap();
    let config = ProxyConfig::builder(proxy_url)
        .with_timeout(std::time::Duration::from_secs(2))
        .build();
    let connector = ProxyConnector::new(config);

    let mut group = c.benchmark_group("proxy_connect_async");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(2));
    
    for concurrency in [1, 10, 50] {
        group.bench_with_input(BenchmarkId::from_parameter(concurrency), &concurrency, |b, &conc| {
            b.iter(|| {
                rt.block_on(async {
                    let target: Uri = "https://example.com:443".parse().unwrap();
                    let tasks: Vec<_> = (0..conc).map(|_| {
                        let c = &connector;
                        let t = target.clone();
                        async move {
                            let res = c.connect(black_box(&t)).await;
                            if res.is_err() {
                                eprintln!("Connect failed: {:?}", res);
                            }
                        }
                    }).collect();
                    join_all(tasks).await;
                })
            });
        });
    }
    group.finish();
}

criterion_group!(benches, proxy_connect_benchmark);
criterion_main!(benches);
