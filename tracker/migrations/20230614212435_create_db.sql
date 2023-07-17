-- Add migration script here
CREATE TABLE users (
    id            INTEGER PRIMARY KEY NOT NULL,
    username      VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name     VARCHAR(255) NOT NULL,
    created_at    DATETIME NOT NULL,
    updated_at    DATETIME NOT NULL,
    deleted_at    DATETIME
);

INSERT INTO users (id, username, password_hash, full_name, created_at, updated_at)
VALUES (1, "reinaldo", "$argon2id$v=19$m=16,t=2,p=1$UWJLcGRadENiUWhPM0FQbg$vWiBq1i2700tqPAiOsalfQ", "Reinaldo A. C. Rauch", datetime(), datetime());

CREATE TABLE torrents (
    id INTEGER PRIMARY KEY NOT NULL,
    short_name VARCHAR(255) NOT NULL,
    long_description TEXT,
    metainfo TEXT NOT NULL,

    created_at    DATETIME NOT NULL,
    updated_at    DATETIME NOT NULL,
    deleted_at    DATETIME
);