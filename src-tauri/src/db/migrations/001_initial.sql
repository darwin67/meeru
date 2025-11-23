-- Accounts table
CREATE TABLE IF NOT EXISTS accounts (
    id CHAR(36) PRIMARY KEY NOT NULL, -- UUID
    email TEXT NOT NULL UNIQUE,
    name TEXT,
    provider TEXT NOT NULL, -- gmail, outlook, custom
    imap_host TEXT NOT NULL,
    imap_port INTEGER NOT NULL,
    smtp_host TEXT NOT NULL,
    smtp_port INTEGER NOT NULL,
    auth_type TEXT NOT NULL, -- password, oauth2
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_sync_at TIMESTAMP
);

-- Mailboxes/Folders table
CREATE TABLE IF NOT EXISTS mailboxes (
    id CHAR(36) PRIMARY KEY NOT NULL, -- UUID
    account_id CHAR(36) NOT NULL, -- UUID
    name TEXT NOT NULL,
    path TEXT NOT NULL, -- Full IMAP path (e.g., "INBOX", "INBOX/Sent")
    delimiter TEXT,
    flags TEXT, -- JSON array of IMAP flags
    role TEXT, -- inbox, sent, drafts, trash, spam, archive, all, junk
    uidvalidity INTEGER,
    uidnext INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    UNIQUE(account_id, path)
);

CREATE INDEX idx_mailboxes_account ON mailboxes(account_id);
CREATE INDEX idx_mailboxes_role ON mailboxes(role);

