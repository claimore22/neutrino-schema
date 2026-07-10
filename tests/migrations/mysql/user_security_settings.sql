-- ==============================================
-- Table `user_security_settings`
-- Separate table for 2FA and account lockout status
-- ==============================================
CREATE TABLE IF NOT EXISTS `user_security_settings` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `user_id` BIGINT NOT NULL COMMENT 'Reference to users.id',
  `two_factor_enabled` TINYINT(1) NOT NULL DEFAULT 0 COMMENT 'Whether 2FA is enabled for this user',
  `two_factor_secret` VARCHAR(255) DEFAULT NULL COMMENT 'TOTP secret for 2FA',
  `two_factor_recovery_codes` TEXT DEFAULT NULL COMMENT 'JSON array of recovery codes',
  `two_factor_confirmed_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When 2FA was confirmed',
  `failed_login_attempts` INT UNSIGNED NOT NULL DEFAULT 0 COMMENT 'Number of consecutive failed login attempts',
  `locked_until` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the account is locked until (if locked due to too many failed attempts)',
  `last_password_change` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the password was last changed',
  `account_locked_until` TIMESTAMP NULL DEFAULT NULL COMMENT 'Timestamp until account is locked (NULL if not locked)',
  `lock_reason` VARCHAR(255) NULL DEFAULT NULL COMMENT 'Reason for account lock (e.g., too many failed attempts)',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When these security settings were created',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When these settings were last updated',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the security settings were soft-deleted (NULL = active)',
  CONSTRAINT `fk_security_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE,
  UNIQUE KEY `uk_user_id` (`user_id`),
  UNIQUE INDEX `idx_public_id` (`public_id`),
  INDEX `idx_two_factor_enabled` (`two_factor_enabled`),
  INDEX `idx_locked_until` (`locked_until`),
  INDEX `idx_account_locked_until` (`account_locked_until`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Stores security-related settings for users including 2FA and account lockout status';
