-- ==============================================
-- Table `user_trusted_devices`
-- Optional table for users to remember and manage trusted devices
-- ==============================================
CREATE TABLE IF NOT EXISTS `user_trusted_devices` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `user_id` BIGINT NOT NULL COMMENT 'Reference to users.id',
  `device_id` BINARY(16) NOT NULL COMMENT 'Unique device identifier',
  `device_name` VARCHAR(100) NULL COMMENT 'User-defined device name (e.g., My Laptop, Phone)',
  `device_type` VARCHAR(50) NULL COMMENT 'Device type (e.g., mobile, tablet, desktop)',
  `os` VARCHAR(50) NULL COMMENT 'Operating system',
  `browser` VARCHAR(100) NULL COMMENT 'Browser name and version',
  `last_used_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the device was last used',
  `expires_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When this device trust expires (NULL = never)',
  `is_revoked` BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether the device has been revoked',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the device was trusted',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the device record was last updated',
  `created_by` BIGINT NULL COMMENT 'Reference to users.id (who created this device rule)',
  `updated_by` BIGINT NULL COMMENT 'Reference to users.id (who last updated this device rule)',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the device rule was soft-deleted (NULL if active)',
  `deleted_by` BIGINT NULL COMMENT 'Reference to users.id (who soft-deleted this device rule)',
  
  UNIQUE KEY `uq_user_device` (`user_id`, `device_id`),
  UNIQUE INDEX `idx_public_id` (`public_id`),
  KEY `idx_user_active` (`user_id`, `is_revoked`),
  KEY `idx_expires` (`expires_at`),
  CONSTRAINT `fk_trusted_device_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Optional trusted devices for user accounts';

-- Add index for device lookups
CREATE INDEX `idx_device_id` ON `user_trusted_devices` (`device_id`);

-- Add index for user and device lookups
CREATE INDEX IF NOT EXISTS `idx_trusted_device` 
ON `user_trusted_devices` (`user_id`, `device_id`);