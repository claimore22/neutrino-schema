-- ==============================================
-- Table `oauth_accounts`
-- Links user accounts to OAuth providers
-- ==============================================
CREATE TABLE IF NOT EXISTS `oauth_accounts` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `user_id` BIGINT NOT NULL COMMENT 'Reference to users.id',
  `provider_id` BIGINT NOT NULL COMMENT 'Reference to oauth_providers.id',
  `provider_user_id` VARCHAR(255) NOT NULL COMMENT 'User ID from the OAuth provider',
  `access_token` VARCHAR(512) NULL DEFAULT NULL COMMENT 'Reference to encrypted access token (not stored in plaintext)  Encrypted with AES-256-GCM ?',
  `refresh_token` VARCHAR(512) NULL DEFAULT NULL COMMENT 'Reference to encrypted refresh token (not stored in plaintext) Encrypted with AES-256-GCM ?',
  `token_expires_at` TIMESTAMP NOT NULL COMMENT 'When the access token expires',
  `token_issued_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the access token was issued',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When this OAuth account link was created',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When this OAuth account was last updated',
  `created_by` BIGINT NULL COMMENT 'Reference to users.id (who created this OAuth account link)',
  `updated_by` BIGINT NULL COMMENT 'Reference to users.id (who last updated this OAuth account link)',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the OAuth account link was soft-deleted (NULL if active)',
  `deleted_by` BIGINT NULL COMMENT 'Reference to users.id (who soft-deleted this OAuth account link)',
  UNIQUE INDEX idx_public_id (`public_id`),
  UNIQUE KEY uq_provider_user (`provider_id`, `provider_user_id`),
  INDEX idx_user_id (`user_id`),

  CONSTRAINT fk_oauth_user FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE,
  CONSTRAINT fk_oauth_provider FOREIGN KEY (`provider_id`) REFERENCES `oauth_providers` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Links user accounts to OAuth providers (e.g., Google, GitHub). User-specific';



-- No Token Storage: Only hashes are stored
-- One-Time View: User sees the token exactly once
-- Secure Verification: Constant-time comparison prevents timing attacks
-- Future-Proof: SHA3-256 is quantum-resistant