-- ==============================================
-- Table `users`
-- Main users table with denormalized last login fields and soft delete
-- ==============================================
CREATE TABLE IF NOT EXISTS `users` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
	`public_id` BINARY(16) NOT NULL DEFAULT (uuid_to_bin(uuid(),true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
	`first_name` VARCHAR(100) NOT NULL DEFAULT '' COMMENT 'User first name' COLLATE 'utf8mb4_unicode_ci',
	`last_name` VARCHAR(100) NOT NULL DEFAULT '' COMMENT 'User last name' COLLATE 'utf8mb4_unicode_ci',
	`email` VARCHAR(100) NOT NULL COMMENT 'User email address (must be unique)' COLLATE 'utf8mb4_unicode_ci',
	`is_verified` TINYINT(1) NOT NULL DEFAULT '0' COMMENT 'Account verification status (false for unverified accounts)',
	`email_verified_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'Timestamp when email was verified (NULL if not verified)',
	`password` VARCHAR(255) NULL DEFAULT NULL COMMENT 'Argon2 hashed password (NULL for OAuth-only users)' COLLATE 'utf8mb4_unicode_ci',
	`remember_token` VARCHAR(100) NULL DEFAULT NULL COMMENT 'Token for "remember me" functionality' COLLATE 'utf8mb4_unicode_ci',
	`user_type` TINYINT UNSIGNED NOT NULL DEFAULT '0' COMMENT 'User role: 0=regular user, 1=admin, 2=moderator, etc.',
	`is_active` TINYINT(1) NOT NULL DEFAULT '1' COMMENT 'Account activation status (false for deactivated accounts)',
	`last_login_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'Timestamp of most recent successful login',
	`last_login_ip` VARBINARY(16) NULL DEFAULT NULL COMMENT 'IP address of most recent login (binary format for both IPv4/IPv6)',
	`created_at` TIMESTAMP NOT NULL DEFAULT (CURRENT_TIMESTAMP) COMMENT 'Timestamp when user account was created',
	`updated_at` TIMESTAMP NOT NULL DEFAULT (CURRENT_TIMESTAMP) ON UPDATE CURRENT_TIMESTAMP COMMENT 'Timestamp when user record was last updated',
	`deleted_at` TIMESTAMP NULL DEFAULT NULL COMMENT 'Timestamp when account was soft-deleted (NULL if active)',
  
  INDEX `idx_email_active` (`email`, `is_active`),
  UNIQUE INDEX `idx_email_unique` (`email`),
  INDEX `idx_deleted_at` (`deleted_at`),
  INDEX `idx_email_verified_at` (`email_verified_at`),
  UNIQUE INDEX `idx_public_id` (`public_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Main users table with denormalized last login fields and soft delete';
