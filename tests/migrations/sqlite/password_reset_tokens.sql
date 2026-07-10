-- ==============================================
-- Table `password_reset_tokens`
-- Stores password reset tokens with expiration and usage tracking
-- ==============================================
CREATE TABLE IF NOT EXISTS password_reset_tokens (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users(id),
  token BLOB NOT NULL,
  expires_at TEXT NOT NULL,
  used_at TEXT DEFAULT NULL,
  ip_address TEXT DEFAULT NULL,
  user_agent TEXT DEFAULT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  deleted_at TEXT DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON password_reset_tokens(public_id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_token ON password_reset_tokens(token);
CREATE INDEX IF NOT EXISTS idx_user ON password_reset_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_expires ON password_reset_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_used ON password_reset_tokens(used_at);
CREATE INDEX IF NOT EXISTS idx_ip_created ON password_reset_tokens(ip_address, created_at);
