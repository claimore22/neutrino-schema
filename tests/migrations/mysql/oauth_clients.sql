-- ==============================================
-- Table `oauth_clients`
-- OAuth client applications
-- ==============================================
CREATE TABLE IF NOT EXISTS `oauth_clients` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `user_id` BIGINT NULL DEFAULT NULL COMMENT 'Reference to users.id (owner of this client)',
  `name` VARCHAR(100) NOT NULL COMMENT 'Name of the OAuth client application',
  `secret` BINARY(32) NOT NULL COMMENT 'SHA3-256 hash of client secret',
  `secret_expires_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the client secret expires (NULL = never expires)',
  `provider` VARCHAR(50) NULL DEFAULT NULL COMMENT 'Name of the OAuth provider (if this is a provider client)',
  `personal_access_client` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether this is a personal access client',
  `pkce_required` BOOLEAN NOT NULL DEFAULT TRUE COMMENT 'Whether PKCE is required (recommended for all clients)',
  `revoked` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether the client has been revoked',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the client was created',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the client was last updated',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the client was soft-deleted (NULL = active)',
  UNIQUE INDEX `idx_public_id` (`public_id`),
  INDEX `idx_user_id` (`user_id`),
  CONSTRAINT `fk_client_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'OAuth clients, App-level (not user-specific)';
