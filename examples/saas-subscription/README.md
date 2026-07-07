# SaaS Subscription Platform — neutrino-schema demo

Seven PostgreSQL tables → fully typed Rust models in one command.

## Before: 200 lines of SQL

```sql
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
    ...
);

CREATE TABLE subscriptions (
    id                    BIGSERIAL    PRIMARY KEY,
    team_id               BIGINT       NOT NULL REFERENCES teams(id),
    plan_id               BIGINT       NOT NULL REFERENCES plans(id),
    ...
);
-- 4 more tables …
```

## After: generated Rust models

```
src/models/
├── mod.rs
├── users.rs
├── teams.rs
├── team_members.rs
├── plans.rs
├── subscriptions.rs
├── invoices.rs
└── api_keys.rs
```

```rust
// src/models/users.rs
#[derive(Debug, Clone)]
pub struct Users {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// src/models/teams.rs
#[derive(Debug, Clone)]
pub struct Teams {
    pub id: i64,
    pub owner_id: i64,
    pub name: String,
    pub slug: String,
    pub billing_email: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// src/models/subscriptions.rs
#[derive(Debug, Clone)]
pub struct Subscriptions {
    pub id: i64,
    pub team_id: i64,
    pub plan_id: i64,
    pub status: String,
    pub current_period_start: chrono::DateTime<chrono::Utc>,
    pub current_period_end: chrono::DateTime<chrono::Utc>,
    pub trial_ends_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

Relationships detected automatically:

```
teams.owner_id → users.id
team_members.team_id → teams.id
team_members.user_id → users.id
subscriptions.team_id → teams.id
subscriptions.plan_id → plans.id
invoices.subscription_id → subscriptions.id
api_keys.team_id → teams.id
```

## Try it yourself

```bash
# 1. Create the database
createdb saas_demo
psql -d saas_demo -f schema.sql

# 2. Generate models
neutrino-schema generate \
    --database-url "postgres://localhost/saas_demo" \
    --output src/models

# 3. Use them in your project
# cargo add neutrino-schema chrono uuid serde_json
```

## Supported databases

The same schema works with PostgreSQL, MySQL/MariaDB, or SQLite.
Just change the `--database-url`:

```bash
# PostgreSQL
neutrino-schema generate --database-url "postgres://localhost/saas_demo" --output src/models

# MySQL
neutrino-schema generate --database-url "mysql://root:pass@localhost/saas_demo" --output src/models

# SQLite
neutrino-schema generate --database-url "sqlite:./saas_demo.db" --output src/models
```

## What `neutrino-schema` handled automatically

- **7 tables → 7 Rust files** — zero manual typing
- **Foreign keys → `i64` relation fields** — no manual mapping
- **Nullable columns → `Option<T>`** — nullability encoded in the type system
- **Timestamp columns → `chrono::DateTime<chrono::Utc>`** — idiomatic Rust types
- **JSONB columns → `serde_json::Value`** — zero-config JSON support
- **BOOLEAN → `bool`** — not integer, not magic strings
- **`_id` suffix heuristic** — relationship inference without FK queries
