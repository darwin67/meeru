use anyhow::{Context, Result};
use async_imap::Client;
use futures_util::TryStreamExt;
use rustls_native_certs::load_native_certs;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::{rustls, TlsConnector};

#[tokio::main]
async fn main() -> Result<()> {
    // Install default crypto provider for rustls
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let imap_server = "127.0.0.1";
    let imap_addr = (imap_server, 993);
    let tcp_stream = TcpStream::connect(imap_addr).await?;

    // Create TLS connector with both native and custom certificates
    let mut root_cert_store = rustls::RootCertStore::empty();

    // Load native certificates
    let cert_result = load_native_certs();
    for cert in cert_result.certs {
        root_cert_store.add(cert)?;
    }
    // Log any errors encountered while loading native certs
    for error in cert_result.errors {
        eprintln!(
            "Warning: failed to load some native certificates: {}",
            error
        );
    }

    // Load custom certificate from dev/stalwart/tls/cert.pem
    let cert_file = File::open("../dev/stalwart/tls/cert.pem")
        .context("Failed to open custom certificate file")?;
    let mut cert_reader = BufReader::new(cert_file);
    let certs = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to parse certificate file")?;

    for cert in certs {
        root_cert_store.add(cert)?;
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let domain = rustls::pki_types::ServerName::try_from(imap_server)?;
    let tls_stream = connector.connect(domain, tcp_stream).await?;

    let client = Client::new(tls_stream);
    println!("created IMAP client");

    // TODO create users in stalwart via config and test login here
    let login = "hello";
    let mut session = client.login(login, "supersecure").await.map_err(|e| e.0)?;
    println!("-- logged in a {}", login);

    session.select("INBOX").await?;
    println!("-- INBOX selected");

    // fetch message number 1 in this mailbox, along with its RFC822 field.
    // RFC 822 dictates the format of the body of e-mails
    let messages_stream = session.fetch("1", "RFC822").await?;
    let messages: Vec<_> = messages_stream.try_collect().await?;
    let message = if let Some(m) = messages.first() {
        m
    } else {
        println!("-- No messages found in INBOX");
        session.logout().await?;
        return Ok(());
    };

    // extract the message's body
    let body = message.body().expect("message did not have a body!");
    let _body = std::str::from_utf8(body)
        .expect("message was not valid utf-8")
        .to_string();
    println!("-- 1 message received, logging out");

    // be nice to the server and log out
    session.logout().await?;

    Ok(())
}
