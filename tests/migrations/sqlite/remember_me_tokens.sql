-- ==============================================
-- Table `remember_me_tokens`
-- Stores persistent login tokens for "Remember Me" functionality
-- ==============================================
CREATE TABLE IF NOT EXISTS remember_me_tokens (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users(id),
  device_id BLOB DEFAULT NULL,
  series BLOB NOT NULL,
  token BLOB NOT NULL,
  expires_at TEXT NOT NULL,
  last_used TEXT DEFAULT NULL,
  user_agent TEXT DEFAULT NULL,
  ip_address TEXT DEFAULT NULL,
  network TEXT DEFAULT NULL,
  deleted_at TEXT DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON remember_me_tokens(public_id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_remember_me_series_token ON remember_me_tokens(series, token);
CREATE INDEX IF NOT EXISTS idx_remember_me_tokens_user ON remember_me_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_remember_me_tokens_series ON remember_me_tokens(series);
CREATE INDEX IF NOT EXISTS idx_remember_me_tokens_expiry ON remember_me_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_remember_me_tokens_device ON remember_me_tokens(device_id);
