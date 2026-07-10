-- ==============================================
-- Table `oauth_authorization_codes`
-- Temporary auth codes for OAuth PKCE flow
-- ==============================================
CREATE TABLE IF NOT EXISTS oauth_authorization_codes (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  code BLOB NOT NULL,
  client_id INTEGER NOT NULL REFERENCES oauth_clients(id),
  user_id INTEGER DEFAULT NULL REFERENCES users(id),
  redirect_uri TEXT DEFAULT NULL,
  expires_at TEXT NOT NULL,
  code_challenge TEXT DEFAULT NULL,
  code_challenge_method TEXT DEFAULT NULL CHECK(code_challenge_method IN ('plain', 'S256')),
  scope TEXT DEFAULT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  consumed INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON oauth_authorization_codes(public_id);
CREATE INDEX IF NOT EXISTS idx_client_id ON oauth_authorization_codes(client_id);
CREATE INDEX IF NOT EXISTS idx_user_id ON oauth_authorization_codes(user_id);
