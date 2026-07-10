-- ==============================================
-- Table `oauth_client_redirect_uris`
-- Permitted OAuth redirect URIs per client
-- ==============================================
CREATE TABLE IF NOT EXISTS oauth_client_redirect_uris (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  client_id INTEGER NOT NULL REFERENCES oauth_clients(id),
  redirect_uri TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_client_redirect ON oauth_client_redirect_uris(client_id, redirect_uri);
CREATE INDEX IF NOT EXISTS idx_client_id ON oauth_client_redirect_uris(client_id);
