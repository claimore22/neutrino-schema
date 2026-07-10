-- ==============================================
-- Table `security_events`
-- Logs security relevant events for auditing
-- ==============================================
CREATE TABLE IF NOT EXISTS `security_events` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Auto-incrementing primary key',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `user_id` BIGINT NULL DEFAULT NULL COMMENT 'Reference to users.id (NULL for system events)',
  `event_type` VARCHAR(50) NOT NULL COMMENT 'Type of security event (e.g., login, password_change, 2fa_enabled, login_failed, account_locked)',
  `ip_address` VARBINARY(16) NULL DEFAULT NULL COMMENT 'IP address where event originated (binary format for both IPv4/IPv6)',
  `user_agent` VARCHAR(500) NULL DEFAULT NULL COMMENT 'User agent string from the browser',
  `metadata` TEXT NULL DEFAULT NULL COMMENT 'Additional JSON or serialized data with event details',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the event occurred',

  UNIQUE INDEX idx_public_id (`public_id`),
  INDEX idx_user_id (`user_id`),
  INDEX idx_event_type (`event_type`),
  INDEX idx_created_at (`created_at`),

  CONSTRAINT fk_sec_event_user FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Logs security relevant events for auditing';
