-- Add migration script here
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
