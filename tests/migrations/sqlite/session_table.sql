-- ==============================================
-- Table `sessions`
-- Session storage table
-- Framework-level sessions
-- ==============================================
CREATE TABLE IF NOT EXISTS sessions (
  session_id BLOB PRIMARY KEY,
  public_id BLOB NOT NULL,
  session_data TEXT NOT NULL,
  expires_at TEXT DEFAULT NULL,
  created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_expires_at ON sessions(expires_at);
CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON sessions(public_id);
