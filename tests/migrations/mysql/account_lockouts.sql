--  `user_id` BIGINT NOT NULL COMMENT 'Reference to users.id',
-- ==============================================
-- Table `account_lockouts`
-- Tracks temporary account lockouts due to too many failed attempts
-- ==============================================
CREATE TABLE IF NOT EXISTS `account_lockouts` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Auto-incrementing primary key',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `user_id` BIGINT NOT NULL COMMENT 'Reference to users.id',
  `ip_address` VARBINARY(16) NOT NULL COMMENT 'IP address where lockout occurred (binary format for both IPv4/IPv6)',
  `reason` VARCHAR(100) NOT NULL COMMENT 'Reason for lockout (e.g., too_many_attempts, suspicious_activity)',
  `locked_until` TIMESTAMP NOT NULL COMMENT 'When the account will be automatically unlocked',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the lockout was created',
  `unlocked_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the account was manually unlocked (if applicable)',
  `unlock_reason` VARCHAR(100) NULL DEFAULT NULL COMMENT 'Reason for manual unlock (if applicable)',
  `unlocked_by` BIGINT NULL DEFAULT NULL COMMENT 'Reference to users.id of admin who unlocked (if applicable)',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the lockout was soft-deleted',
  `locked_by` BIGINT NULL DEFAULT NULL COMMENT 'Reference to users.id of admin who locked (if applicable)',
  `created_by` BIGINT NULL COMMENT 'Reference to users.id (who created this lockout)',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the lockout record was last updated',
  `updated_by` BIGINT NULL COMMENT 'Reference to users.id (who last updated this lockout)',
  UNIQUE INDEX `idx_public_id` (`public_id`),
  INDEX `idx_user_id` (`user_id`),
  INDEX `idx_locked_until` (`locked_until`),
  
  CONSTRAINT `fk_lockout_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_lockout_unlocked_by` FOREIGN KEY (`unlocked_by`) REFERENCES `users` (`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Tracks temporary account lockouts due to too many failed attempts';
