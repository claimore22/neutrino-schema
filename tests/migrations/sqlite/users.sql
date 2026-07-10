-- ==============================================
-- Table `users`
-- Main users table with denormalized last login fields and soft delete
-- ==============================================
CREATE TABLE IF NOT EXISTS users (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  first_name TEXT NOT NULL DEFAULT '',
  last_name TEXT NOT NULL DEFAULT '',
  email TEXT NOT NULL,
  is_verified INTEGER NOT NULL DEFAULT 0,
  email_verified_at TEXT DEFAULT NULL,
  password TEXT DEFAULT NULL,
  remember_token TEXT DEFAULT NULL,
  user_type INTEGER NOT NULL DEFAULT 0,
  is_active INTEGER NOT NULL DEFAULT 1,
  last_login_at TEXT DEFAULT NULL,
  last_login_ip TEXT DEFAULT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  deleted_at TEXT DEFAULT NULL
);

CREATE INDEX IF NOT EXISTS idx_email_active ON users(email, is_active);
CREATE UNIQUE INDEX IF NOT EXISTS idx_email_unique ON users(email);
CREATE INDEX IF NOT EXISTS idx_deleted_at ON users(deleted_at);
CREATE INDEX IF NOT EXISTS idx_email_verified_at ON users(email_verified_at);
CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON users(public_id);
