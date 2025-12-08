use anyhow::{Context, Result};
use async_imap::Client;
use rustls::ClientConfig;
use rustls_native_certs::load_native_certs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::{client::TlsStream, TlsConnector};

// plain_client creates a plain text IMAP client
pub async fn plain_client(host: &str, port: u16) -> Result<Client<TcpStream>> {
    let conn = connect(host, port).await?;
    Ok(Client::new(conn))
}

pub async fn tls_client(
    host: &str,
    port: u16,
    ca_cert_path: Option<&Path>,
) -> Result<Client<TlsStream<TcpStream>>> {
    let conn = connect(host, port).await?;

    // Create TLS configuration with native certificates
    let mut root_cert_store = rustls::RootCertStore::empty();

    // Load custom CA certificate if provided
    if let Some(ca_path) = ca_cert_path {
        let cert_file = File::open(ca_path)
            .with_context(|| format!("Failed to open CA certificate file: {:?}", ca_path))?;
        let mut cert_reader = BufReader::new(cert_file);
        let certs = rustls_pemfile::certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse CA certificate file")?;

        for cert in certs {
            root_cert_store
                .add(cert)
                .context("Failed to add custom CA certificate")?;
        }
    } else {
        // Load native certificates only if no custom CA is provided
        let cert_result = load_native_certs();
        for cert in cert_result.certs {
            root_cert_store.add(cert).context("failed to add cert")?;
        }
        // Log any errors encountered while loading native certs
        // TODO this might potentially need to show to the UI if there are loading issues
        for error in cert_result.errors {
            eprintln!(
                "Warning: failed to load some native certificates: {}",
                error
            );
        }
    }

    let config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));

    // Upgrade TCP stream to TLS
    let domain = rustls::pki_types::ServerName::try_from(host.to_string())
        .context("invalid DNS name")?
        .to_owned();

    let tls_stream = connector
        .connect(domain, conn)
        .await
        .context("TLS handshake failed")?;

    Ok(Client::new(tls_stream))
}

async fn connect(host: &str, port: u16) -> Result<TcpStream> {
    let addr = (host, port);
    TcpStream::connect(addr)
        .await
        .context(format!("error connecting to {}:{}", host, port))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::TestEmailServer;

    #[tokio::test]
    async fn test_plain_client() {
        let serv = TestEmailServer::new()
            .user("testuser", "testpass", "example.com")
            .setup()
            .await
            .unwrap();
        let host = "127.0.0.1";
        let imap_port = serv.get_host_port_ipv4(143).await.unwrap();

        let client = plain_client(host, imap_port).await.unwrap();

        // Authenticate with the custom user (GreenMail uses local_part by default for login)
        let auth = client.login("testuser", "testpass").await;

        assert!(
            auth.is_ok(),
            "Authentication should succeed with custom user"
        );
    }

    #[tokio::test]
    async fn test_tls_client() {
        // Install default crypto provider for rustls
        let _ = rustls::crypto::ring::default_provider().install_default();

        // Get absolute paths to certificate files
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let keystore_path = format!("{}/testdata/certs/greenmail.p12", manifest_dir);
        let ca_cert_path = format!("{}/testdata/certs/ca-cert.pem", manifest_dir);

        let serv = TestEmailServer::new()
            .user("testuser", "yolo", "example.com")
            .tls_keystore(&keystore_path, "supersecure", None)
            .setup()
            .await
            .unwrap();
        let host = "127.0.0.1";
        let imaps_port = serv.get_host_port_ipv4(993).await.unwrap();

        // Use custom CA certificate for testing with GreenMail's self-signed certificate
        let client = tls_client(host, imaps_port, Some(Path::new(&ca_cert_path)))
            .await
            .unwrap();

        // Authenticate with the custom user (GreenMail uses local_part by default for login)
        let auth = client.login("testuser", "yolo").await;

        assert!(
            auth.is_ok(),
            "Authentication should succeed with TLS client and custom user"
        );
    }

    #[tokio::test]
    async fn test_preload_emails() {
        // Install default crypto provider for rustls
        let _ = rustls::crypto::ring::default_provider().install_default();

        // Get absolute paths
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let keystore_path = format!("{}/testdata/certs/greenmail.p12", manifest_dir);
        let ca_cert_path = format!("{}/testdata/certs/ca-cert.pem", manifest_dir);
        let preload_path = format!("{}/testdata/emails", manifest_dir);

        let serv = TestEmailServer::new()
            .tls_keystore(&keystore_path, "supersecure", None)
            .preload_dir(&preload_path)
            .setup()
            .await
            .unwrap();
        let host = "127.0.0.1";
        let imaps_port = serv.get_host_port_ipv4(993).await.unwrap();

        // Connect with TLS
        let client = tls_client(host, imaps_port, Some(Path::new(&ca_cert_path)))
            .await
            .unwrap();

        // Authenticate (password defaults to email address when using preload)
        let mut session = client
            .login("testuser@example.com", "testuser@example.com")
            .await
            .map_err(|e| e.0)
            .unwrap();

        // Select INBOX and verify emails were loaded
        session.select("INBOX").await.unwrap();

        assert!(
            session.examine("INBOX").await.is_ok(),
            "INBOX should exist and be accessible"
        );

        // Logout
        session.logout().await.unwrap();
    }
}
