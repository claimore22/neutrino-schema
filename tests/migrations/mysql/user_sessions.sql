-- ==============================================
-- Table `user_sessions`
-- User sessions table
-- Application-level sessions
-- ==============================================
CREATE TABLE IF NOT EXISTS `user_sessions` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)),
  `user_id` BIGINT NOT NULL COMMENT 'FK to users.id',
  `session_id` BINARY(32) NOT NULL UNIQUE COMMENT 'Secure random 32-byte ID',
  `device_id` BINARY(16) NULL COMMENT 'FK to user_trusted_devices.device_id',
  `ip_address` VARBINARY(16) NULL COMMENT 'IP address where session was created (binary format for both IPv4/IPv6)',
  `user_agent` VARCHAR(500) NULL COMMENT 'User agent string from the browser',
  `user_agent_hash` BINARY(32) NULL COMMENT 'SHA3-256 hash of user agent',
  `payload` VARBINARY(5000) NOT NULL COMMENT 'Encrypted session blob',
  `is_revoked` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether the session has been revoked',
  `is_2fa_verified` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether 2FA has been verified for this session',
  `last_activity` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the session was last active',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the session was created',
  `expires_at` TIMESTAMP NOT NULL DEFAULT (CURRENT_TIMESTAMP + INTERVAL 30 MINUTE) COMMENT 'When the session expires',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the session was last updated',
  `metadata` JSON DEFAULT NULL COMMENT 'Additional metadata',
  `revoked_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the session was revoked',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the session was deleted',  
  
  UNIQUE INDEX `idx_public_id` (`public_id`),
  INDEX `idx_user_last_activity` (`user_id`, `last_activity`),
  INDEX `idx_user_active` (`user_id`, `is_revoked`, `expires_at`),
  INDEX `idx_expiring_sessions` (`expires_at`),
  INDEX `idx_device` (`user_id`, `device_id`),

  CONSTRAINT `fk_sessions_user`
    FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_sessions_device`
    FOREIGN KEY (`user_id`, `device_id`)
    REFERENCES `user_trusted_devices` (`user_id`, `device_id`)
    ON DELETE CASCADE
) ENGINE=InnoDB
  DEFAULT CHARSET=utf8mb4
  COLLATE=utf8mb4_unicode_ci;
