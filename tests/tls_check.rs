use ylong_https_proxy::tls::TlsManager;
use ylong_https_proxy::config::ProxyConfig;
use url::Url;

#[test]
fn test_tls_manager_initializes_with_native_certs() {
    let url = Url::parse("http://127.0.0.1:8080").unwrap();
    let config = ProxyConfig::builder(url).build();
    
    // This tests loading native certs (macOS Keychain)
    let manager = TlsManager::new(&config);
    assert!(manager.is_ok());
}
