-- ==============================================
-- Table `user_sessions`
-- User sessions table
-- Application-level sessions
-- ==============================================
CREATE TABLE IF NOT EXISTS user_sessions (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users(id),
  session_id BLOB NOT NULL UNIQUE,
  device_id BLOB DEFAULT NULL,
  ip_address TEXT DEFAULT NULL,
  user_agent TEXT DEFAULT NULL,
  user_agent_hash BLOB DEFAULT NULL,
  payload BLOB NOT NULL,
  is_revoked INTEGER NOT NULL DEFAULT 0,
  is_2fa_verified INTEGER NOT NULL DEFAULT 0,
  last_activity TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  expires_at TEXT NOT NULL,
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  metadata TEXT DEFAULT NULL,
  revoked_at TEXT DEFAULT NULL,
  deleted_at TEXT DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON user_sessions(public_id);
CREATE INDEX IF NOT EXISTS idx_user_last_activity ON user_sessions(user_id, last_activity);
CREATE INDEX IF NOT EXISTS idx_user_active ON user_sessions(user_id, is_revoked, expires_at);
CREATE INDEX IF NOT EXISTS idx_expiring_sessions ON user_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_device ON user_sessions(user_id, device_id);
