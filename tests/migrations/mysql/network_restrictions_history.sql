
-- ==============================================
-- Table `network_restrictions_history`
-- Audit log of all changes to network_restrictions
-- ==============================================
CREATE TABLE IF NOT EXISTS `network_restrictions_history` (
  `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `restriction_id` BIGINT NOT NULL COMMENT 'Reference to network_restrictions.id',
  `action` ENUM('CREATE','UPDATE','DELETE') NOT NULL COMMENT 'Type of action performed',
  `old_value` JSON NULL COMMENT 'Previous state (before update/delete)',
  `new_value` JSON NULL COMMENT 'New state (after create/update)',
  `performed_by` BIGINT NULL COMMENT 'Reference to users.id (who performed this action)',
  `performed_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  UNIQUE INDEX idx_public_id (`public_id`),

  CONSTRAINT `fk_history_restriction`
    FOREIGN KEY (`restriction_id`) REFERENCES `network_restrictions` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_history_user`
    FOREIGN KEY (`performed_by`) REFERENCES `users` (`id`) ON DELETE SET NULL,

  KEY `idx_restriction_action_time` (`restriction_id`, `action`, `performed_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Audit log of changes to network restrictions';
