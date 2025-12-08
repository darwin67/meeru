CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY,
    display_name TEXT NOT NULL,
    email_address TEXT NOT NULL,

    provider TEXT, -- "gmail", "office365", "icloud", "generic-imap", ...
    primary_protocol TEXT NOT NULL, -- "imap", "pop3", "jmap", "activesync"

    UNIQUE(email_address, provider)
);

-- An account can have multiple protocol endpoints (e.g., IMAP + SMTP + JMAP)
CREATE TABLE account_endpoints (
    id              INTEGER PRIMARY KEY,
    account_id      INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,

    protocol        TEXT NOT NULL,  -- "imap", "pop3", "jmap", "activesync", "smtp"
    host            TEXT,           -- null for JMAP/ActiveSync if you store base URL instead
    port            INTEGER,
    use_tls         INTEGER,        -- bool 0/1

    base_url        TEXT,           -- for HTTP protocols (JMAP, ActiveSync)
    api_version     TEXT,           -- if relevant
    is_primary      INTEGER NOT NULL DEFAULT 0 -- bool 0/1
);

-- Abstract: a folder-like container; POP3 = one “INBOX”, JMAP = Mailbox, ActiveSync = Folder.
CREATE TABLE folders (
    id              INTEGER PRIMARY KEY,
    account_id      INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,

    -- Human-ish name
    name            TEXT NOT NULL,      -- "Inbox", "Sent", etc.
    full_path       TEXT,               -- hierarchical path if applicable (IMAP)

    -- Role / special use
    role            TEXT,               -- "inbox", "sent", "trash", "junk", "archive", "drafts", "outbox", etc.
    is_virtual      INTEGER NOT NULL DEFAULT 0, -- e.g. unified smart folders

    -- Generic hierarchy
    parent_id       INTEGER REFERENCES folders(id),

    -- Protocol-specific remote identity (opaque)
    remote_id        TEXT,               -- IMAP: mailbox name, JMAP: mailboxId, ActiveSync: FolderId
    remote_parent_id TEXT,              -- ActiveSync/JMAP parent, if needed

    UNIQUE(account_id, remote_id)
);

-- One logical message per account; multiple folders can link to it.
CREATE TABLE messages (
    id              INTEGER PRIMARY KEY,
    account_id      INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,

    -- Generic identifiers
    external_message_id TEXT,           -- RFC 5322 Message-ID (may be NULL)
    thread_key      TEXT,               -- for threading: could be a hash of Message-ID/References or provider thread id

    -- Basic header fields
    subject         TEXT,
    from_addr       TEXT,
    to_addrs        TEXT,               -- JSON / comma-separated
    cc_addrs        TEXT,
    bcc_addrs       TEXT,

    date            INTEGER,            -- unix seconds
    size            INTEGER,

    -- Body summary
    preview_snippet TEXT,

    -- Storage references (blobs, attachments, etc.) can live in separate tables
    body_storage_key    TEXT,
    attachment_manifest TEXT,

    UNIQUE(account_id, external_message_id)
);

-- This is where IMAP flags, JMAP keywords, ActiveSync read/star state all converge.
CREATE TABLE folder_messages (
    folder_id       INTEGER NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
    message_id      INTEGER NOT NULL REFERENCES messages(id) ON DELETE CASCADE,

    -- Per-folder view
    is_read         INTEGER NOT NULL DEFAULT 0,
    is_flagged      INTEGER NOT NULL DEFAULT 0,
    is_answered     INTEGER NOT NULL DEFAULT 0,
    is_deleted      INTEGER NOT NULL DEFAULT 0,

    -- Raw provider flags/keywords (for roundtripping)
    raw_flags       TEXT,               -- IMAP: "\Seen \Flagged", JMAP: JSON keywords, ActiveSync: whatever

    PRIMARY KEY (folder_id, message_id)
);


-- IMAP specific
CREATE TABLE imap_folder_state (
    folder_id       INTEGER PRIMARY KEY REFERENCES folders(id) ON DELETE CASCADE,
    uid_validity    INTEGER,
    highest_modseq  INTEGER,
    uid_next        INTEGER,
    highest_uid     INTEGER,
    -- last time we did a full resync / QRESYNC
    last_sync_ts    INTEGER
);

CREATE TABLE imap_message_state (
    folder_id       INTEGER NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
    message_id      INTEGER NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    uid             INTEGER NOT NULL,
    modseq          INTEGER,

    PRIMARY KEY (folder_id, uid),
    UNIQUE(folder_id, message_id)
);


-- POP3 specific
-- NOTE:
-- POP3 is “single mailbox, no folders, server may not support persistent UIDs except via UIDL”.
CREATE TABLE pop3_state (
    account_id      INTEGER PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    endpoint_id     INTEGER NOT NULL REFERENCES account_endpoints(id) ON DELETE CASCADE,
    last_sync_ts    INTEGER
);

CREATE TABLE pop3_message_state (
    account_id      INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    uidl            TEXT NOT NULL,       -- string returned by UIDL
    message_id      INTEGER REFERENCES messages(id) ON DELETE CASCADE, -- can be NULL if not yet mapped
    is_deleted      INTEGER NOT NULL DEFAULT 0,

    PRIMARY KEY (account_id, uidl)
);


-- JMAP specific
-- NOTE:
-- JMAP has message ids, mailboxIds, and a global/state token per object type.
CREATE TABLE jmap_state (
    account_id      INTEGER PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    mail_state      TEXT,  -- "state" string from Mailbox/get
    msg_state       TEXT,  -- "state" from Email/get
    last_sync_ts    INTEGER
);

CREATE TABLE jmap_message_state (
    account_id      INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    jmap_id         TEXT NOT NULL,       -- Email id
    message_id      INTEGER NOT NULL REFERENCES messages(id) ON DELETE CASCADE,

    PRIMARY KEY (account_id, jmap_id)
);

-- ActiveSync specific
-- NOTE:
-- ActiveSync = items identified by ServerId, per-folder SyncKey.
CREATE TABLE eas_folder_state (
    folder_id       INTEGER PRIMARY KEY REFERENCES folders(id) ON DELETE CASCADE,
    sync_key        TEXT,
    last_sync_ts    INTEGER
);

CREATE TABLE eas_message_state (
    folder_id       INTEGER NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
    server_id       TEXT NOT NULL,      -- ActiveSync server-id for the item
    message_id      INTEGER NOT NULL REFERENCES messages(id) ON DELETE CASCADE,

    PRIMARY KEY (folder_id, server_id),
    UNIQUE(folder_id, message_id)
);
