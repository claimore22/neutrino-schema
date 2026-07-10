-- ==============================================
-- Table `user_roles`
-- Many-to-many relationship between users and roles
-- ==============================================
CREATE TABLE IF NOT EXISTS `user_roles` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `user_id` BIGINT NOT NULL COMMENT 'Reference to users.id',
  `role_id` TINYINT UNSIGNED NOT NULL COMMENT 'Reference to roles.id',
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the role was assigned to the user',
  `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the role assignment was last updated',
  `deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the role assignment was soft-deleted (NULL if active)',
  UNIQUE KEY `uk_user_role` (`user_id`, `role_id`),
  UNIQUE INDEX `idx_public_id` (`public_id`),
  CONSTRAINT `fk_user_roles_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_user_roles_role` FOREIGN KEY (`role_id`) REFERENCES `roles` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci 
COMMENT 'Many-to-many relationship between users and roles';
