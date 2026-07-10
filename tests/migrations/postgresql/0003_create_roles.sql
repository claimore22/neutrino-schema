-- ==============================================
-- Table roles
-- User role definitions
-- ==============================================
CREATE TABLE IF NOT EXISTS roles (
  id SMALLINT NOT NULL,
  public_id UUID NOT NULL DEFAULT gen_random_uuid(),
  name VARCHAR(50) NOT NULL UNIQUE,
  description TEXT DEFAULT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by BIGINT DEFAULT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_by BIGINT DEFAULT NULL,
  deleted_at TIMESTAMPTZ DEFAULT NULL,
  deleted_by BIGINT DEFAULT NULL,

  PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_public_id ON roles(public_id);

COMMENT ON TABLE roles IS 'User role definitions';
COMMENT ON COLUMN roles.id IS 'Internal ID: 0=regular user, 1=admin, 2=moderator, etc.';
