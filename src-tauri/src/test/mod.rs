use anyhow::{Context, Result};
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage,
};

pub async fn email_server() -> Result<ContainerAsync<GenericImage>> {
    GenericImage::new("greenmail/standalone", "2.1.7")
        .with_exposed_port(3143.tcp()) // IMAP
        .with_exposed_port(3993.tcp()) // IMAPS
        .with_exposed_port(3025.tcp()) // SMTP
        .with_exposed_port(3465.tcp()) // SMTPS
        .with_exposed_port(3110.tcp()) // POP3
        .with_exposed_port(3995.tcp()) // POP3s
        .with_exposed_port(8080.tcp()) // GreenMail API
        .with_wait_for(WaitFor::message_on_stdout("Starting GreenMail API server"))
        .start()
        .await
        .context("Failed to start email server for test")
}
