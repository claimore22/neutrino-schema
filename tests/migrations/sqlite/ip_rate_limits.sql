-- ==============================================
-- Table `ip_rate_limits`
-- Tracks rate limiting by IP address
-- ==============================================
CREATE TABLE IF NOT EXISTS ip_rate_limits (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  ip_address TEXT NOT NULL,
  endpoint TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 1,
  first_attempt_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  last_attempt_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  blocked_until TEXT DEFAULT NULL,
  block_reason TEXT DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON ip_rate_limits(public_id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_ip_endpoint ON ip_rate_limits(ip_address, endpoint);
CREATE INDEX IF NOT EXISTS idx_ip_blocked_until ON ip_rate_limits(ip_address, blocked_until);
CREATE INDEX IF NOT EXISTS idx_last_attempt ON ip_rate_limits(last_attempt_at);
