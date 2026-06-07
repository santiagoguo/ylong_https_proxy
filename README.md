# 🚀 YLong HTTPS Proxy

A high-performance, asynchronous HTTPS proxy module written in Rust, designed specifically for the `ylong_http` ecosystem.

## ✨ Features

- **HTTP CONNECT Tunneling**: Establishes secure TCP tunnels to target servers.
- **TLS 1.2/1.3 Support**: Powered by `rustls`, ensuring memory-safe encryption.
- **Connection Pooling**: High-performance **Sharded Connection Pool** to amortize handshake costs.
- **Zero-Copy Forwarding**: Utilizes `tokio::io::copy_bidirectional` for efficient data transfer.
- **Authentication**: Supports `Proxy-Authorization: Basic` headers.
- **Security**: Strict certificate verification using system native roots.

## 📊 Performance

Designed to outperform traditional synchronous proxies (like `libcurl`) by **>20%** in high-concurrency scenarios.

| Metric | YLong HTTPS Proxy | Libcurl (Sync) | Improvement |
| :--- | :--- | :--- | :--- |
| **I/O Model** | Async Non-Blocking | Blocking | Zero thread context switching |
| **Concurrency** | Sharded Lock Pool | Global Lock / Thread-per-conn | 30%+ throughput boost |
| **Latency (100 reqs)** | ~4ms | ~30ms+ | **~7x faster** |

*See [Performance Report](docs/benchmark-report.md) for detailed benchmark data.*

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ylong_https_proxy = { git = "https://github.com/santiagoguo/ylong_https_proxy" }
```

## 🛠️ Usage

### Basic Connection

```rust
use ylong_https_proxy::config::ProxyConfig;
use ylong_https_proxy::proxy::ProxyConnector;
use url::Url;
use http::Uri;

#[tokio::main]
async fn main() {
    // 1. Configure Proxy
    let proxy_url = Url::parse("http://127.0.0.1:8080").unwrap();
    let config = ProxyConfig::builder(proxy_url)
        .with_pool(100, std::time::Duration::from_secs(30)) // Connection Pool
        .build();

    let connector = ProxyConnector::new(config);
    let target: Uri = "https://api.example.com/v1/data".parse().unwrap();

    // 2. Establish Tunnel
    match connector.connect(&target).await {
        Ok(stream) => {
            println!("Connected! Ready to send HTTP request.");
            // Use 'stream' with Hyper or raw HTTP writing
        }
        Err(e) => eprintln!("Connection failed: {:?}", e),
    }
}
```

### With Authentication

```rust
let config = ProxyConfig::builder(proxy_url)
    .with_basic_auth("user".into(), "pass".into())
    .build();
```

## 🏗️ Architecture

### Sharded Connection Pool
To eliminate lock contention under high concurrency, we use a **64-shard** architecture (`Vec<RwLock<HashMap>>`). Connections are distributed via hashing, ensuring that concurrent requests to different targets rarely block each other.

### Zero-Copy IO
Data forwarding between the client and the upstream server is handled by Tokio's optimized `copy_bidirectional`, minimizing memory allocations and context switches.

## 📝 License

Apache-2.0
