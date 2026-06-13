use http::Uri;
use url::Url;
use ylong_https_proxy::config::ProxyConfig;
use ylong_https_proxy::pool::ProxyClient;

#[tokio::test]
async fn test_pool_creates_new_connection_when_empty() {
    let proxy_url = Url::parse("http://127.0.0.1:8080").unwrap();
    let config = ProxyConfig::builder(proxy_url).build();
    let client = ProxyClient::new(config).unwrap();

    let target: Uri = "https://example.com".parse().unwrap();
    // Expect error because no proxy is actually running on 8080,
    // but this tests the flow enters the connector.
    let res = client.connect(&target).await;
    assert!(res.is_err()); // IO Error (Connection Refused)
}
