use crate::config::{ProxyConfig, TlsVerifyMode, TlsVersion};
use crate::error::{ProxyError, ProxyResult};
use openssl::error::ErrorStack;
use openssl::ssl::{SslConnector, SslFiletype, SslMethod, SslVerifyMode};
use openssl::x509::X509;
use std::fs;
use std::path::Path;
use tokio::net::TcpStream;
use tokio_openssl::SslStream;
use tracing::{info, warn};

/// Manages TLS configuration and handshakes using OpenSSL.
///
/// Supports:
/// - Server certificate verification (unidirectional)
/// - Mutual TLS / bidirectional authentication (client cert + private key)
/// - Custom CA certificate chains
/// - Custom cipher suites
/// - TLS version constraints
pub struct TlsManager {
    connector: SslConnector,
}

impl TlsManager {
    /// Create a new TLS manager with the given proxy configuration.
    ///
    /// Uses OpenSSL for all TLS operations as required by the competition spec.
    pub fn new(config: &ProxyConfig) -> ProxyResult<Self> {
        let mut builder =
            SslConnector::builder(SslMethod::tls()).map_err(map_ssl_error)?;

        let tls = &config.tls_config;

        // --- TLS Version Constraints ---
        set_min_tls_version(&mut builder, tls.min_tls_version)?;
        set_max_tls_version(&mut builder, tls.max_tls_version)?;

        // --- Cipher Suites ---
        if !tls.cipher_suites.is_empty() {
            let cipher_string = tls.cipher_suites.join(":");
            builder
                .set_cipher_list(&cipher_string)
                .map_err(|e| {
                    ProxyError::TlsError(format!("Invalid cipher suite list: {e}"))
                })?;
            info!("Custom cipher suites configured: {cipher_string}");
        }

        // --- Server Certificate Verification ---
        match &tls.verify_mode {
            TlsVerifyMode::Strict => {
                builder.set_verify(SslVerifyMode::PEER);

                // Load custom CA certificates
                for ca_path in &tls.ca_cert_files {
                    load_ca_path(&mut builder, ca_path)?;
                }

                // Also load system CA certificates if no custom CAs provided
                if tls.ca_cert_files.is_empty() {
                    builder
                        .set_default_verify_paths()
                        .map_err(|e| {
                            ProxyError::TlsError(format!(
                                "Failed to load system CA certs: {e}"
                            ))
                        })?;
                }

                info!("TLS verification enabled (Strict mode), CA certs loaded");
            }
            TlsVerifyMode::None => {
                builder.set_verify(SslVerifyMode::NONE);
                warn!("TLS verification DISABLED (None mode) — not recommended for production");
            }
        }

        // --- Mutual TLS (Bidirectional Authentication) ---
        if let (Some(cert_path), Some(key_path)) =
            (&tls.client_cert_file, &tls.client_key_file)
        {
            builder
                .set_certificate_file(cert_path, SslFiletype::PEM)
                .map_err(|e| {
                    ProxyError::TlsError(format!(
                        "Failed to load client certificate from {}: {e}",
                        cert_path.display()
                    ))
                })?;

            builder
                .set_private_key_file(key_path, SslFiletype::PEM)
                .map_err(|e| {
                    ProxyError::TlsError(format!(
                        "Failed to load client private key from {}: {e}",
                        key_path.display()
                    ))
                })?;

            builder
                .check_private_key()
                .map_err(|e| {
                    ProxyError::TlsError(format!("Client cert/key mismatch: {e}"))
                })?;

            info!(
                "Mutual TLS configured: cert={}, key={}",
                cert_path.display(),
                key_path.display()
            );
        }

        Ok(Self {
            connector: builder.build(),
        })
    }

