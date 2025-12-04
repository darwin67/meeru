use anyhow::{Context, Result};
use tokio::net::TcpStream;

pub async fn connect(host: &str, port: u16) -> Result<TcpStream> {
    let addr = (host, port);
    TcpStream::connect(addr)
        .await
        .context(format!("error connecting to {}:{}", host, port))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::email_server;

    #[tokio::test]
    async fn test_connect_imap() {
        let serv = email_server().await.unwrap();
        let imap_port = serv.get_host_port_ipv4(3143).await.unwrap();

        assert!(connect("127.0.0.1", imap_port).await.is_ok());
    }
}
