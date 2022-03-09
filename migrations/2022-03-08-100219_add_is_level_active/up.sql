-- Your SQL goes here
ALTER TABLE users
   ADD COLUMN is_level_active BOOLEAN NOT NULL DEFAULT FALSE;
