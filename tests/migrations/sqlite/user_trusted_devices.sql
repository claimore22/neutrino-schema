-- ==============================================
-- Table `user_trusted_devices`
-- Optional table for users to remember and manage trusted devices
-- ==============================================
CREATE TABLE IF NOT EXISTS user_trusted_devices (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users(id),
  device_id BLOB NOT NULL,
  device_name TEXT DEFAULT NULL,
  device_type TEXT DEFAULT NULL,
  os TEXT DEFAULT NULL,
  browser TEXT DEFAULT NULL,
  last_used_at TEXT DEFAULT NULL,
  expires_at TEXT DEFAULT NULL,
  is_revoked INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  created_by INTEGER DEFAULT NULL REFERENCES users(id),
  updated_by INTEGER DEFAULT NULL REFERENCES users(id),
  deleted_at TEXT DEFAULT NULL,
  deleted_by INTEGER DEFAULT NULL REFERENCES users(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_user_device ON user_trusted_devices(user_id, device_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON user_trusted_devices(public_id);
CREATE INDEX IF NOT EXISTS idx_user_active ON user_trusted_devices(user_id, is_revoked);
CREATE INDEX IF NOT EXISTS idx_expires ON user_trusted_devices(expires_at);
CREATE INDEX IF NOT EXISTS idx_device_id ON user_trusted_devices(device_id);
CREATE INDEX IF NOT EXISTS idx_trusted_device ON user_trusted_devices(user_id, device_id);
