-- ==============================================
-- Table `user_allowed_ips`
-- Optional table for users to whitelist specific IPs, should be mandatory for privileged users
-- ==============================================
CREATE TABLE IF NOT EXISTS user_allowed_ips (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users(id),
  ip_address TEXT NOT NULL,
  label TEXT DEFAULT NULL,
  is_active INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  created_by INTEGER DEFAULT NULL REFERENCES users(id),
  updated_by INTEGER DEFAULT NULL REFERENCES users(id),
  deleted_at TEXT DEFAULT NULL,
  deleted_by INTEGER DEFAULT NULL REFERENCES users(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_user_ip ON user_allowed_ips(user_id, ip_address);
CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON user_allowed_ips(public_id);
CREATE INDEX IF NOT EXISTS idx_user_active ON user_allowed_ips(user_id, is_active);
CREATE INDEX IF NOT EXISTS idx_ip_address ON user_allowed_ips(ip_address);
