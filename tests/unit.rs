use std::path::PathBuf;
use std::time::Duration;
use url::Url;
use ylong_https_proxy::config::{ProxyConfig, TlsVerifyMode, TlsVersion};
use ylong_https_proxy::ProxyAuth;

#[test]
fn test_config_builder() {
    let url = Url::parse("http://127.0.0.1:8080").unwrap();
    let config = ProxyConfig::builder(url)
        .with_basic_auth("user".into(), "pass".into())
        .with_timeout(Duration::from_secs(10))
        .with_pool(32, Duration::from_secs(120))
        .build();

    assert!(matches!(config.auth, ProxyAuth::Basic { .. }));
    assert_eq!(config.connect_timeout.as_secs(), 10);
    assert_eq!(config.pool_max_size, 32);
}

#[test]
fn test_tls_verify_mode() {
    let url = Url::parse("http://127.0.0.1:8080").unwrap();

    // Default should be Strict
    let config = ProxyConfig::builder(url.clone()).build();
    assert!(matches!(
        config.tls_config.verify_mode,
        TlsVerifyMode::Strict
    ));

    // Can set to None
    let config = ProxyConfig::builder(url.clone())
        .with_verify_none()
        .build();
    assert!(matches!(
        config.tls_config.verify_mode,
        TlsVerifyMode::None
    ));
}

#[test]
fn test_tls_version_config() {
    let url = Url::parse("http://127.0.0.1:8080").unwrap();
    let config = ProxyConfig::builder(url)
        .with_min_tls_version(TlsVersion::Tlsv12)
        .with_max_tls_version(TlsVersion::Tlsv13)
        .build();

    assert_eq!(config.tls_config.min_tls_version, TlsVersion::Tlsv12);
    assert_eq!(config.tls_config.max_tls_version, TlsVersion::Tlsv13);
}

#[test]
fn test_cipher_suites_config() {
    let url = Url::parse("http://127.0.0.1:8080").unwrap();
    let suites = vec![
        "ECDHE-ECDSA-AES256-GCM-SHA384".to_string(),
        "ECDHE-RSA-AES256-GCM-SHA384".to_string(),
    ];
    let config = ProxyConfig::builder(url)
        .with_cipher_suites(suites.clone())
        .build();

    assert_eq!(config.tls_config.cipher_suites, suites);
}

#[test]
fn test_mutual_tls_config() {
    let url = Url::parse("http://127.0.0.1:8080").unwrap();
    let config = ProxyConfig::builder(url)
        .with_mutual_tls(
            PathBuf::from("/path/to/client.crt"),
            PathBuf::from("/path/to/client.key"),
        )
        .build();

    assert_eq!(
        config.tls_config.client_cert_file,
        Some(PathBuf::from("/path/to/client.crt"))
    );
    assert_eq!(
        config.tls_config.client_key_file,
        Some(PathBuf::from("/path/to/client.key"))
    );
}

#[test]
fn test_ca_cert_config() {
    let url = Url::parse("http://127.0.0.1:8080").unwrap();
    let config = ProxyConfig::builder(url)
        .with_ca_file(PathBuf::from("/path/to/ca.pem"))
        .build();

    assert_eq!(
        config.tls_config.ca_cert_files,
        vec![PathBuf::from("/path/to/ca.pem")]
    );
}
