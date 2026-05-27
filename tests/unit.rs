use ylong_https_proxy::config::ProxyConfig;
use url::Url;
use std::time::Duration;

#[test]
fn test_config_builder() {
    let url = Url::parse("http://127.0.0.1:8080").unwrap();
    let config = ProxyConfig::builder(url)
        .with_basic_auth("user".into(), "pass".into())
        .with_timeout(Duration::from_secs(10))
        .with_pool(32, Duration::from_secs(120))
        .build();

    assert!(matches!(config.auth, ylong_https_proxy::ProxyAuth::Basic { .. }));
    assert_eq!(config.connect_timeout.as_secs(), 10);
    assert_eq!(config.pool_max_size, 32);
}
