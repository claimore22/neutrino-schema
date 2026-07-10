-- ==============================================
-- Table `personal_access_tokens`
-- API tokens for user authentication
-- ==============================================
CREATE TABLE IF NOT EXISTS `personal_access_tokens` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `user_id` BIGINT NOT NULL COMMENT 'Reference to users.id',
  `name` VARCHAR(100) NOT NULL COMMENT 'Descriptive name for the token (shown to user)',
  `token` BINARY(32) NOT NULL COMMENT 'Hashed token value (SHA3-256 raw bytes)',
  `abilities` VARCHAR(1000) NULL DEFAULT NULL COMMENT 'JSON array of abilities/permissions granted to this token',
  `last_used_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When this token was last used',
  `expires_at` TIMESTAMP NOT NULL COMMENT 'When this token expires (REQUIRED for security, should be a reasonable duration)',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the token was created',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the token was last updated',
  `revoked` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether the token has been revoked',
  `last_used_ip` VARBINARY(16) NULL COMMENT 'IP address where the token was last used',
  `last_used_user_agent` VARCHAR(255) NULL COMMENT 'User agent where the token was last used',
  `revoked_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the token was revoked',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the token was soft-deleted',
  
  UNIQUE INDEX `idx_public_id` (`public_id`),
  UNIQUE KEY `uk_token` (`token`),
  INDEX `idx_user_id` (`user_id`),
  INDEX `idx_expires_at` (`expires_at`),
  INDEX `idx_revoked` (`revoked`),
  CONSTRAINT `fk_pat_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'API tokens for user authentication';

-- Trigger moved to: triggers/before_personal_token_insert.sql
