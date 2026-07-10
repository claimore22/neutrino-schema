-- ==============================================
-- Table `oauth_refresh_tokens`
-- OAuth refresh tokens linked to access tokens
-- ==============================================
CREATE TABLE IF NOT EXISTS oauth_refresh_tokens (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  token BLOB NOT NULL,
  access_token_id INTEGER NOT NULL REFERENCES oauth_access_tokens(id),
  previous_token_id INTEGER DEFAULT NULL REFERENCES oauth_refresh_tokens(id),
  revoked INTEGER NOT NULL DEFAULT 0,
  expires_at TEXT NOT NULL,
  used_at TEXT DEFAULT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  deleted_at TEXT DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON oauth_refresh_tokens(public_id);
CREATE INDEX IF NOT EXISTS idx_access_token_id ON oauth_refresh_tokens(access_token_id);
