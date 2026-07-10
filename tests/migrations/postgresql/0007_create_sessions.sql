-- ==============================================
-- Table sessions
-- Session storage table
-- Framework-level sessions
-- ==============================================
CREATE TABLE IF NOT EXISTS sessions (
  session_id BYTEA NOT NULL,
  public_id UUID NOT NULL DEFAULT gen_random_uuid(),
  session_data JSONB NOT NULL,
  expires_at TIMESTAMPTZ DEFAULT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),

  PRIMARY KEY (session_id)
);

CREATE INDEX IF NOT EXISTS idx_expires_at ON sessions(expires_at);
CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON sessions(public_id);

COMMENT ON TABLE sessions IS 'Internal session store for the web framework runtime';
