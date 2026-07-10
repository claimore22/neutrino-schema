-- ==============================================
-- Table `oauth_device_codes`
-- Implements OAuth 2.0 Device Authorization Grant (RFC 8628)
-- ==============================================
CREATE TABLE IF NOT EXISTS `oauth_device_codes` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `device_code` VARCHAR(100) NOT NULL UNIQUE COMMENT 'The device verification code (stored hashed)',
  `user_code` VARCHAR(50) NOT NULL UNIQUE COMMENT 'The end-user verification code (short, user-typable code)',
  `client_id` BIGINT NOT NULL COMMENT 'Reference to oauth_clients.id',
  `user_id` BIGINT NULL DEFAULT NULL COMMENT 'Reference to users.id (populated after user authenticates)',
  `scopes` VARCHAR(1000) NULL DEFAULT NULL COMMENT 'Space-separated list of requested scopes',
  `ip_address` VARBINARY(16) NULL DEFAULT NULL COMMENT 'IP address where the device code was requested (binary format for both IPv4/IPv6)',
  `user_agent` VARCHAR(500) NULL DEFAULT NULL COMMENT 'User agent from the device making the request',
  `device_name` VARCHAR(255) NULL DEFAULT NULL COMMENT 'User-friendly name of the device (if provided)',
  `expires_at` TIMESTAMP NOT NULL COMMENT 'When the device code expires (typically 30 minutes from creation)',
  `last_poll` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the device last polled for authorization',
  `poll_interval` INT UNSIGNED NOT NULL DEFAULT 5 COMMENT 'Minimum seconds between polling requests (for rate limiting)',
  `status` ENUM('pending', 'approved', 'denied', 'expired') NOT NULL DEFAULT 'pending' COMMENT 'Current status of the device authorization',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the device code was created',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the device code was last updated',
  `approved_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the user approved the device',
  `denied_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the user denied the device',

  UNIQUE INDEX idx_public_id (`public_id`),
  INDEX idx_device_code (`device_code`),
  INDEX idx_user_code (`user_code`),
  INDEX idx_client_id (`client_id`),
  INDEX idx_user_id (`user_id`),
  INDEX idx_status (`status`),
  INDEX idx_expires_at (`expires_at`),

  CONSTRAINT fk_device_client FOREIGN KEY (`client_id`) REFERENCES `oauth_clients` (`id`) ON DELETE CASCADE,
  CONSTRAINT fk_device_user FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Implements OAuth 2.0 Device Authorization Grant (RFC 8628)';
