-- ==============================================
-- Table `oauth_accounts`
-- Links user accounts to OAuth providers
-- ==============================================
CREATE TABLE IF NOT EXISTS oauth_accounts (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users(id),
  provider_id INTEGER NOT NULL REFERENCES oauth_providers(id),
  provider_user_id TEXT NOT NULL,
  access_token TEXT DEFAULT NULL,
  refresh_token TEXT DEFAULT NULL,
  token_expires_at TEXT NOT NULL,
  token_issued_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  created_by INTEGER DEFAULT NULL REFERENCES users(id),
  updated_by INTEGER DEFAULT NULL REFERENCES users(id),
  deleted_at TEXT DEFAULT NULL,
  deleted_by INTEGER DEFAULT NULL REFERENCES users(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON oauth_accounts(public_id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_provider_user ON oauth_accounts(provider_id, provider_user_id);
CREATE INDEX IF NOT EXISTS idx_user_id ON oauth_accounts(user_id);
