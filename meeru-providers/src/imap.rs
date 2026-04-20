//! IMAP operations for the generic provider baseline.

use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

use async_imap::{Client, Session};
use futures::TryStreamExt;
use native_tls::TlsConnector as NativeTlsConnector;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::TcpStream,
};
use tokio_native_tls::{TlsConnector, TlsStream};

use crate::{
    generic::{FetchedMessage, GenericAccountConfig, GenericCredentials, GenericMailbox},
    Error, ImapEnvelopeSummary, ImapMessageIdentity, Result, TransportSecurity,
};

#[derive(Debug)]
enum ImapNetworkStream {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl AsyncRead for ImapNetworkStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Plain(stream) => Pin::new(stream).poll_read(cx, buf),
            Self::Tls(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for ImapNetworkStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            Self::Plain(stream) => Pin::new(stream).poll_write(cx, buf),
            Self::Tls(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Plain(stream) => Pin::new(stream).poll_flush(cx),
            Self::Tls(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Plain(stream) => Pin::new(stream).poll_shutdown(cx),
            Self::Tls(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

pub async fn validate_connectivity(config: &GenericAccountConfig) -> Result<()> {
    let mut session = connect_and_login(config).await?;
    session
        .logout()
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?;
    Ok(())
}

pub async fn list_mailboxes(config: &GenericAccountConfig) -> Result<Vec<GenericMailbox>> {
    let mut session = connect_and_login(config).await?;
    let names = session
        .list(None, Some("*"))
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?
        .try_collect::<Vec<_>>()
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?;
    session
        .logout()
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?;

    Ok(names
        .into_iter()
        .map(|name| GenericMailbox {
            path: name.name().to_string(),
            display_name: name.name().to_string(),
        })
        .collect())
}

pub async fn fetch_mailbox_page(
    config: &GenericAccountConfig,
    mailbox_path: &str,
    page_size: usize,
) -> Result<Vec<ImapEnvelopeSummary>> {
    let mut session = connect_and_login(config).await?;
    let mailbox = session
        .select(mailbox_path)
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?;
    let uidvalidity = mailbox.uid_validity.ok_or_else(|| {
        Error::Protocol(format!("mailbox {mailbox_path} did not report UIDVALIDITY"))
    })?;

    if mailbox.exists == 0 || page_size == 0 {
        session
            .logout()
            .await
            .map_err(|error| Error::Protocol(error.to_string()))?;
        return Ok(Vec::new());
    }

    let start = mailbox.exists.saturating_sub(page_size as u32).max(1);
    let sequence_set = format!("{start}:{}", mailbox.exists);
    let fetches = session
        .fetch(sequence_set, "(UID ENVELOPE INTERNALDATE RFC822.SIZE)")
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?
        .try_collect::<Vec<_>>()
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?;
    session
        .logout()
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?;

    fetches
        .into_iter()
        .map(|fetch| summary_from_fetch(mailbox_path, uidvalidity, &fetch))
        .collect()
}

pub async fn search_mailbox(
    config: &GenericAccountConfig,
    mailbox_path: &str,
    query: &str,
    limit: usize,
) -> Result<Vec<ImapEnvelopeSummary>> {
    let mut session = connect_and_login(config).await?;
    let mailbox = session
        .select(mailbox_path)
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?;
    let uidvalidity = mailbox.uid_validity.ok_or_else(|| {
        Error::Protocol(format!("mailbox {mailbox_path} did not report UIDVALIDITY"))
    })?;

    let mut uids = session
        .uid_search(query)
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?
        .into_iter()
        .collect::<Vec<_>>();
    uids.sort_unstable();
    uids.reverse();
    if limit > 0 {
        uids.truncate(limit);
    }

    if uids.is_empty() {
        session
            .logout()
            .await
            .map_err(|error| Error::Protocol(error.to_string()))?;
        return Ok(Vec::new());
    }

    let fetches = session
        .uid_fetch(join_uids(&uids), "(UID ENVELOPE INTERNALDATE RFC822.SIZE)")
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?
        .try_collect::<Vec<_>>()
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?;
    session
        .logout()
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?;

    fetches
        .into_iter()
        .map(|fetch| summary_from_fetch(mailbox_path, uidvalidity, &fetch))
        .collect()
}

pub async fn fetch_messages(
    config: &GenericAccountConfig,
    mailbox_path: &str,
    uids: &[u32],
) -> Result<Vec<FetchedMessage>> {
    if uids.is_empty() {
        return Ok(Vec::new());
    }

    let mut session = connect_and_login(config).await?;
    let mailbox = session
        .select(mailbox_path)
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?;
    let uidvalidity = mailbox.uid_validity.ok_or_else(|| {
        Error::Protocol(format!("mailbox {mailbox_path} did not report UIDVALIDITY"))
    })?;

    let fetches = session
        .uid_fetch(join_uids(uids), "(UID RFC822)")
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?
        .try_collect::<Vec<_>>()
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?;
    session
        .logout()
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?;

    fetches
        .into_iter()
        .map(|fetch| {
            let uid = fetch
                .uid
                .ok_or_else(|| Error::Protocol("FETCH response missing UID".to_string()))?;
            let raw_message = fetch
                .body()
                .ok_or_else(|| Error::Protocol("FETCH response missing RFC822 body".to_string()))?
                .to_vec();

            Ok(FetchedMessage {
                identity: ImapMessageIdentity::new(mailbox_path, uidvalidity, uid),
                raw_message,
            })
        })
        .collect()
}

fn summary_from_fetch(
    mailbox_path: &str,
    uidvalidity: u32,
    fetch: &async_imap::types::Fetch,
) -> Result<ImapEnvelopeSummary> {
    let uid = fetch
        .uid
        .ok_or_else(|| Error::Protocol("FETCH response missing UID".to_string()))?;
    let envelope = fetch.envelope();
    let provider_id = ImapMessageIdentity::new(mailbox_path, uidvalidity, uid).provider_id();

    Ok(ImapEnvelopeSummary {
        uid,
        provider_id,
        message_id: envelope
            .and_then(|envelope| envelope.message_id.as_ref())
            .map(|value| String::from_utf8_lossy(value.as_ref()).into_owned()),
        subject: envelope
            .and_then(|envelope| envelope.subject.as_ref())
            .map(|value| String::from_utf8_lossy(value.as_ref()).into_owned()),
        internal_date: fetch
            .internal_date()
            .map(|date| date.with_timezone(&chrono::Utc)),
    })
}

async fn connect_and_login(config: &GenericAccountConfig) -> Result<Session<ImapNetworkStream>> {
    config.validate()?;

    match config.imap.security {
        TransportSecurity::Tls => {
            let tcp_stream = connect_tcp(config).await?;
            let tls_stream = connect_tls(&config.imap.host, tcp_stream).await?;
            login_client(
                client_with_greeting(ImapNetworkStream::Tls(tls_stream)).await?,
                config,
            )
            .await
        },
        TransportSecurity::None => {
            login_client(
                client_with_greeting(ImapNetworkStream::Plain(connect_tcp(config).await?)).await?,
                config,
            )
            .await
        },
        TransportSecurity::StartTls => {
            let mut client =
                client_with_greeting(ImapNetworkStream::Plain(connect_tcp(config).await?)).await?;
            client
                .run_command_and_check_ok("STARTTLS", None)
                .await
                .map_err(|error| Error::Protocol(error.to_string()))?;
            let plain_stream = match client.into_inner() {
                ImapNetworkStream::Plain(stream) => stream,
                ImapNetworkStream::Tls(_) => {
                    return Err(Error::Protocol(
                        "unexpected TLS stream before STARTTLS upgrade".to_string(),
                    ))
                },
            };
            let tls_stream = connect_tls(&config.imap.host, plain_stream).await?;
            login_client(Client::new(ImapNetworkStream::Tls(tls_stream)), config).await
        },
    }
}

async fn client_with_greeting(stream: ImapNetworkStream) -> Result<Client<ImapNetworkStream>> {
    let mut client = Client::new(stream);
    let greeting = client
        .read_response()
        .await
        .map_err(|error| Error::Connection(error.to_string()))?;

    if greeting.is_none() {
        return Err(Error::Connection(
            "imap server closed the connection before greeting".to_string(),
        ));
    }

    Ok(client)
}

async fn login_client(
    client: Client<ImapNetworkStream>,
    config: &GenericAccountConfig,
) -> Result<Session<ImapNetworkStream>> {
    match &config.credentials {
        GenericCredentials::Password { username, password } => client
            .login(username, password)
            .await
            .map_err(|(error, _)| Error::Authentication(error.to_string())),
        GenericCredentials::OAuth2Bearer {
            username,
            access_token,
        } => {
            let auth = ImapOAuth2Authenticator {
                username: username.clone(),
                access_token: access_token.clone(),
            };
            client
                .authenticate("XOAUTH2", &auth)
                .await
                .map_err(|(error, _)| Error::Authentication(error.to_string()))
        },
    }
}

async fn connect_tcp(config: &GenericAccountConfig) -> Result<TcpStream> {
    TcpStream::connect((config.imap.host.as_str(), config.imap.port))
        .await
        .map_err(|error| Error::Connection(error.to_string()))
}

async fn connect_tls(host: &str, stream: TcpStream) -> Result<TlsStream<TcpStream>> {
    let connector = NativeTlsConnector::builder()
        .build()
        .map_err(|error| Error::Connection(error.to_string()))?;
    let connector = TlsConnector::from(connector);

    connector
        .connect(host, stream)
        .await
        .map_err(|error| Error::Connection(error.to_string()))
}

fn join_uids(uids: &[u32]) -> String {
    uids.iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[derive(Clone)]
struct ImapOAuth2Authenticator {
    username: String,
    access_token: String,
}

impl async_imap::Authenticator for &ImapOAuth2Authenticator {
    type Response = String;

    fn process(&mut self, _challenge: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.username, self.access_token
        )
    }
}

impl fmt::Display for ImapNetworkStream {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain(_) => formatter.write_str("plain-imap-stream"),
            Self::Tls(_) => formatter.write_str("tls-imap-stream"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::join_uids;

    #[test]
    fn joins_uid_sequence_sets() {
        assert_eq!(join_uids(&[4, 8, 15]), "4,8,15");
    }
}
