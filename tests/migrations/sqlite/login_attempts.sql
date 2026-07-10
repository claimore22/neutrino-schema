-- ==============================================
-- Table `login_attempts`
-- Tracks failed and successful login attempts for security and rate limiting
-- ==============================================
CREATE TABLE IF NOT EXISTS login_attempts (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  user_id INTEGER DEFAULT NULL REFERENCES users(id),
  email TEXT DEFAULT NULL,
  ip_address TEXT NOT NULL,
  user_agent TEXT DEFAULT NULL,
  attempted_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  successful INTEGER NOT NULL DEFAULT 0,
  failure_reason TEXT DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON login_attempts(public_id);
CREATE INDEX IF NOT EXISTS idx_user_id ON login_attempts(user_id);
CREATE INDEX IF NOT EXISTS idx_email ON login_attempts(email);
CREATE INDEX IF NOT EXISTS idx_ip ON login_attempts(ip_address);
CREATE INDEX IF NOT EXISTS idx_attempted_at ON login_attempts(attempted_at);
