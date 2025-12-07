use anyhow::{Context, Result};
use async_imap::Client;
use rustls::ClientConfig;
use rustls_native_certs::load_native_certs;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::{client::TlsStream, TlsConnector};

// plain_client creates a plain text IMAP client
pub async fn plain_client(host: &str, port: u16) -> Result<Client<TcpStream>> {
    let conn = connect(host, port).await?;
    Ok(Client::new(conn))
}

pub async fn tls_client(host: &str, port: u16) -> Result<Client<TlsStream<TcpStream>>> {
    let conn = connect(host, port).await?;

    // Create TLS configuration with native certificates
    let mut root_cert_store = rustls::RootCertStore::empty();
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

        let serv = TestEmailServer::new()
            .user("testuser", "yolo", "example.com")
            .setup()
            .await
            .unwrap();
        let host = "127.0.0.1";
        let imaps_port = serv.get_host_port_ipv4(993).await.unwrap();

        // Use insecure client for testing with GreenMail's self-signed certificate
        let client = tls_client(host, imaps_port).await.unwrap();

        // Authenticate with the custom user (GreenMail uses local_part by default for login)
        let auth = client.login("testuser", "yolo").await;

        assert!(
            auth.is_ok(),
            "Authentication should succeed with TLS client and custom user"
        );
    }
}