    /// Upgrade a raw TCP stream to a TLS stream by performing the OpenSSL handshake.
    ///
    /// Uses SNI (Server Name Indication) with the target domain name.
    pub async fn connect(
        &self,
        domain: &str,
        stream: TcpStream,
    ) -> ProxyResult<SslStream<TcpStream>> {
        info!("Starting OpenSSL TLS handshake for {domain}");

        // Build an SSL session object with SNI
        let ssl = self
            .connector
            .configure()
            .map_err(map_ssl_error)?
            .into_ssl(domain)
            .map_err(map_ssl_error)?;

        // Create the SslStream and perform the handshake
        let mut ssl_stream =
            SslStream::new(ssl, stream).map_err(|e| {
                ProxyError::TlsError(format!("Failed to create SSL stream: {e}"))
            })?;

        // Perform the TLS handshake
        SslStream::connect(std::pin::Pin::new(&mut ssl_stream))
            .await
            .map_err(|e| {
                ProxyError::TlsError(format!("TLS handshake failed for {domain}: {e}"))
            })?;

        info!("OpenSSL TLS handshake successful for {domain}");
        Ok(ssl_stream)
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Load CA certificates from a file or directory.
fn load_ca_path(
    builder: &mut openssl::ssl::SslConnectorBuilder,
    path: &Path,
) -> ProxyResult<()> {
    if path.is_dir() {
        builder
            .set_default_verify_paths()
            .map_err(|e| {
                ProxyError::TlsError(format!("Failed to set verify paths: {e}"))
            })?;
        info!("Loaded CA certificates from directory: {}", path.display());
    } else if path.is_file() {
        let cert_bytes = fs::read(path).map_err(|e| {
            ProxyError::TlsError(format!(
                "Failed to read CA cert file {}: {e}",
                path.display()
            ))
        })?;
        let certs = X509::stack_from_pem(&cert_bytes).map_err(|e| {
            ProxyError::TlsError(format!(
                "Failed to parse CA cert file {}: {e}",
                path.display()
            ))
        })?;
        for cert in certs {
            builder
                .cert_store_mut()
                .add_cert(cert)
                .map_err(|e| ProxyError::TlsError(format!("Failed to add CA cert: {e}")))?;
        }
        info!("Loaded CA certificates from file: {}", path.display());
    } else {
        return Err(ProxyError::TlsError(format!(
            "CA cert path does not exist: {}",
            path.display()
        )));
    }
    Ok(())
}

/// Set the minimum TLS version on the connector builder.
fn set_min_tls_version(
    builder: &mut openssl::ssl::SslConnectorBuilder,
    version: TlsVersion,
) -> ProxyResult<()> {
    #[allow(deprecated)]
    let min = match version {
        TlsVersion::Tlsv10 => openssl::ssl::SslVersion::TLS1,
        TlsVersion::Tlsv11 => openssl::ssl::SslVersion::TLS1_1,
        TlsVersion::Tlsv12 => openssl::ssl::SslVersion::TLS1_2,
        TlsVersion::Tlsv13 => openssl::ssl::SslVersion::TLS1_3,
    };
    builder
        .set_min_proto_version(Some(min))
        .map_err(|e| {
            ProxyError::TlsError(format!("Failed to set min TLS version: {e}"))
        })?;
    Ok(())
}

/// Set the maximum TLS version on the connector builder.
fn set_max_tls_version(
    builder: &mut openssl::ssl::SslConnectorBuilder,
    version: TlsVersion,
) -> ProxyResult<()> {
    #[allow(deprecated)]
    let max = match version {
        TlsVersion::Tlsv10 => openssl::ssl::SslVersion::TLS1,
        TlsVersion::Tlsv11 => openssl::ssl::SslVersion::TLS1_1,
        TlsVersion::Tlsv12 => openssl::ssl::SslVersion::TLS1_2,
        TlsVersion::Tlsv13 => openssl::ssl::SslVersion::TLS1_3,
    };
    builder
        .set_max_proto_version(Some(max))
        .map_err(|e| {
            ProxyError::TlsError(format!("Failed to set max TLS version: {e}"))
        })?;
    Ok(())
}

/// Map an OpenSSL ErrorStack to a ProxyError.
fn map_ssl_error(e: ErrorStack) -> ProxyError {
    ProxyError::TlsError(format!("OpenSSL error: {e}"))
}
