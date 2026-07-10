-- ==============================================
-- Table `oauth_providers`
-- OAuth identity providers
-- ==============================================
CREATE TABLE IF NOT EXISTS `oauth_providers` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `name` VARCHAR(50) NOT NULL UNIQUE COMMENT 'Unique provider identifier (e.g., google, github, microsoft)',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the provider was added to the system',
  `created_by` BIGINT NULL COMMENT 'Reference to users.id (who created this provider)',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the provider record was last updated',
  `updated_by` BIGINT NULL COMMENT 'Reference to users.id (who last updated this provider)',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the provider was soft-deleted (NULL = active)',
  `deleted_by` BIGINT NULL COMMENT 'Reference to users.id (who soft-deleted this provider)',
  UNIQUE INDEX idx_public_id (`public_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'OAuth identity providers';

-- -- Then insert the default providers
-- INSERT IGNORE INTO `oauth_providers` (`name`) VALUES 
-- ('local'),
-- ('google'),
-- ('github'),
-- ('microsoft'),
-- ('facebook');