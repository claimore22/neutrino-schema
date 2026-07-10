-- ==============================================
-- Table `oauth_clients`
-- OAuth client applications
-- ==============================================
CREATE TABLE IF NOT EXISTS oauth_clients (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  user_id INTEGER DEFAULT NULL REFERENCES users(id),
  name TEXT NOT NULL,
  secret BLOB NOT NULL,
  secret_expires_at TEXT DEFAULT NULL,
  provider TEXT DEFAULT NULL,
  personal_access_client INTEGER NOT NULL DEFAULT 0,
  pkce_required INTEGER NOT NULL DEFAULT 1,
  revoked INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  deleted_at TEXT DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON oauth_clients(public_id);
CREATE INDEX IF NOT EXISTS idx_user_id ON oauth_clients(user_id);
