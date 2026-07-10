-- ==============================================
-- Table `user_allowed_ips`
-- Optional table for users to whitelist specific IPs, should be mandatory for privileged users
-- ==============================================
CREATE TABLE IF NOT EXISTS `user_allowed_ips` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `user_id` BIGINT NOT NULL COMMENT 'Reference to users.id',
  `ip_address` VARBINARY(16) NOT NULL COMMENT 'Allowed IP address (binary format for both IPv4/IPv6)',
  `label` VARCHAR(100) NULL COMMENT 'User-defined label for the IP (e.g., Home, Office)',
  `is_active` BOOLEAN NOT NULL DEFAULT TRUE COMMENT 'Whether this IP rule is active',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the IP was whitelisted',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the IP rule was last updated',
  `created_by` BIGINT NULL COMMENT 'Reference to users.id (who created this IP rule)',
  `updated_by` BIGINT NULL COMMENT 'Reference to users.id (who last updated this IP rule)',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the IP rule was soft-deleted (NULL if active)',
  `deleted_by` BIGINT NULL COMMENT 'Reference to users.id (who soft-deleted this IP rule)',
  
  UNIQUE KEY `uq_user_ip` (`user_id`, `ip_address`),
  UNIQUE INDEX `idx_public_id` (`public_id`),
  KEY `idx_user_active` (`user_id`, `is_active`),
  CONSTRAINT `fk_allowed_ip_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Optional IP whitelist for user accounts, should be mandatory for privileged users';

-- Add index for IP lookups
CREATE INDEX `idx_ip_address` ON `user_allowed_ips` (`ip_address`);
