CREATE TABLE IF NOT EXISTS users (
     id SERIAL,
     name VARCHAR(255) NOT NULL
);

CREATE UNIQUE INDEX users__name ON users (name);