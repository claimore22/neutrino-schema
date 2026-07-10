-- ==============================================
-- Table `oauth_device_codes`
-- Implements OAuth 2.0 Device Authorization Grant (RFC 8628)
-- ==============================================
CREATE TABLE IF NOT EXISTS oauth_device_codes (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  device_code TEXT NOT NULL,
  user_code TEXT NOT NULL,
  client_id INTEGER NOT NULL REFERENCES oauth_clients(id),
  user_id INTEGER DEFAULT NULL REFERENCES users(id),
  scopes TEXT DEFAULT NULL,
  ip_address TEXT DEFAULT NULL,
  user_agent TEXT DEFAULT NULL,
  device_name TEXT DEFAULT NULL,
  expires_at TEXT NOT NULL,
  last_poll TEXT DEFAULT NULL,
  poll_interval INTEGER NOT NULL DEFAULT 5,
  status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'approved', 'denied', 'expired')),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  approved_at TEXT DEFAULT NULL,
  denied_at TEXT DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON oauth_device_codes(public_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_device_code ON oauth_device_codes(device_code);
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_code ON oauth_device_codes(user_code);
CREATE INDEX IF NOT EXISTS idx_client_id ON oauth_device_codes(client_id);
CREATE INDEX IF NOT EXISTS idx_user_id ON oauth_device_codes(user_id);
CREATE INDEX IF NOT EXISTS idx_status ON oauth_device_codes(status);
CREATE INDEX IF NOT EXISTS idx_expires_at ON oauth_device_codes(expires_at);
