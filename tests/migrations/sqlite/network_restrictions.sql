-- ==============================================
-- Table `network_restrictions`
-- Current active network restrictions
-- ==============================================
CREATE TABLE IF NOT EXISTS network_restrictions (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  role_id INTEGER NOT NULL REFERENCES roles(id),
  name TEXT NOT NULL,
  description TEXT DEFAULT NULL,
  ip_ranges TEXT NOT NULL,
  countries TEXT DEFAULT NULL,
  is_active INTEGER NOT NULL DEFAULT 1,
  created_by INTEGER DEFAULT NULL REFERENCES users(id),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_by INTEGER DEFAULT NULL REFERENCES users(id),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  deleted_by INTEGER DEFAULT NULL REFERENCES users(id),
  deleted_at TEXT DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON network_restrictions(public_id);
CREATE INDEX IF NOT EXISTS idx_role_active ON network_restrictions(role_id, is_active);
CREATE INDEX IF NOT EXISTS idx_deleted_at ON network_restrictions(deleted_at);
