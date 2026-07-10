-- ==============================================
-- Table `email_verification_tokens`
-- Stores email verification tokens with expiration and rate limiting
-- ==============================================
CREATE TABLE IF NOT EXISTS email_verification_tokens (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users(id),
  token BLOB NOT NULL,
  expires_at TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT DEFAULT NULL,
  ip_address TEXT DEFAULT NULL,
  user_agent TEXT DEFAULT NULL,
  used_at TEXT DEFAULT NULL,
  email TEXT NOT NULL,
  deleted_at TEXT DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON email_verification_tokens(public_id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_token ON email_verification_tokens(token);
CREATE INDEX IF NOT EXISTS idx_user ON email_verification_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_expires ON email_verification_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_used ON email_verification_tokens(used_at);
CREATE INDEX IF NOT EXISTS idx_email_verification_rate_limit ON email_verification_tokens(ip_address, created_at);
CREATE INDEX IF NOT EXISTS idx_email_rate_limit ON email_verification_tokens(email, created_at);
