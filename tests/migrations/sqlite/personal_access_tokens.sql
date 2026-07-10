-- ==============================================
-- Table `personal_access_tokens`
-- API tokens for user authentication
-- ==============================================
CREATE TABLE IF NOT EXISTS personal_access_tokens (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users(id),
  name TEXT NOT NULL,
  token BLOB NOT NULL,
  abilities TEXT DEFAULT NULL,
  last_used_at TEXT DEFAULT NULL,
  expires_at TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  revoked INTEGER NOT NULL DEFAULT 0,
  last_used_ip TEXT DEFAULT NULL,
  last_used_user_agent TEXT DEFAULT NULL,
  revoked_at TEXT DEFAULT NULL,
  deleted_at TEXT DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON personal_access_tokens(public_id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_token ON personal_access_tokens(token);
CREATE INDEX IF NOT EXISTS idx_user_id ON personal_access_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_expires_at ON personal_access_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_revoked ON personal_access_tokens(revoked);
