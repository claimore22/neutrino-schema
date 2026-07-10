-- ==============================================
-- Table `user_security_settings`
-- Separate table for 2FA and account lockout status
-- ==============================================
CREATE TABLE IF NOT EXISTS user_security_settings (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  user_id INTEGER NOT NULL UNIQUE REFERENCES users(id),
  two_factor_enabled INTEGER NOT NULL DEFAULT 0,
  two_factor_secret TEXT DEFAULT NULL,
  two_factor_recovery_codes TEXT DEFAULT NULL,
  two_factor_confirmed_at TEXT DEFAULT NULL,
  failed_login_attempts INTEGER NOT NULL DEFAULT 0,
  locked_until TEXT DEFAULT NULL,
  last_password_change TEXT DEFAULT NULL,
  account_locked_until TEXT DEFAULT NULL,
  lock_reason TEXT DEFAULT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  deleted_at TEXT DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON user_security_settings(public_id);
CREATE INDEX IF NOT EXISTS idx_two_factor_enabled ON user_security_settings(two_factor_enabled);
CREATE INDEX IF NOT EXISTS idx_locked_until ON user_security_settings(locked_until);
CREATE INDEX IF NOT EXISTS idx_account_locked_until ON user_security_settings(account_locked_until);
