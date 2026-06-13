use std::path::PathBuf;
use std::time::Duration;
use url::Url;

// =============================================================================
// Proxy Authentication
// =============================================================================

#[derive(Debug, Clone)]
pub enum ProxyAuth {
    None,
    Basic { username: String, password: String },
}

// =============================================================================
// TLS Verification Mode
// =============================================================================

#[derive(Debug, Clone)]
pub enum TlsVerifyMode {
    /// Strict: verify server certificate against CA chain (default)
    Strict,
    /// No verification: accept any certificate (for testing only)
    None,
}

// =============================================================================
// TLS Configuration for the proxy server connection
// =============================================================================

#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Verification mode (Strict or None)
    pub verify_mode: TlsVerifyMode,

    /// Paths to CA certificate files or directories
    pub ca_cert_files: Vec<PathBuf>,

    /// --- Mutual TLS (Bidirectional) ---

    /// Client certificate file (PEM format)
    pub client_cert_file: Option<PathBuf>,

    /// Client private key file (PEM format)
    pub client_key_file: Option<PathBuf>,

    /// Custom cipher suites (e.g., "ECDHE-ECDSA-AES256-GCM-SHA384")
    /// If empty, uses OpenSSL defaults
    pub cipher_suites: Vec<String>,

    /// Minimum TLS version (default: TLS 1.2)
    pub min_tls_version: TlsVersion,

    /// Maximum TLS version (default: TLS 1.3)
    pub max_tls_version: TlsVersion,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            verify_mode: TlsVerifyMode::Strict,
            ca_cert_files: Vec::new(),
            client_cert_file: None,
            client_key_file: None,
            cipher_suites: Vec::new(),
            min_tls_version: TlsVersion::Tlsv12,
            max_tls_version: TlsVersion::Tlsv13,
        }
    }
}

// =============================================================================
// TLS Version
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsVersion {
    Tlsv10,
    Tlsv11,
    Tlsv12,
    Tlsv13,
}

// =============================================================================
// Proxy Configuration (Builder Pattern)
// =============================================================================

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub proxy_url: Url,
    pub auth: ProxyAuth,
    pub connect_timeout: Duration,
    pub pool_max_size: usize,
    pub pool_idle_timeout: Duration,
    pub tls_config: TlsConfig,
}

impl ProxyConfig {
    pub fn builder(proxy_url: Url) -> ProxyConfigBuilder {
        ProxyConfigBuilder {
            proxy_url,
            auth: ProxyAuth::None,
            connect_timeout: Duration::from_secs(5),
            pool_max_size: 16,
            pool_idle_timeout: Duration::from_secs(60),
            tls_config: TlsConfig::default(),
        }
    }
}

pub struct ProxyConfigBuilder {
    proxy_url: Url,
    auth: ProxyAuth,
    connect_timeout: Duration,
    pool_max_size: usize,
    pool_idle_timeout: Duration,
    tls_config: TlsConfig,
}

impl ProxyConfigBuilder {
    // --- Auth ---
    pub fn with_basic_auth(mut self, user: String, pass: String) -> Self {
        self.auth = ProxyAuth::Basic {
            username: user,
            password: pass,
        };
        self
    }

    // --- Connection ---
    pub fn with_timeout(mut self, dur: Duration) -> Self {
        self.connect_timeout = dur;
        self
    }

    // --- Pool ---
    pub fn with_pool(mut self, max_size: usize, idle: Duration) -> Self {
        self.pool_max_size = max_size;
        self.pool_idle_timeout = idle;
        self
    }

    // --- TLS Verification Mode ---
    /// Set TLS verification mode (Strict or None)
    pub fn with_verify_mode(mut self, mode: TlsVerifyMode) -> Self {
        self.tls_config.verify_mode = mode;
        self
    }

    /// Disable TLS verification (for testing only)
    pub fn with_verify_none(mut self) -> Self {
        self.tls_config.verify_mode = TlsVerifyMode::None;
        self
    }

    // --- CA Certificates ---
    /// Add a CA certificate file or directory for server verification
    pub fn with_ca_file(mut self, path: PathBuf) -> Self {
        self.tls_config.ca_cert_files.push(path);
        self
    }

    // --- Mutual TLS (Bidirectional Authentication) ---
    /// Set client certificate for mutual TLS (bidirectional authentication)
    pub fn with_client_cert(mut self, cert_path: PathBuf) -> Self {
        self.tls_config.client_cert_file = Some(cert_path);
        self
    }

    /// Set client private key for mutual TLS (bidirectional authentication)
    pub fn with_client_key(mut self, key_path: PathBuf) -> Self {
        self.tls_config.client_key_file = Some(key_path);
        self
    }

    /// Set both client certificate and key for mutual TLS
    pub fn with_mutual_tls(mut self, cert_path: PathBuf, key_path: PathBuf) -> Self {
        self.tls_config.client_cert_file = Some(cert_path);
        self.tls_config.client_key_file = Some(key_path);
        self
    }

    // --- Cipher Suites ---
    /// Set custom cipher suites for TLS connections
    pub fn with_cipher_suites(mut self, suites: Vec<String>) -> Self {
        self.tls_config.cipher_suites = suites;
        self
    }

    // --- TLS Version ---
    /// Set minimum TLS version
    pub fn with_min_tls_version(mut self, version: TlsVersion) -> Self {
        self.tls_config.min_tls_version = version;
        self
    }

    /// Set maximum TLS version
    pub fn with_max_tls_version(mut self, version: TlsVersion) -> Self {
        self.tls_config.max_tls_version = version;
        self
    }

    // --- Build ---
    pub fn build(self) -> ProxyConfig {
        ProxyConfig {
            proxy_url: self.proxy_url,
            auth: self.auth,
            connect_timeout: self.connect_timeout,
            pool_max_size: self.pool_max_size,
            pool_idle_timeout: self.pool_idle_timeout,
            tls_config: self.tls_config,
        }
    }
}
