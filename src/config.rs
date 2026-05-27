use url::Url;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum ProxyAuth {
    None,
    Basic { username: String, password: String },
}

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub proxy_url: Url,
    pub auth: ProxyAuth,
    pub connect_timeout: Duration,
    pub pool_max_size: usize,
    pub pool_idle_timeout: Duration,
}

impl ProxyConfig {
    pub fn builder(proxy_url: Url) -> ProxyConfigBuilder {
        ProxyConfigBuilder {
            proxy_url,
            auth: ProxyAuth::None,
            connect_timeout: Duration::from_secs(5),
            pool_max_size: 16,
            pool_idle_timeout: Duration::from_secs(60),
        }
    }
}

pub struct ProxyConfigBuilder {
    proxy_url: Url,
    auth: ProxyAuth,
    connect_timeout: Duration,
    pool_max_size: usize,
    pool_idle_timeout: Duration,
}

impl ProxyConfigBuilder {
    pub fn with_basic_auth(mut self, user: String, pass: String) -> Self {
        self.auth = ProxyAuth::Basic { username: user, password: pass };
        self
    }
    pub fn with_timeout(mut self, dur: Duration) -> Self {
        self.connect_timeout = dur;
        self
    }
    pub fn with_pool(mut self, max_size: usize, idle: Duration) -> Self {
        self.pool_max_size = max_size;
        self.pool_idle_timeout = idle;
        self
    }
    pub fn build(self) -> ProxyConfig {
        ProxyConfig {
            proxy_url: self.proxy_url,
            auth: self.auth,
            connect_timeout: self.connect_timeout,
            pool_max_size: self.pool_max_size,
            pool_idle_timeout: self.pool_idle_timeout,
        }
    }
}
