-- ==============================================
-- Table `email_verification_tokens`
-- Stores email verification tokens with expiration and rate limiting
-- ==============================================
CREATE TABLE IF NOT EXISTS `email_verification_tokens` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `user_id` BIGINT NOT NULL COMMENT 'Reference to users.id',
  `token` BINARY(32) NOT NULL COMMENT 'SHA3-256 hashed token (raw bytes)',
  `expires_at` TIMESTAMP NOT NULL DEFAULT (CURRENT_TIMESTAMP + INTERVAL 24 HOUR) COMMENT 'When this token expires (24 hours from creation)',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the token was created',
  `updated_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the record was last updated',
  `ip_address` VARBINARY(16) NULL DEFAULT NULL COMMENT 'IP address that requested the verification',
  `user_agent` VARCHAR(255) NULL DEFAULT NULL COMMENT 'User agent that requested the reset',
  `used_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the token was used (NULL if unused)',
  `email` VARCHAR(255) NOT NULL COMMENT 'Email address being verified',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the token was soft-deleted',

  UNIQUE INDEX `idx_public_id` (`public_id`),
  UNIQUE KEY `uk_token` (`token`),
  INDEX `idx_user` (`user_id`),
  INDEX `idx_expires` (`expires_at`),
  INDEX `idx_used` (`used_at`),
  INDEX `idx_email_verification_rate_limit` (`ip_address`, `created_at`),
  INDEX `idx_email_rate_limit` (`email`, `created_at`),
  CONSTRAINT `fk_verification_token_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Stores email verification tokens with expiration and usage tracking';

-- Trigger moved to: triggers/before_email_verification_token_insert.sql
