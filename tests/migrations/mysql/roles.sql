-- ==============================================
-- Table `roles`
-- User role definitions
-- ==============================================
CREATE TABLE IF NOT EXISTS `roles` (
  `id` TINYINT UNSIGNED NOT NULL PRIMARY KEY,
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `name` VARCHAR(50) NOT NULL UNIQUE COMMENT 'Unique role name identifier',
  `description` TEXT NULL COMMENT 'Description of the role',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the role was created',
  `created_by` BIGINT NULL COMMENT 'Reference to users.id (who created this role)',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the role was last updated',
  `updated_by` BIGINT NULL COMMENT 'Reference to users.id (who last updated this role)',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the role was soft-deleted (NULL if active)',
  `deleted_by` BIGINT NULL COMMENT 'Reference to users.id (who soft-deleted this role)',

  UNIQUE INDEX `idx_public_id` (`public_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci 
COMMENT 'User role definitions';
