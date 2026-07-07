-- SaaS Subscription Platform
--
-- Run this against PostgreSQL to create a demo database:
--   psql -U postgres -d saas_demo -f schema.sql
--
-- Then introspect with neutrino-schema:
--   neutrino-schema generate --database-url "postgres://localhost/saas_demo" --output src/models

CREATE TABLE users (
    id            BIGSERIAL    PRIMARY KEY,
    email         VARCHAR(255) NOT NULL UNIQUE,
    name          VARCHAR(255) NOT NULL,
    avatar_url    TEXT,
    is_active     BOOLEAN      NOT NULL DEFAULT true,
    metadata      JSONB,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE TABLE teams (
    id            BIGSERIAL    PRIMARY KEY,
    owner_id      BIGINT       NOT NULL REFERENCES users(id),
    name          VARCHAR(255) NOT NULL,
    slug          VARCHAR(100) NOT NULL UNIQUE,
    billing_email VARCHAR(255),
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE TABLE team_members (
    id            BIGSERIAL    PRIMARY KEY,
    team_id       BIGINT       NOT NULL REFERENCES teams(id),
    user_id       BIGINT       NOT NULL REFERENCES users(id),
    role          VARCHAR(50)  NOT NULL DEFAULT 'member',
    joined_at     TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE TABLE plans (
    id            BIGSERIAL    PRIMARY KEY,
    name          VARCHAR(255) NOT NULL,
    description   TEXT,
    price_cents   INTEGER      NOT NULL,
    currency      VARCHAR(3)   NOT NULL DEFAULT 'USD',
    interval      VARCHAR(20)  NOT NULL DEFAULT 'monthly',
    features      JSONB,
    is_public     BOOLEAN      NOT NULL DEFAULT true,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE TABLE subscriptions (
    id                    BIGSERIAL    PRIMARY KEY,
    team_id               BIGINT       NOT NULL REFERENCES teams(id),
    plan_id               BIGINT       NOT NULL REFERENCES plans(id),
    status                VARCHAR(50)  NOT NULL DEFAULT 'active',
    current_period_start  TIMESTAMPTZ  NOT NULL,
    current_period_end    TIMESTAMPTZ  NOT NULL,
    trial_ends_at         TIMESTAMPTZ,
    cancelled_at          TIMESTAMPTZ,
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE TABLE invoices (
    id                BIGSERIAL    PRIMARY KEY,
    subscription_id   BIGINT       NOT NULL REFERENCES subscriptions(id),
    amount_cents      INTEGER      NOT NULL,
    currency          VARCHAR(3)   NOT NULL DEFAULT 'USD',
    status            VARCHAR(50)  NOT NULL DEFAULT 'pending',
    paid_at           TIMESTAMPTZ,
    due_at            TIMESTAMPTZ,
    pdf_url           TEXT,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE TABLE api_keys (
    id            BIGSERIAL    PRIMARY KEY,
    team_id       BIGINT       NOT NULL REFERENCES teams(id),
    key_hash      VARCHAR(64)  NOT NULL,
    label         VARCHAR(255) NOT NULL,
    last_used_at  TIMESTAMPTZ,
    expires_at    TIMESTAMPTZ,
    is_revoked    BOOLEAN      NOT NULL DEFAULT false,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now()
);
