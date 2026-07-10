-- ==============================================
-- Table `oauth_authorization_codes`
-- Temporary auth codes for OAuth PKCE flow
-- ==============================================
CREATE TABLE IF NOT EXISTS `oauth_authorization_codes` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `code` BINARY(32) NOT NULL COMMENT 'The hashed authorization code issued to the client (SHA3-256 raw bytes)',
  `client_id` BIGINT NOT NULL COMMENT 'Reference to oauth_clients.id',
  `user_id` BIGINT NULL DEFAULT NULL COMMENT 'Reference to users.id (resource owner)',
  `redirect_uri` VARCHAR(2000) NULL DEFAULT NULL COMMENT 'Redirect URI used in the authorization request',
  `expires_at` TIMESTAMP NOT NULL DEFAULT (CURRENT_TIMESTAMP + INTERVAL 10 MINUTE) COMMENT 'When this authorization code expires (default: 10 minutes from creation)',
  `code_challenge` VARCHAR(128) NULL DEFAULT NULL COMMENT 'PKCE code challenge (base64url encoded)',
  `code_challenge_method` ENUM('plain', 'S256') NULL DEFAULT NULL COMMENT 'PKCE code challenge method (S256 recommended)',
  `scope` VARCHAR(1000) NULL DEFAULT NULL COMMENT 'Space-separated list of authorized scopes',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the authorization code was created',
  `consumed` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether this code has been used to obtain tokens',

  UNIQUE INDEX `idx_public_id` (`public_id`),
  INDEX `idx_client_id` (`client_id`),
  INDEX `idx_user_id` (`user_id`),

  CONSTRAINT `fk_authcode_client` FOREIGN KEY (`client_id`) REFERENCES `oauth_clients` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_authcode_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Temporary auth codes for OAuth PKCE flow';
