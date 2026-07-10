-- ==============================================
-- Table `sessions`
-- Session storage table
-- Framework-level sessions
-- ==============================================
CREATE TABLE IF NOT EXISTS sessions (
  `session_id` BINARY(32) PRIMARY KEY COMMENT 'Secure random 32-byte ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `session_data` JSON NOT NULL COMMENT 'Session data (JSON format)',
  `expires_at` TIMESTAMP NULL COMMENT 'When the session expires',
  `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT 'When the session was created',
  `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the session was last updated',
  INDEX idx_expires_at (expires_at),
  UNIQUE INDEX idx_public_id (`public_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Internal session store for the web framework runtime';