CREATE TABLE users (
    id             BLOB    PRIMARY KEY NOT NULL,          -- Uuid (16 bytes)
    email          TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    email_verified INTEGER NOT NULL DEFAULT 0,            -- bool
    created_at     TEXT    NOT NULL,                      -- DateTime<Utc> en RFC 3339
    updated_at     TEXT    NOT NULL
) STRICT;

CREATE TABLE credentials (
    id         BLOB PRIMARY KEY NOT NULL,
    user_id    BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind       TEXT NOT NULL CHECK (kind IN ('password', 'passkey', 'totp')),
    secret     TEXT NOT NULL,                             -- hash argon2, public key or TOTP secret
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT                                       -- logic deletion
) STRICT;

CREATE INDEX idx_credentials_user_id ON credentials(user_id);
