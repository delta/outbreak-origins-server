-- Your SQL goes here
ALTER TABLE users
ADD COLUMN retryAttemptsLeft INT NOT NULL DEFAULT 3;
