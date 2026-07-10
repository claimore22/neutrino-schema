-- neutrino-schema test fixture: SQLite
-- Note: PRAGMA foreign_keys = ON must be executed by the test runner before loading this fixture.
-- Covers major SQLite type affinities, constraints, and indexes.

CREATE TABLE users (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    email           TEXT    NOT NULL,
    full_name       TEXT    NOT NULL,
    bio             TEXT,
    age             INTEGER NOT NULL DEFAULT 0,
    salary          REAL,
    is_active       INTEGER NOT NULL DEFAULT 1,
    mood            TEXT    NOT NULL DEFAULT 'ok' CHECK(mood IN ('sad', 'ok', 'happy')),
    avatar          BLOB,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT,
    birth_date      TEXT,
    lunch_time      TEXT,

    CONSTRAINT users_email_unique UNIQUE(email),
    CONSTRAINT users_age_check CHECK(age >= 0)
);

CREATE TABLE posts (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         INTEGER NOT NULL,
    title           TEXT    NOT NULL,
    body            TEXT,
    is_active       INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),

    CONSTRAINT posts_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT posts_title_check CHECK(title <> '')
);

CREATE TABLE tags (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT    NOT NULL,
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
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         INTEGER NOT NULL,
    email           TEXT    NOT NULL,
    display_name    TEXT    NOT NULL,
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

    real_value          REAL,

    text_value          TEXT,
    varchar_value       VARCHAR(100),

    blob_value          BLOB,

    json_value          JSON,
    date_value          DATE,
    datetime_value      DATETIME
);

-- Indexes (SQLite does not support DESC or partial)
CREATE INDEX idx_users_created_at ON users(created_at);
CREATE UNIQUE INDEX idx_users_email_lower ON users(LOWER(email));
CREATE INDEX idx_posts_user_id_created ON posts(user_id, created_at);
CREATE INDEX idx_profiles_score ON profiles(score);

-- Seed data: users
INSERT INTO users (id, email, full_name, bio, age, salary, is_active, mood, avatar, created_at, updated_at, birth_date, lunch_time)
VALUES
    (1, 'alice@example.com', 'Alice Johnson', 'Software engineer passionate about Rust.', 30, 75000.0, 1, 'happy', NULL, '2024-01-15T08:00:00', '2024-06-01T12:00:00', '1994-03-20', '12:30:00'),
    (2, 'bob@example.com', 'Bob Smith', 'Designer and frontend developer.', 25, 50000.0, 1, 'ok', NULL, '2024-03-20T10:30:00', NULL, '1999-07-15', '09:00:00');

-- Seed data: posts
INSERT INTO posts (id, user_id, title, body, is_active, created_at)
VALUES
    (1, 1, 'First Post', 'Hello world! This is the first post.', 1, '2024-01-16T09:00:00'),
    (2, 1, 'Second Post', 'Another post about nothing in particular.', 1, '2024-02-01T12:00:00'),
    (3, 2, 'Bob''s First', 'Bob enters the blogging world.', 1, '2024-03-21T14:00:00');

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
    real_value,
    text_value, varchar_value,
    json_value, date_value, datetime_value
) VALUES (
    1,
    100, 2000000, 9000000000,
    3.14,
    'This is a long text value.', 'hello',
    '{"key": "value"}', '2024-06-15', '2024-06-15 14:30:00'
);
