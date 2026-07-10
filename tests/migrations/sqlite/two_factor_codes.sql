-- ==============================================
-- Table `two_factor_codes`
-- Stores two-factor authentication codes for users
-- ==============================================
CREATE TABLE IF NOT EXISTS two_factor_codes (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users(id),
  code TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  used_at TEXT DEFAULT NULL,
  ip_address TEXT DEFAULT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON two_factor_codes(public_id);
CREATE INDEX IF NOT EXISTS idx_user_code ON two_factor_codes(user_id, code);
CREATE INDEX IF NOT EXISTS idx_expires ON two_factor_codes(expires_at);
