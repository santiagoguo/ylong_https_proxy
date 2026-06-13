# 🚀 YLong HTTPS Proxy

A high-performance, asynchronous HTTPS proxy module written in Rust, designed specifically for the `ylong_http` ecosystem.

**TLS Backend: OpenSSL 3.x** (as required by competition spec)

## ✨ Features

- **HTTP CONNECT Tunneling**: Establishes secure TCP tunnels to target servers.
- **OpenSSL TLS 1.2/1.3**: Powered by OpenSSL for memory-safe, production-grade encryption.
- **Server Certificate Verification**: Strict verification using system native roots or custom CA chains.
- **Mutual TLS (Bidirectional Authentication)**: Supports client certificate + private key for two-way TLS authentication.
- **Custom Cipher Suites**: Configurable TLS cipher suite selection for compliance requirements.
- **TLS Version Constraints**: Configurable minimum/maximum TLS versions.
- **Connection Pooling**: High-performance **64-Sharded Connection Pool** to amortize handshake costs.
- **Zero-Copy Forwarding**: Utilizes `tokio::io::copy_bidirectional` for efficient data transfer.
- **Authentication**: Supports `Proxy-Authorization: Basic` headers.
- **Extensible Protocol Architecture**: Trait-based `ProxyProtocol` abstraction for adding new protocols (SOCKS4/5, custom).

## 📊 Performance

Designed to outperform traditional synchronous proxies (like `libcurl`) by **>20%** in high-concurrency scenarios.

| Metric | YLong HTTPS Proxy | Libcurl (Sync) | Improvement |
| :--- | :--- | :--- | :--- |
| **TLS Backend** | OpenSSL 3.x (async) | OpenSSL (sync) | Same security, better throughput |
| **I/O Model** | Async Non-Blocking | Blocking | Zero thread context switching |
| **Concurrency** | 64-Shard Lock Pool | Global Lock / Thread-per-conn | 30%+ throughput boost |
| **Latency (100 reqs)** | ~4.04 ms | ~30 ms+ | **~9x faster** |

*See [Performance Report](docs/benchmark-report.md) for detailed benchmark data.*

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ylong_https_proxy = { git = "https://github.com/santiagoguo/ylong_https_proxy" }
```

### Build Requirements (macOS)

```bash
# Install OpenSSL via Homebrew
brew install openssl@3

# Build with OpenSSL paths
export OPENSSL_DIR=/opt/homebrew/opt/openssl@3
export OPENSSL_LIB_DIR=/opt/homebrew/opt/openssl@3/lib
export OPENSSL_INCLUDE_DIR=/opt/homebrew/opt/openssl@3/include
cargo build --release
```

## 🛠️ Usage

### Basic Connection

```rust
use ylong_https_proxy::config::ProxyConfig;
use ylong_https_proxy::pool::ProxyClient;
use url::Url;
use http::Uri;

#[tokio::main]
async fn main() {
    // 1. Configure Proxy
    let proxy_url = Url::parse("http://127.0.0.1:8080").unwrap();
    let config = ProxyConfig::builder(proxy_url)
        .with_pool(100, std::time::Duration::from_secs(30))
        .build();

    let client = ProxyClient::new(config).unwrap();
    let target: Uri = "https://api.example.com/v1/data".parse().unwrap();

    // 2. Establish TLS Tunnel
    match client.connect(&target).await {
        Ok(stream) => {
            println!("Connected with TLS! Ready to send HTTP request.");
        }
        Err(e) => eprintln!("Connection failed: {e:?}"),
    }
}
```

### With Authentication

```rust
let config = ProxyConfig::builder(proxy_url)
    .with_basic_auth("user".into(), "pass".into())
    .build();
```

### With Mutual TLS (Bidirectional Authentication)

```rust
let config = ProxyConfig::builder(proxy_url)
    .with_mutual_tls(
        std::path::PathBuf::from("/path/to/client.crt"),
        std::path::PathBuf::from("/path/to/client.key"),
    )
    .build();
```

### With Custom CA Certificate

```rust
let config = ProxyConfig::builder(proxy_url)
    .with_ca_file(std::path::PathBuf::from("/path/to/ca.pem"))
    .build();
```

### With Custom Cipher Suites

```rust
let config = ProxyConfig::builder(proxy_url)
    .with_cipher_suites(vec![
        "ECDHE-ECDSA-AES256-GCM-SHA384".to_string(),
        "ECDHE-RSA-AES256-GCM-SHA384".to_string(),
    ])
    .build();
```

### With TLS Version Constraints

```rust
use ylong_https_proxy::config::TlsVersion;

let config = ProxyConfig::builder(proxy_url)
    .with_min_tls_version(TlsVersion::Tlsv12)
    .with_max_tls_version(TlsVersion::Tlsv13)
    .build();
```

## 🏗️ Architecture

### Sharded Connection Pool
To eliminate lock contention under high concurrency, this module uses a **64-shard** architecture (`Vec<RwLock<HashMap>>`). Connections are distributed via hashing, ensuring that concurrent requests to different targets rarely block each other.

### Extensible Protocol Layer
The `ProxyProtocol` trait allows adding new proxy protocols without modifying existing code:

```rust
use ylong_https_proxy::proxy::{ProxyProtocol, ProxyProtocolType};

// Implement for SOCKS5, custom protocols, etc.
#[async_trait]
impl ProxyProtocol for MySocks5Proxy {
    fn protocol_type(&self) -> ProxyProtocolType {
        ProxyProtocolType::Socks5
    }

    async fn establish_tunnel(&self, target: &Uri) -> ProxyResult<TcpStream> {
        // SOCKS5 handshake implementation
    }
}
```

### Zero-Copy IO
Data forwarding between the client and the upstream server is handled by Tokio's optimized `copy_bidirectional`, minimizing memory allocations and context switches.

## 🧪 Testing

```bash
cargo test
cargo clippy
```

## 📋 Competition Compliance

| Requirement | Status | Details |
|---|---|---|
| OpenSSL TLS Backend | ✅ | OpenSSL 3.x via `openssl` + `tokio-openssl` crates |
| TLS Single-way Auth | ✅ | Server cert verification with system/custom CA |
| TLS Mutual Auth | ✅ | Client cert + private key configuration |
| TLS Config (ciphers, versions) | ✅ | Custom cipher suites, min/max TLS version |
| Modular Proxy Design | ✅ | Separate proxy/tls/pool/config modules |
| Protocol Extensibility | ✅ | `ProxyProtocol` trait + `ProtocolRegistry` |
| Performance >20% vs libcurl | ✅ | ~9x improvement at 100 concurrency |

## 👤 Author

Developed by [santiagoguo](https://github.com/santiagoguo).

## 📝 License

Apache-2.0
