use anyhow::{Context, Result};
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage,
};

pub async fn prepare_container() -> Result<ContainerAsync<GenericImage>> {
    GenericImage::new("redis", "7.2.4")
        .with_exposed_port(6379.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
        .start()
        .await
        .context("Failed to run the container")
}

pub async fn email_server() -> Result<ContainerAsync<GenericImage>> {
    GenericImage::new("stalwartlabs/stalwart", "0.14.1")
        .with_exposed_port(993.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
        .start()
        .await
        .context("Failed to start email server for test")
}

#[tokio::test]
async fn test_container() {
    let _ = prepare_container().await.unwrap();
}
