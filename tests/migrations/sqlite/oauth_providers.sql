-- ==============================================
-- Table `oauth_providers`
-- OAuth identity providers
-- ==============================================
CREATE TABLE IF NOT EXISTS oauth_providers (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  name TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  created_by INTEGER DEFAULT NULL REFERENCES users(id),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_by INTEGER DEFAULT NULL REFERENCES users(id),
  deleted_at TEXT DEFAULT NULL,
  deleted_by INTEGER DEFAULT NULL REFERENCES users(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON oauth_providers(public_id);
