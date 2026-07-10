-- ==============================================
-- Table `ip_rate_limits`
-- Tracks rate limiting by IP address
-- ==============================================
CREATE TABLE IF NOT EXISTS `ip_rate_limits` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY COMMENT 'Internal auto-incrementing ID',
  `public_id` BINARY(16) NOT NULL DEFAULT (UUID_TO_BIN(UUID(), true)) COMMENT 'Public unique identifier (UUID v4 as binary)',
  `ip_address` VARBINARY(16) NOT NULL COMMENT 'IP address being rate limited (binary format for both IPv4/IPv6)',
  `endpoint` VARCHAR(255) NOT NULL COMMENT 'API or route endpoint being accessed',
  `attempts` INT UNSIGNED NOT NULL DEFAULT 1 COMMENT 'Number of attempts made',
  `first_attempt_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the first attempt was made',
  `last_attempt_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When the most recent attempt was made',
  `blocked_until` TIMESTAMP NULL DEFAULT NULL COMMENT 'When the IP will be unblocked (NULL if not blocked)',
  `block_reason` VARCHAR(100) NULL DEFAULT NULL COMMENT 'Reason for blocking (e.g., too_many_attempts, suspicious_activity)',

  UNIQUE INDEX `idx_public_id` (`public_id`),
  UNIQUE KEY `uq_ip_endpoint` (`ip_address`, `endpoint`),
  INDEX `idx_ip_blocked_until` (`ip_address`, `blocked_until`),
  INDEX `idx_last_attempt` (`last_attempt_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Tracks rate limiting by IP address to prevent abuse';
