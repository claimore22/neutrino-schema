-- neutrino-schema test fixture: MySQL
-- Covers all major MySQL types, constraints, and indexes.

CREATE TABLE users (
    id              INT             NOT NULL AUTO_INCREMENT,
    email           VARCHAR(255)    NOT NULL,
    full_name       VARCHAR(255)    NOT NULL,
    bio             TEXT,
    age             INT             NOT NULL DEFAULT 0,
    salary          DECIMAL(10,2),
    is_active       TINYINT(1)      NOT NULL DEFAULT 1,
    mood            ENUM('sad', 'ok', 'happy') NOT NULL DEFAULT 'ok',
    avatar          MEDIUMBLOB,
    metadata        JSON,
    created_at      DATETIME        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP       NULL DEFAULT NULL,
    birth_date      DATE,
    lunch_time      TIME,

    CONSTRAINT users_pkey PRIMARY KEY (id),
    CONSTRAINT users_email_unique UNIQUE(email),
    CONSTRAINT users_age_check CHECK(age >= 0)
) ENGINE=InnoDB;

CREATE TABLE posts (
    id              INT             NOT NULL AUTO_INCREMENT,
    user_id         INT             NOT NULL,
    title           VARCHAR(500)    NOT NULL,
    body            TEXT,
    is_active       TINYINT(1)      NOT NULL DEFAULT 1,
    created_at      DATETIME        NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT posts_pkey PRIMARY KEY (id),
    CONSTRAINT posts_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT posts_title_check CHECK(title <> '')
) ENGINE=InnoDB;

CREATE TABLE tags (
    id              INT             NOT NULL AUTO_INCREMENT,
    name            VARCHAR(100)    NOT NULL,
    description     TEXT,

    CONSTRAINT tags_pkey PRIMARY KEY (id),
    CONSTRAINT tags_name_unique UNIQUE(name)
) ENGINE=InnoDB;

CREATE TABLE post_tags (
    post_id         INT             NOT NULL,
    tag_id          INT             NOT NULL,

    CONSTRAINT post_tags_pkey PRIMARY KEY (post_id, tag_id),
    CONSTRAINT post_tags_post_id_fkey FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
    CONSTRAINT post_tags_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE TABLE profiles (
    id              INT             NOT NULL AUTO_INCREMENT,
    user_id         INT             NOT NULL,
    email           VARCHAR(255)    NOT NULL,
    display_name    VARCHAR(255)    NOT NULL,
    website         VARCHAR(500),
    avatar_url      TEXT,
    score           INT             NOT NULL DEFAULT 0,

    CONSTRAINT profiles_pkey PRIMARY KEY (id),
    CONSTRAINT profiles_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT profiles_email_unique UNIQUE(email)
) ENGINE=InnoDB;

CREATE TABLE all_types (
    id                  INT             NOT NULL,

    tiny_int_value      TINYINT,
    small_int_value     SMALLINT,
    medium_int_value    MEDIUMINT,
    integer_value       INT,
    bigint_value        BIGINT,

    decimal_value       DECIMAL(10,2),
    float_value         FLOAT,
    double_value        DOUBLE,

    varchar_value       VARCHAR(255),
    text_value          TEXT,

    enum_value          ENUM('sad', 'ok', 'happy'),

    json_value          JSON,
    blob_value          MEDIUMBLOB,

    datetime_value      DATETIME,
    timestamp_value     TIMESTAMP NULL DEFAULT NULL,

    CONSTRAINT all_types_pkey PRIMARY KEY (id)
) ENGINE=InnoDB;

-- Indexes
CREATE INDEX idx_users_created_at ON users(created_at);
CREATE INDEX idx_users_is_active ON users(is_active);
CREATE UNIQUE INDEX idx_users_email_lower ON users((LOWER(email)));
CREATE INDEX idx_posts_user_id_created ON posts(user_id, created_at DESC);
CREATE FULLTEXT INDEX idx_posts_body_fulltext ON posts(body);
CREATE INDEX idx_profiles_score ON profiles(score DESC);

-- Seed data: users
INSERT INTO users (id, email, full_name, bio, age, salary, is_active, mood, avatar, metadata, created_at, updated_at, birth_date, lunch_time)
VALUES
    (1, 'alice@example.com', 'Alice Johnson', 'Software engineer passionate about Rust.', 30, 75000.00, 1, 'happy', NULL, '{"theme": "dark", "language": "rust"}', '2024-01-15 08:00:00', '2024-06-01 12:00:00', '1994-03-20', '12:30:00'),
    (2, 'bob@example.com', 'Bob Smith', 'Designer and frontend developer.', 25, 50000.00, 1, 'ok', NULL, '{"theme": "light"}', '2024-03-20 10:30:00', NULL, '1999-07-15', '09:00:00');

-- Seed data: posts
INSERT INTO posts (id, user_id, title, body, is_active, created_at)
VALUES
    (1, 1, 'First Post', 'Hello world! This is the first post.', 1, '2024-01-16 09:00:00'),
    (2, 1, 'Second Post', 'Another post about nothing in particular.', 1, '2024-02-01 12:00:00'),
    (3, 2, 'Bob''s First', 'Bob enters the blogging world.', 1, '2024-03-21 14:00:00');

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
    tiny_int_value, small_int_value, medium_int_value, integer_value, bigint_value,
    decimal_value, float_value, double_value,
    varchar_value, text_value,
    enum_value,
    json_value,
    datetime_value
) VALUES (
    1,
    100, 1000, 50000, 2000000, 9000000000,
    1234.56, 3.14, 2.71828,
    'hello', 'This is a long text value.',
    'happy',
    '{"key": "value"}',
    '2024-06-15 14:30:00'
);
