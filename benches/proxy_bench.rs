use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;
use ylong_https_proxy::{HttpsProxyClient, ProxyConfig};
use url::Url;
use http::Uri;

/// Performance Benchmark: ylong_https_proxy vs theoretical libcurl
fn proxy_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Config: Use a mock local proxy to simulate benchmark
    let proxy_url = Url::parse("http://127.0.0.1:8080").unwrap();
    let config = ProxyConfig::new(proxy_url);
    
    let client = HttpsProxyClient::new(config).unwrap();

    c.bench_function("ylong_https_proxy_connect", |b| {
        b.iter(|| {
            rt.block_on(async {
                let target: Uri = "https://example.com".parse().unwrap();
                let _ = client.connect(black_box(&target)).await;
            })
        })
    });
}

criterion_group!(benches, proxy_benchmark);
criterion_main!(benches);
