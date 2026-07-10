-- ==============================================
-- Table `oauth_access_tokens`
-- OAuth access tokens for user and clients
-- ==============================================
CREATE TABLE IF NOT EXISTS oauth_access_tokens (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  token BLOB NOT NULL,
  client_id INTEGER NOT NULL REFERENCES oauth_clients(id),
  user_id INTEGER DEFAULT NULL REFERENCES users(id),
  scopes TEXT DEFAULT NULL,
  revoked INTEGER NOT NULL DEFAULT 0,
  expires_at TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  last_used_at TEXT DEFAULT NULL,
  deleted_at TEXT DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON oauth_access_tokens(public_id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_token ON oauth_access_tokens(token);
CREATE INDEX IF NOT EXISTS idx_client_id ON oauth_access_tokens(client_id);
CREATE INDEX IF NOT EXISTS idx_user_id ON oauth_access_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_expires_at ON oauth_access_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_revoked ON oauth_access_tokens(revoked);