-- Threads table (conversation grouping)
CREATE TABLE IF NOT EXISTS threads (
    id CHAR(36) PRIMARY KEY NOT NULL, -- UUID
    account_id CHAR(36) NOT NULL, -- UUID
    subject TEXT NOT NULL,
    participants TEXT NOT NULL, -- JSON array of email addresses
    snippet TEXT, -- Preview text
    message_count INTEGER NOT NULL DEFAULT 0,
    has_attachments BOOLEAN NOT NULL DEFAULT 0,
    is_unread BOOLEAN NOT NULL DEFAULT 1,
    is_starred BOOLEAN NOT NULL DEFAULT 0,
    is_important BOOLEAN NOT NULL DEFAULT 0,
    category TEXT, -- primary, social, promotions, updates, forums
    labels TEXT, -- JSON array of label names
    first_message_at TIMESTAMP NOT NULL,
    last_message_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

CREATE INDEX idx_threads_account ON threads(account_id);
CREATE INDEX idx_threads_unread ON threads(is_unread);
CREATE INDEX idx_threads_starred ON threads(is_starred);
CREATE INDEX idx_threads_last_message ON threads(last_message_at DESC);
CREATE INDEX idx_threads_category ON threads(category);

-- Emails table
CREATE TABLE IF NOT EXISTS emails (
    id CHAR(36) PRIMARY KEY NOT NULL, -- UUID
    thread_id CHAR(36) NOT NULL, -- UUID
    account_id CHAR(36) NOT NULL, -- UUID
    mailbox_id CHAR(36) NOT NULL, -- UUID
    uid INTEGER NOT NULL, -- IMAP UID
    message_id TEXT NOT NULL, -- RFC 2822 Message-ID
    in_reply_to TEXT, -- RFC 2822 In-Reply-To
    email_references TEXT, -- RFC 2822 References (JSON array)
    subject TEXT,
    from_address TEXT NOT NULL,
    from_name TEXT,
    to_addresses TEXT NOT NULL, -- JSON array of {email, name}
    cc_addresses TEXT, -- JSON array of {email, name}
    bcc_addresses TEXT, -- JSON array of {email, name}
    reply_to TEXT, -- JSON array of {email, name}
    date TIMESTAMP NOT NULL,
    received_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    size INTEGER NOT NULL DEFAULT 0,
    flags TEXT, -- JSON array of IMAP flags (Seen, Flagged, Draft, etc.)
    is_unread BOOLEAN NOT NULL DEFAULT 1,
    is_starred BOOLEAN NOT NULL DEFAULT 0,
    is_draft BOOLEAN NOT NULL DEFAULT 0,
    has_attachments BOOLEAN NOT NULL DEFAULT 0,
    body_text TEXT,
    body_html TEXT,
    snippet TEXT,
    headers TEXT, -- JSON object of email headers
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (thread_id) REFERENCES threads(id) ON DELETE CASCADE,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (mailbox_id) REFERENCES mailboxes(id) ON DELETE CASCADE,
    UNIQUE(account_id, mailbox_id, uid)
);

CREATE INDEX idx_emails_thread ON emails(thread_id);
CREATE INDEX idx_emails_account ON emails(account_id);
CREATE INDEX idx_emails_mailbox ON emails(mailbox_id);
CREATE INDEX idx_emails_message_id ON emails(message_id);
CREATE INDEX idx_emails_date ON emails(date DESC);
CREATE INDEX idx_emails_unread ON emails(is_unread);
CREATE INDEX idx_emails_starred ON emails(is_starred);

-- Attachments table
CREATE TABLE IF NOT EXISTS attachments (
    id CHAR(36) PRIMARY KEY NOT NULL, -- UUID
    email_id CHAR(36) NOT NULL, -- UUID
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size INTEGER NOT NULL,
    content_id TEXT, -- For inline images
    is_inline BOOLEAN NOT NULL DEFAULT 0,
    data BLOB, -- Store small attachments inline, NULL for large ones
    local_path TEXT, -- Path to file on disk for large attachments
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (email_id) REFERENCES emails(id) ON DELETE CASCADE
);

CREATE INDEX idx_attachments_email ON attachments(email_id);

-- Contacts table
CREATE TABLE IF NOT EXISTS contacts (
    id CHAR(36) PRIMARY KEY NOT NULL, -- UUID
    email TEXT NOT NULL UNIQUE,
    name TEXT,
    avatar_url TEXT,
    is_vip BOOLEAN NOT NULL DEFAULT 0,
    email_count INTEGER NOT NULL DEFAULT 0, -- Number of emails exchanged
    last_emailed_at TIMESTAMP,
    metadata TEXT, -- JSON object for additional data
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_contacts_email ON contacts(email);
CREATE INDEX idx_contacts_vip ON contacts(is_vip);
CREATE INDEX idx_contacts_last_emailed ON contacts(last_emailed_at DESC);

-- Sync queue for offline support
CREATE TABLE IF NOT EXISTS sync_queue (
    id CHAR(36) PRIMARY KEY NOT NULL, -- UUID
    account_id CHAR(36) NOT NULL, -- UUID
    operation TEXT NOT NULL, -- send_email, mark_read, delete, move, etc.
    payload TEXT NOT NULL, -- JSON payload for the operation
    status TEXT NOT NULL DEFAULT 'pending', -- pending, processing, completed, failed
    retry_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

CREATE INDEX idx_sync_queue_account ON sync_queue(account_id);
CREATE INDEX idx_sync_queue_status ON sync_queue(status);

-- Search index (FTS5 for full-text search)
CREATE VIRTUAL TABLE IF NOT EXISTS emails_fts USING fts5(
    email_id UNINDEXED,
    subject,
    from_name,
    from_address,
    body_text,
    content='emails',
    content_rowid='rowid'
);

-- Triggers to keep FTS index in sync
CREATE TRIGGER IF NOT EXISTS emails_fts_insert AFTER INSERT ON emails BEGIN
    INSERT INTO emails_fts(email_id, subject, from_name, from_address, body_text)
    VALUES (new.id, new.subject, new.from_name, new.from_address, new.body_text);
END;

CREATE TRIGGER IF NOT EXISTS emails_fts_update AFTER UPDATE ON emails BEGIN
    UPDATE emails_fts
    SET subject = new.subject,
        from_name = new.from_name,
        from_address = new.from_address,
        body_text = new.body_text
    WHERE email_id = new.id;
END;

CREATE TRIGGER IF NOT EXISTS emails_fts_delete AFTER DELETE ON emails BEGIN
    DELETE FROM emails_fts WHERE email_id = old.id;
END;
