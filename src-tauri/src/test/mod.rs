use std::default::Default;

use anyhow::{Context, Result};
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

pub struct TestEmailServer {
    // docker image name (default: "greenmail/standalone")
    image: String,
    // docker image tag (default: "2.1.7")
    tag: String,
    // enables SMTP port 25
    enable_smtp: bool,
    // enables SMTPS port 465
    enable_smtps: bool,
    // enables IMAP port 143
    enable_imap: bool,
    // enables IMAPS port 993
    enable_imaps: bool,
    // enables POP3 port 110
    enable_pop3: bool,
    // enables POP3S port 995
    enable_pop3s: bool,
    // enables GreenMail API port 8080
    enable_api: bool,
    // users as (user, passwd, domain) tuples
    users: Vec<(String, String, String)>,
}

impl Default for TestEmailServer {
    fn default() -> Self {
        TestEmailServer {
            image: "greenmail/standalone".to_string(),
            tag: "2.1.7".to_string(),
            enable_smtp: true,
            enable_smtps: true,
            enable_imap: true,
            enable_imaps: true,
            enable_pop3: false,
            enable_pop3s: false,
            enable_api: false,
            users: Vec::new(),
        }
    }
}

impl TestEmailServer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn image(mut self, img: &str) -> Self {
        self.image = img.to_string();
        self
    }

    pub fn tag(mut self, tag: &str) -> Self {
        self.tag = tag.to_string();
        self
    }

    pub fn pop3(mut self) -> Self {
        self.enable_pop3 = true;
        self
    }

    pub fn pop3s(mut self) -> Self {
        self.enable_pop3s = true;
        self
    }

    pub fn api(mut self) -> Self {
        self.enable_api = true;
        self
    }

    pub fn user(mut self, user: &str, passwd: &str, domain: &str) -> Self {
        self.users
            .push((user.to_string(), passwd.to_string(), domain.to_string()));
        self
    }

    pub async fn setup(&self) -> Result<ContainerAsync<GenericImage>> {
        let mut img = GenericImage::new(&self.image, &self.tag);

        if self.enable_smtp {
            img = img.with_exposed_port(25.tcp());
        }
        if self.enable_smtps {
            img = img.with_exposed_port(465.tcp());
        }
        if self.enable_imap {
            img = img.with_exposed_port(143.tcp());
        }
        if self.enable_imaps {
            img = img.with_exposed_port(993.tcp());
        }
        if self.enable_pop3 {
            img = img.with_exposed_port(110.tcp());
        }
        if self.enable_pop3s {
            img = img.with_exposed_port(995.tcp());
        }
        if self.enable_api {
            img = img.with_exposed_port(8080.tcp());
        }

        // Build GREENMAIL_OPTS with base configuration
        let mut greenmail_opts =
            "-Dgreenmail.hostname=0.0.0.0 -Dgreenmail.setup.all -Dgreenmail.verbose".to_string();
        if !self.users.is_empty() {
            let users_str = self
                .users
                .iter()
                .map(|(user, passwd, domain)| format!("{}:{}@{}", user, passwd, domain))
                .collect::<Vec<_>>()
                .join(",");
            greenmail_opts.push_str(&format!(" -Dgreenmail.users={}", users_str));
        }

        img.with_wait_for(WaitFor::message_on_stdout("Starting GreenMail API server"))
            .with_env_var("GREENMAIL_OPTS", greenmail_opts)
            .start()
            .await
            .context("Failed to start email server for test")
    }
}
