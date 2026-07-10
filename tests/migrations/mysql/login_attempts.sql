-- ==============================================
-- Table `login_attempts`
-- Tracks failed and successful login attempts for security and rate limiting
-- ==============================================
CREATE TABLE IF NOT EXISTS `login_attempts` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Auto-incrementing primary key',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `user_id` BIGINT NULL DEFAULT NULL COMMENT 'Reference to users.id (if user was identified)',
  `email` VARCHAR(100) NULL DEFAULT NULL COMMENT 'Email used in the login attempt',
  `ip_address` VARBINARY(16) NOT NULL COMMENT 'IP address where the attempt originated (binary format for both IPv4/IPv6)',
  `user_agent` VARCHAR(500) NULL DEFAULT NULL COMMENT 'User agent string from the browser',
  `attempted_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the login attempt occurred',
  `successful` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether the login attempt was successful',
  `failure_reason` VARCHAR(100) NULL DEFAULT NULL COMMENT 'Reason for failed login (e.g., invalid_credentials, account_locked, 2fa_required)',

  UNIQUE INDEX `idx_public_id` (`public_id`),
  INDEX `idx_user_id` (`user_id`),
  INDEX `idx_email` (`email`),
  INDEX `idx_ip` (`ip_address`),
  INDEX `idx_attempted_at` (`attempted_at`),

  CONSTRAINT fk_login_user FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Tracks failed and successful login attempts for security and rate limiting';
