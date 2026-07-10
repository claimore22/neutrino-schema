-- ==============================================
-- Table `oauth_refresh_tokens`
-- OAuth refresh tokens linked to access tokens
-- ==============================================
CREATE TABLE IF NOT EXISTS `oauth_refresh_tokens` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `token` BINARY(32) NOT NULL COMMENT 'Hashed refresh token value (SHA3-256 raw bytes)',
  `access_token_id` BIGINT NOT NULL COMMENT 'Reference to oauth_access_tokens.id that this refresh token is bound to',
  `previous_token_id` BIGINT NULL DEFAULT NULL COMMENT 'Reference to the previous refresh token (for refresh token rotation)',
  `revoked` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether the refresh token has been revoked',
  `expires_at` TIMESTAMP NOT NULL DEFAULT (CURRENT_TIMESTAMP + INTERVAL 30 DAY) COMMENT 'When the refresh token expires (default: 30 days from creation, must have an expiration for security)',
  `used_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When this refresh token was last used to obtain a new access token',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the refresh token was created',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the refresh token was last updated',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the token was soft-deleted',
  UNIQUE INDEX `idx_public_id` (`public_id`),
  INDEX `idx_access_token_id` (`access_token_id`),

  CONSTRAINT `fk_refresh_access_token` FOREIGN KEY (`access_token_id`) REFERENCES `oauth_access_tokens` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'OAuth refresh tokens linked to access tokens';
