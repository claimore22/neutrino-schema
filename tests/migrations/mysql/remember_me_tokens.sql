-- ==============================================
-- Table `remember_me_tokens`
-- Stores persistent login tokens for "Remember Me" functionality
-- ==============================================
CREATE TABLE IF NOT EXISTS `remember_me_tokens` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `user_id` BIGINT NOT NULL COMMENT 'Reference to users.id',
  `device_id` BINARY(32) NULL COMMENT 'Optional device identifier (SHA3-256 raw bytes)',
  `series` BINARY(32) NOT NULL COMMENT 'Series identifier for token rotation (SHA3-256 raw bytes)',
  `token` BINARY(32) NOT NULL COMMENT 'Hashed token value (SHA3-256 raw bytes)',
  `expires_at` TIMESTAMP NOT NULL COMMENT 'When this token expires',
  `last_used` TIMESTAMP NULL DEFAULT NULL COMMENT 'When this token was last used',
  `user_agent` VARCHAR(255) NULL COMMENT 'User agent string (truncated)',
  `ip_address` VARBINARY(16) NULL COMMENT 'IP address (supports both IPv4 and IPv6)',
  `network` VARCHAR(100) NULL COMMENT 'Network identifier (e.g., WiFi name, mobile network)',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the token was soft-deleted (NULL if active)',
  
  -- Indexes
  UNIQUE INDEX `idx_public_id` (`public_id`),
  UNIQUE KEY `uk_remember_me_series_token` (`series`, `token`),
  INDEX `idx_remember_me_tokens_user` (`user_id`),
  INDEX `idx_remember_me_tokens_series` (`series`),
  INDEX `idx_remember_me_tokens_expiry` (`expires_at`),
  INDEX `idx_remember_me_tokens_device` (`device_id`) USING HASH,
  
  -- Foreign key
  CONSTRAINT `fk_remember_me_tokens_user` 
    FOREIGN KEY (`user_id`) 
    REFERENCES `users` (`id`) 
    ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci 
COMMENT='Stores persistent login tokens for Remember Me functionality';

-- Triggers moved to separate files:
--   - triggers/before_remember_me_token_insert.sql
--   - triggers/update_remember_me_token_last_used.sql