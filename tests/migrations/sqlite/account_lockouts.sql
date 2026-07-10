-- ==============================================
-- Table `account_lockouts`
-- Tracks temporary account lockouts due to too many failed attempts
-- ==============================================
CREATE TABLE IF NOT EXISTS account_lockouts (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users(id),
  ip_address TEXT NOT NULL,
  reason TEXT NOT NULL,
  locked_until TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  unlocked_at TEXT DEFAULT NULL,
  unlock_reason TEXT DEFAULT NULL,
  unlocked_by INTEGER DEFAULT NULL REFERENCES users(id),
  deleted_at TEXT DEFAULT NULL,
  locked_by INTEGER DEFAULT NULL REFERENCES users(id),
  created_by INTEGER DEFAULT NULL REFERENCES users(id),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_by INTEGER DEFAULT NULL REFERENCES users(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON account_lockouts(public_id);
CREATE INDEX IF NOT EXISTS idx_user_id ON account_lockouts(user_id);
CREATE INDEX IF NOT EXISTS idx_locked_until ON account_lockouts(locked_until);
