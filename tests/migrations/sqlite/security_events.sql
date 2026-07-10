-- ==============================================
-- Table `security_events`
-- Logs security relevant events for auditing
-- ==============================================
CREATE TABLE IF NOT EXISTS security_events (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  user_id INTEGER DEFAULT NULL REFERENCES users(id),
  event_type TEXT NOT NULL,
  ip_address TEXT DEFAULT NULL,
  user_agent TEXT DEFAULT NULL,
  metadata TEXT DEFAULT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON security_events(public_id);
CREATE INDEX IF NOT EXISTS idx_user_id ON security_events(user_id);
CREATE INDEX IF NOT EXISTS idx_event_type ON security_events(event_type);
CREATE INDEX IF NOT EXISTS idx_created_at ON security_events(created_at);
