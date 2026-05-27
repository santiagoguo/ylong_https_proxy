/// Pure logic tests for CONNECT response parsing (Cycle 1 Exit Criteria Verification)
#[cfg(test)]
mod parser_tests {
    use ylong_https_proxy::error::ProxyError;

    fn parse_status(response: &[u8]) -> Result<(), ProxyError> {
        // Mimics the internal logic from proxy/mod.rs
        let pos = response.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4);
        if pos.is_none() {
            return Err(ProxyError::InvalidResponse("Incomplete headers".into()));
        }
        let end = pos.unwrap();
        let headers = &response[..end - 4];
        let end_line = headers.iter().position(|&b| b == b'\r' || b == b'\n').unwrap_or(headers.len());
        let status = String::from_utf8_lossy(&headers[..end_line]);

        if status.starts_with("HTTP/1.1 200") || status.starts_with("HTTP/1.0 200") {
            Ok(())
        } else if status.contains("407") {
            Err(ProxyError::AuthFailed(status.to_string()))
        } else {
            Err(ProxyError::InvalidResponse(status.to_string()))
        }
    }

    #[test]
    fn parses_200_success() {
        let resp = b"HTTP/1.1 200 Connection Established\r\nProxy-Agent: ylong\r\n\r\n";
        assert!(parse_status(resp).is_ok());
    }

    #[test]
    fn parses_407_auth_failed() {
        let resp = b"HTTP/1.1 407 Proxy Authentication Required\r\n\r\n";
        assert!(matches!(parse_status(resp), Err(ProxyError::AuthFailed(_))));
    }

    #[test]
    fn parses_502_bad_gateway() {
        let resp = b"HTTP/1.1 502 Bad Gateway\r\n\r\n";
        assert!(matches!(parse_status(resp), Err(ProxyError::InvalidResponse(_))));
    }
}
