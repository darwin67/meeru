use meeru_providers::{parse_rfc822_message, GenericCredentials, ImapMessageIdentity};

#[test]
fn parses_text_html_and_attachments_from_rfc822() {
    let raw = concat!(
        "From: Sender <sender@example.com>\r\n",
        "To: Recipient <recipient@example.com>\r\n",
        "Subject: Welcome\r\n",
        "Message-ID: <welcome@example.com>\r\n",
        "Date: Sat, 19 Apr 2026 10:30:00 +0000\r\n",
        "MIME-Version: 1.0\r\n",
        "Content-Type: multipart/mixed; boundary=\"mix\"\r\n",
        "\r\n",
        "--mix\r\n",
        "Content-Type: multipart/alternative; boundary=\"alt\"\r\n",
        "\r\n",
        "--alt\r\n",
        "Content-Type: text/plain; charset=\"utf-8\"\r\n",
        "\r\n",
        "Hello from text.\r\n",
        "--alt\r\n",
        "Content-Type: text/html; charset=\"utf-8\"\r\n",
        "\r\n",
        "<html><body><p>Hello from html.</p></body></html>\r\n",
        "--alt--\r\n",
        "--mix\r\n",
        "Content-Type: text/plain; name=\"hello.txt\"\r\n",
        "Content-Disposition: attachment; filename=\"hello.txt\"\r\n",
        "Content-Transfer-Encoding: base64\r\n",
        "\r\n",
        "aGVsbG8gYXR0YWNobWVudA==\r\n",
        "--mix--\r\n"
    )
    .as_bytes();

    let parsed = parse_rfc822_message(raw).expect("message should parse");

    assert_eq!(parsed.message_id.as_deref(), Some("welcome@example.com"));
    assert_eq!(parsed.subject.as_deref(), Some("Welcome"));
    assert_eq!(parsed.from.as_ref().map(|from| from.address.as_str()), Some("sender@example.com"));
    assert_eq!(parsed.to.len(), 1);
    assert_eq!(parsed.to[0].address, "recipient@example.com");
    assert_eq!(parsed.text_body.as_deref(), Some("Hello from text."));
    assert_eq!(
        parsed.html_body.as_deref(),
        Some("<html><body><p>Hello from html.</p></body></html>")
    );
    assert_eq!(parsed.attachments.len(), 1);
    assert_eq!(parsed.attachments[0].filename, "hello.txt");
    assert_eq!(parsed.attachments[0].content, b"hello attachment");
}

#[test]
fn html_only_messages_still_produce_text_and_html_bodies() {
    let raw = concat!(
        "From: Sender <sender@example.com>\r\n",
        "To: Recipient <recipient@example.com>\r\n",
        "Subject: Html only\r\n",
        "Date: Sat, 19 Apr 2026 10:30:00 +0000\r\n",
        "Content-Type: text/html; charset=\"utf-8\"\r\n",
        "\r\n",
        "<html><body><p>HTML only body.</p></body></html>\r\n"
    )
    .as_bytes();

    let parsed = parse_rfc822_message(raw).expect("message should parse");

    assert!(parsed.text_body.as_deref().is_some_and(|text| text.contains("HTML only body.")));
    assert!(
        parsed
            .html_body
            .as_deref()
            .is_some_and(|html| html.contains("HTML only body."))
    );
}

#[test]
fn exposes_helper_types_for_generic_mvp_inputs() {
    let credentials = GenericCredentials::Password {
        username: "alice@example.com".to_string(),
        password: "secret".to_string(),
    };
    let identity = ImapMessageIdentity::new("INBOX", 99, 1234);

    assert_eq!(credentials.username(), "alice@example.com");
    assert_eq!(identity.provider_id(), "INBOX:99:1234");
}
