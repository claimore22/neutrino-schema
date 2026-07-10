-- ==============================================
-- Table `network_restrictions`
-- Current active network restrictions
-- ==============================================
CREATE TABLE IF NOT EXISTS `network_restrictions` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) UNIQUE COMMENT 'Public unique identifier (UUID v4 as binary)',
  `role_id` TINYINT UNSIGNED NOT NULL COMMENT 'Reference to roles.id (scope of this restriction)',
  `name` VARCHAR(100) NOT NULL COMMENT 'Name of the network restriction',
  `description` TEXT NULL COMMENT 'Description of the network restriction',
  `ip_ranges` JSON NOT NULL COMMENT 'Array of allowed IP ranges in CIDR notation',
  `countries` JSON NULL COMMENT 'Array of ISO 3166-1 alpha-2 country codes',
  `is_active` BOOLEAN NOT NULL DEFAULT TRUE COMMENT 'Whether this network restriction is active',
  `created_by` BIGINT NULL COMMENT 'Reference to users.id (who created this restriction)',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_by` BIGINT NULL COMMENT 'Reference to users.id (who last updated this restriction)',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `deleted_by` BIGINT NULL COMMENT 'Reference to users.id (who soft-deleted this restriction)',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the restriction was soft-deleted (NULL = active)',

  UNIQUE INDEX idx_public_id (`public_id`),

  CONSTRAINT `fk_network_restrictions_role`
    FOREIGN KEY (`role_id`) REFERENCES `roles` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_network_restrictions_created_by`
    FOREIGN KEY (`created_by`) REFERENCES `users` (`id`) ON DELETE SET NULL,
  CONSTRAINT `fk_network_restrictions_updated_by`
    FOREIGN KEY (`updated_by`) REFERENCES `users` (`id`) ON DELETE SET NULL,
  CONSTRAINT `fk_network_restrictions_deleted_by`
    FOREIGN KEY (`deleted_by`) REFERENCES `users` (`id`) ON DELETE SET NULL,

  KEY `idx_role_active` (`role_id`, `is_active`),
  KEY `idx_deleted_at` (`deleted_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Enforces network-level security for privileged users';