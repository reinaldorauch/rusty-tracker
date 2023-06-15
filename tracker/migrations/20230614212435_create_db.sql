-- Add migration script here
CREATE TABLE users (
    id            INTEGER PRIMARY KEY NOT NULL,
    username      VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL
);