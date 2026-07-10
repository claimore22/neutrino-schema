-- ==============================================
-- Table `oauth_client_grants`
-- OAuth client grant types
-- ==============================================
CREATE TABLE IF NOT EXISTS oauth_client_grants (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  client_id INTEGER NOT NULL REFERENCES oauth_clients(id),
  grant_type TEXT NOT NULL CHECK(grant_type IN ('authorization_code', 'client_credentials', 'device_code', 'refresh_token')),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  revoked_at TEXT DEFAULT NULL,
  revocation_reason TEXT DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON oauth_client_grants(public_id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_client_grant ON oauth_client_grants(client_id, grant_type);
