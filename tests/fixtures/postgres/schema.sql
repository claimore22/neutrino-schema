-- neutrino-schema test fixture: PostgreSQL
-- Covers all major PostgreSQL types, constraints, indexes, and ENUM support.

CREATE TYPE mood AS ENUM ('sad', 'ok', 'happy');

CREATE TABLE users (
    id              INTEGER PRIMARY KEY,
    email           VARCHAR(255) NOT NULL,
    full_name       TEXT NOT NULL,
    bio             TEXT,
    age             INTEGER NOT NULL DEFAULT 0,
    salary          NUMERIC(10,2),
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    mood            mood NOT NULL DEFAULT 'ok',
    avatar          BYTEA,
    metadata        JSONB,
    preferences     JSON,
    home_ip         INET,
    uuid_col        UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP,
    birth_date      DATE,
    lunch_time      TIME,

    CONSTRAINT users_email_unique UNIQUE(email),
    CONSTRAINT users_age_check CHECK(age >= 0)
);

CREATE TABLE posts (
    id              INTEGER PRIMARY KEY,
    user_id         INTEGER NOT NULL,
    title           VARCHAR(500) NOT NULL,
    body            TEXT,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    tags            TEXT[],
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT posts_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT posts_title_check CHECK(title <> '')
);

CREATE TABLE tags (
    id              INTEGER PRIMARY KEY,
    name            VARCHAR(100) NOT NULL,
    description     TEXT,

    CONSTRAINT tags_name_unique UNIQUE(name)
);

CREATE TABLE post_tags (
    post_id         INTEGER NOT NULL,
    tag_id          INTEGER NOT NULL,

    CONSTRAINT post_tags_pkey PRIMARY KEY (post_id, tag_id),
    CONSTRAINT post_tags_post_id_fkey FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
    CONSTRAINT post_tags_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE TABLE profiles (
    id              INTEGER PRIMARY KEY,
    user_id         INTEGER NOT NULL,
    email           VARCHAR(255) NOT NULL,
    display_name    TEXT NOT NULL,
    website         TEXT,
    avatar_url      TEXT,
    score           INTEGER NOT NULL DEFAULT 0,

    CONSTRAINT profiles_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT profiles_email_unique UNIQUE(email)
);

CREATE TABLE all_types (
    id                  INTEGER PRIMARY KEY,

    small_int_value     SMALLINT,
    integer_value       INTEGER,
    bigint_value        BIGINT,

    serial_value        SERIAL,
    bigserial_value     BIGSERIAL,

    numeric_value       NUMERIC(10,2),
    real_value          REAL,
    double_value        DOUBLE PRECISION,

    varchar_value       VARCHAR(100),
    text_value          TEXT,

    boolean_value       BOOLEAN,
    bytea_value         BYTEA,

    date_value          DATE,
    time_value          TIME,
    timestamp_value     TIMESTAMP,
    timestamptz_value   TIMESTAMPTZ,

    json_value          JSON,
    jsonb_value         JSONB,

    uuid_value          UUID,
    inet_value          INET,

    mood_value          mood,
    text_array_value    TEXT[]
);

-- Indexes
CREATE INDEX idx_users_created_at ON users(created_at);
CREATE INDEX idx_users_is_active ON users(is_active) WHERE is_active = TRUE;
CREATE UNIQUE INDEX idx_users_lower_email ON users(LOWER(email));
CREATE INDEX idx_posts_user_id_created ON posts(user_id, created_at DESC);
CREATE INDEX idx_posts_metadata ON posts USING GIN (metadata);
CREATE INDEX idx_profiles_score ON profiles(score DESC);

-- Seed data: users
INSERT INTO users (id, email, full_name, bio, age, salary, is_active, mood, avatar, metadata, preferences, home_ip, uuid_col, created_at, updated_at, birth_date, lunch_time)
VALUES
    (1, 'alice@example.com', 'Alice Johnson', 'Software engineer passionate about Rust.', 30, 75000.00, TRUE, 'happy', NULL, '{"theme": "dark", "language": "rust"}', '{"notifications": true}', '192.168.1.100', 'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', '2024-01-15T08:00:00Z', '2024-06-01 12:00:00', '1994-03-20', '12:30:00'),
    (2, 'bob@example.com', 'Bob Smith', 'Designer and frontend developer.', 25, 50000.00, TRUE, 'ok', '\xdeadbeef'::bytea, '{"theme": "light"}', NULL, '10.0.0.1', 'b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', '2024-03-20T10:30:00Z', NULL, '1999-07-15', '09:00:00');

-- Seed data: posts
INSERT INTO posts (id, user_id, title, body, is_active, tags, created_at)
VALUES
    (1, 1, 'First Post', 'Hello world! This is the first post.', TRUE, ARRAY['hello', 'introduction'], '2024-01-16T09:00:00Z'),
    (2, 1, 'Second Post', 'Another post about nothing in particular.', TRUE, ARRAY['random'], '2024-02-01T12:00:00Z'),
    (3, 2, 'Bob''s First', 'Bob enters the blogging world.', TRUE, ARRAY['hello'], '2024-03-21T14:00:00Z');

-- Seed data: tags
INSERT INTO tags (id, name, description)
VALUES
    (1, 'hello', 'Greetings and introductions'),
    (2, 'introduction', 'First posts and welcomes'),
    (3, 'random', 'Miscellaneous content'),
    (4, 'tech', 'Technology-related posts');

-- Seed data: post_tags
INSERT INTO post_tags (post_id, tag_id) VALUES
    (1, 1), (1, 2), (2, 3), (3, 1);

-- Seed data: profiles
INSERT INTO profiles (id, user_id, email, display_name, website, avatar_url, score)
VALUES
    (1, 1, 'alice@example.com', 'Alice', 'https://alice.dev', NULL, 100),
    (2, 2, 'bob@example.com', 'Bob', NULL, 'https://avatars.example.com/bob', 42);

-- Seed data: all_types
INSERT INTO all_types (
    id,
    small_int_value, integer_value, bigint_value,
    numeric_value, real_value, double_value,
    varchar_value, text_value,
    boolean_value,
    date_value, time_value, timestamp_value, timestamptz_value,
    json_value, jsonb_value,
    uuid_value, inet_value,
    mood_value, text_array_value
) VALUES (
    1,
    100, 2000000, 9000000000,
    1234.56, 3.14, 2.71828,
    'hello', 'This is a long text value.',
    TRUE,
    '2024-06-15', '14:30:00', '2024-06-15 14:30:00', '2024-06-15 14:30:00+00',
    '{"key": "value"}', '{"nested": {"count": 42}}',
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', '192.168.1.100',
    'happy', ARRAY['a', 'b', 'c']
);
