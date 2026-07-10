-- ==============================================
-- Table `oauth_client_grants`
-- OAuth client grant types
-- ==============================================
CREATE TABLE IF NOT EXISTS `oauth_client_grants` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `client_id` BIGINT NOT NULL COMMENT 'Reference to oauth_clients.id',
  `grant_type` ENUM(
    'authorization_code',  -- Standard OAuth 2.0/2.1 authorization code flow with PKCE
    'client_credentials',  -- For machine-to-machine authentication
    'device_code',         -- For device authorization flow
    'refresh_token'        -- For token refresh (used with authorization_code)
  ) NOT NULL COMMENT 'OAuth 2.1 grant types for this client (excludes deprecated implicit/password grants)',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When this grant was created',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When this grant was last updated',
  `revoked_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When this grant was revoked, NULL means active',
  `revocation_reason` VARCHAR(255) NULL DEFAULT NULL COMMENT 'Reason for revocation',
  
  UNIQUE INDEX idx_public_id (`public_id`),
  UNIQUE KEY `uq_client_grant` (`client_id`, `grant_type`),
  CONSTRAINT `fk_grant_client` FOREIGN KEY (`client_id`) 
    REFERENCES `oauth_clients`(`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'OAuth client grant types for client (excludes deprecated implicit/password grants)';
