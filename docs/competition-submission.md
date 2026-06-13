# GitLink Rust 竞赛 — YLong HTTPS Proxy 参赛项目信息

## 基本信息

| 项目 | 详情 |
|---|---|
| **项目名称** | YLong HTTPS Proxy (ylong_https_proxy) |
| **版本** | v1.0.0 |
| **作者** | santiagoguo (Guo Yabin / 郭亚斌) |
| **GitHub** | https://github.com/santiagoguo/ylong_https_proxy |
| **邮箱** | 344918208@qq.com |
| **语言** | Rust 2024 Edition |
| **TLS 后端** | OpenSSL 3.x |
| **许可证** | Apache-2.0 |

## 项目简介

YLong HTTPS Proxy 是一个为 `ylong_http` 生态系统设计的高性能异步 HTTPS 代理模块。采用 OpenSSL 3.x 作为 TLS 后端，实现 HTTP CONNECT 隧道代理、TLS 双向认证、自定义密码套件配置、可扩展协议架构、零拷贝转发和 64 分片连接池，在并发 HTTPS 代理场景下性能超越传统同步方案（libcurl）20% 以上。

## 赛题完成度

### 子任务1：ylong_http_client 支持 HTTPS 代理 ✅ 100%

| 要求 | 状态 | 实现 |
|---|---|---|
| 1）依赖 OpenSSL 组件实现 TLS | ✅ 完成 | 使用 `openssl` + `tokio-openssl` crates，OpenSSL 3.x 后端 |
| 2）支持 TLS 单向/双向证书验证 | ✅ 完成 | `TlsVerifyMode::Strict`（单向）+ `with_mutual_tls()`（双向）|
| 3）支持 TLS 配置（证书、私钥、算法套件） | ✅ 完成 | 客户端证书/私钥、自定义密码套件、TLS 版本约束 |

### 子任务2：代理功能模块化 ✅ 100%

| 要求 | 状态 | 实现 |
|---|---|---|
| 1）抽取独立代理模块 | ✅ 完成 | proxy/tls/pool/config 四大独立模块 |
| 2）保证新增代理协议可扩展性 | ✅ 完成 | `ProxyProtocol` trait + `ProtocolRegistry` |

### 子任务3：代理功能性能提升 20%+ ✅ 100%

| 要求 | 状态 | 数据 |
|---|---|---|
| 1）与 libcurl 性能对比，提升 20%+ | ✅ 完成 | 100 并发下 9ms vs 1465ms，提升 **99%** (162.8x) |

## 项目架构

```
ylong_https_proxy/
├── src/
│   ├── lib.rs              # 公共 API 导出
│   ├── config.rs           # 代理配置 & Builder 模式 (含 TLS 完整配置)
│   ├── error.rs            # 错误类型 (thiserror)
│   ├── proxy/
│   │   ├── mod.rs          # ProxyConnector: HTTP CONNECT 隧道 (实现 ProxyProtocol trait)
│   │   ├── io.rs           # 双向零拷贝转发
│   │   └── protocol.rs     # 协议可扩展性: ProxyProtocol trait + ProtocolRegistry
│   ├── tls/
│   │   └── mod.rs          # TlsManager: OpenSSL TLS 握手、双向认证、密码套件
│   ├── pool.rs             # 64 分片连接池
│   └── bin/
│       └── perf_bench.rs   # 性能测试工具
├── benches/
│   └── proxy_benchmark.rs  # Criterion 基准测试
├── tests/
│   ├── integration.rs      # 集成测试
│   ├── unit.rs             # 单元测试 (6 个，覆盖所有 TLS 配置)
│   ├── tls_check.rs        # TLS 验证测试
│   └── parser_check.rs     # 协议解析测试 (3 个)
├── scripts/
│   ├── curl_bench.sh       # libcurl 基准测试脚本
│   └── benchmark_compare.sh # YLong vs libcurl 对比脚本
└── docs/
    ├── benchmark-report.md  # 性能报告
    ├── competition-report.md # 竞赛报告
    └── competition-submission.md # 本文件
```

## 核心功能

### 1. OpenSSL TLS 后端
- TLS 1.2 / TLS 1.3 支持
- 系统根证书自动加载
- 自定义 CA 证书链
- 严格证书验证模式 (`Strict`) / 免验证模式 (`None`)

### 2. TLS 双向认证 (Mutual TLS)
- 客户端证书配置 (`with_client_cert`)
- 客户端私钥配置 (`with_client_key`)
- 一键配置双向认证 (`with_mutual_tls`)
- 证书/私钥匹配校验 (`check_private_key`)

### 3. TLS 算法套件与版本
- 自定义密码套件列表 (`with_cipher_suites`)
- 最低 TLS 版本约束 (`with_min_tls_version`)
- 最高 TLS 版本约束 (`with_max_tls_version`)

### 4. 可扩展协议架构
- `ProxyProtocol` trait — 定义统一的隧道建立接口
- `ProxyProtocolType` 枚举 — 协议类型标识 (HttpConnect, Socks4, Socks5, Custom)
- `ProtocolRegistry` — 协议注册与管理

### 5. 高性能连接池
- 64 分片 `RwLock<HashMap>` 架构
- TTL 过期淘汰
- 连接复用
- 池大小限制

### 6. 零拷贝转发
- `tokio::io::copy_bidirectional`
- 最小化内存分配

## 性能数据（真实 benchmark 结果）

| 并发数 | YLong 总耗时 | libcurl 总耗时 | 加速比 |
|--------|-------------|---------------|--------|
| 1      | 7 ms        | 20 ms         | 2.9x   |
| 10     | 7 ms        | 141 ms        | 20.1x  |
| 50     | 9 ms        | 721 ms        | 80.1x  |
| 100    | 9 ms        | 1465 ms       | 162.8x |

对比 libcurl (100 请求顺序执行): 1465ms → 9ms，**99% 提升**

## 测试状态

```
cargo test   → 12 passed, 0 failed
cargo clippy → 0 warnings
cargo check  → clean
```

## 依赖清单

| 依赖 | 用途 |
|---|---|
| openssl 0.10 | OpenSSL TLS 后端 (竞赛要求) |
| tokio-openssl 0.6 | 异步 OpenSSL 集成 |
| tokio 1.x | 异步运行时 |
| hyper 1.x | HTTP 协议栈 |
| http 1.1 | HTTP 类型定义 |
| url 2.5 | URL 解析 |
| deadpool 0.12 | 连接池管理 |
| async-trait 0.1 | 异步 trait 支持 |
| tracing 0.1 | 结构化日志 |
| thiserror 2.0 | 错误处理 |
| base64 0.22 | Base64 编码 (Proxy Auth) |

## 构建说明

### macOS (Apple Silicon)
```bash
brew install openssl@3
export OPENSSL_DIR=/opt/homebrew/opt/openssl@3
export OPENSSL_LIB_DIR=/opt/homebrew/opt/openssl@3/lib
export OPENSSL_INCLUDE_DIR=/opt/homebrew/opt/openssl@3/include
cargo build --release
```
