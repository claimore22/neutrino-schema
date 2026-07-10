-- ==============================================
-- Table `oauth_access_tokens`
-- OAuth access tokens for user and clients
-- ==============================================
CREATE TABLE IF NOT EXISTS `oauth_access_tokens` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `token` BINARY(32) NOT NULL COMMENT 'Hashed access token value (SHA-256 raw bytes)',
  `client_id` BIGINT NOT NULL COMMENT 'Reference to oauth_clients.id',
  `user_id` BIGINT NULL DEFAULT NULL COMMENT 'Reference to users.id (resource owner)',
  `scopes` VARCHAR(1000) NULL DEFAULT NULL COMMENT 'Space-separated list of authorized scopes',
  `revoked` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether the token has been revoked',
  `expires_at` TIMESTAMP NOT NULL COMMENT 'When the access token expires (REQUIRED for all tokens for security)',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the token was created',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the token was last updated',
  `last_used_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When this token was last used',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the token was soft-deleted',

  UNIQUE INDEX `idx_public_id` (`public_id`),
  UNIQUE KEY `uk_token` (`token`),
  INDEX `idx_client_id` (`client_id`),
  INDEX `idx_user_id` (`user_id`),
  INDEX `idx_expires_at` (`expires_at`),
  INDEX `idx_revoked` (`revoked`),

  CONSTRAINT `fk_token_client` FOREIGN KEY (`client_id`) REFERENCES `oauth_clients` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_token_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'OAuth access tokens for user and clients';

-- Trigger moved to: triggers/before_oauth_token_insert.sql