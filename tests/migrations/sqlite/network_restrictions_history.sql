-- ==============================================
-- Table `network_restrictions_history`
-- Audit log of all changes to network_restrictions
-- ==============================================
CREATE TABLE IF NOT EXISTS network_restrictions_history (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  restriction_id INTEGER NOT NULL REFERENCES network_restrictions(id),
  action TEXT NOT NULL CHECK(action IN ('CREATE', 'UPDATE', 'DELETE')),
  old_value TEXT DEFAULT NULL,
  new_value TEXT DEFAULT NULL,
  performed_by INTEGER DEFAULT NULL REFERENCES users(id),
  performed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON network_restrictions_history(public_id);
CREATE INDEX IF NOT EXISTS idx_restriction_action_time ON network_restrictions_history(restriction_id, action, performed_at);
