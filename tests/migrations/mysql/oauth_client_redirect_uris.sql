-- ==============================================
-- Table `oauth_client_redirect_uris`
-- Permitted OAuth redirect URIs per client
-- ==============================================
CREATE TABLE IF NOT EXISTS `oauth_client_redirect_uris` (
  `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `client_id` BIGINT NOT NULL,
  `redirect_uri` VARCHAR(766) NOT NULL,
  `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT `fk_uri_client` FOREIGN KEY (`client_id`) REFERENCES `oauth_clients` (`id`) ON DELETE CASCADE,
  UNIQUE INDEX `uq_client_redirect` (`client_id`, `redirect_uri`),
  INDEX `idx_client_id` (`client_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Permitted OAuth redirect URIs per client';
