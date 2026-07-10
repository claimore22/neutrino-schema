-- ==============================================
-- Table `password_reset_tokens`
-- Stores password reset tokens with expiration and usage tracking
-- ==============================================
CREATE TABLE IF NOT EXISTS `password_reset_tokens` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `user_id` BIGINT NOT NULL COMMENT 'Reference to users.id',
  `token` BINARY(32) NOT NULL COMMENT 'Hashed token value (SHA3-256 raw bytes)',
  `expires_at` TIMESTAMP NOT NULL DEFAULT (CURRENT_TIMESTAMP + INTERVAL 1 HOUR) COMMENT 'When this token expires (default is 1 hour from creation)',
  `used_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When this token was used (NULL if unused)',
  `ip_address` VARBINARY(16) NULL DEFAULT NULL COMMENT 'IP address that requested the reset',
  `user_agent` VARCHAR(255) NULL DEFAULT NULL COMMENT 'User agent that requested the reset',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the token was created',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the token was last updated',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the token was soft-deleted',

  UNIQUE INDEX `idx_public_id` (`public_id`),
  UNIQUE KEY `uk_token` (`token`),
  KEY `idx_user` (`user_id`),
  KEY `idx_expires` (`expires_at`),
  KEY `idx_used` (`used_at`),
  KEY `idx_ip_created` (`ip_address`, `created_at`),
  CONSTRAINT `fk_password_reset_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Stores password reset tokens with expiration and usage tracking';

-- Trigger moved to: triggers/before_password_reset_token_insert.sql
