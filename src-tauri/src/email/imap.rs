use anyhow::{Context, Result};
use async_imap::Client;
use tokio::net::TcpStream;

// plain_client creates a plain text IMAP client
pub async fn plain_client(host: &str, port: u16) -> Result<Client<TcpStream>> {
    let stream = connect(host, port).await?;
    Ok(Client::new(stream))
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
}
