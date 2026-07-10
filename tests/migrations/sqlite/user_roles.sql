-- ==============================================
-- Table `user_roles`
-- Many-to-many relationship between users and roles
-- ==============================================
CREATE TABLE IF NOT EXISTS user_roles (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  public_id BLOB NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users(id),
  role_id INTEGER NOT NULL REFERENCES roles(id),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  deleted_at TEXT DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_user_role ON user_roles(user_id, role_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON user_roles(public_id);
