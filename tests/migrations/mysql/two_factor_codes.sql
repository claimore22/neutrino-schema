-- ==============================================
-- Table `two_factor_codes`
-- Stores two-factor authentication codes for users
-- ==============================================
CREATE TABLE IF NOT EXISTS `two_factor_codes` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Auto-incrementing primary key',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `user_id` BIGINT NOT NULL COMMENT 'Reference to users.id',
  `code` VARCHAR(100) NOT NULL COMMENT 'Hashed verification code',
  `expires_at` TIMESTAMP NOT NULL DEFAULT ((now() + interval 10 minute)) COMMENT 'When this code expires (default is 10 minutes from creation)',
  `used_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When this code was used (NULL if unused)',
  `ip_address` VARBINARY(16) NULL DEFAULT NULL COMMENT 'IP address that requested the code',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the code was created',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the code was last updated',
  
  UNIQUE INDEX `idx_public_id` (`public_id`),
  KEY `idx_user_code` (`user_id`, `code`),
  KEY `idx_expires` (`expires_at`),
  CONSTRAINT `fk_two_factor_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Stores two-factor authentication codes with expiration and usage tracking';
