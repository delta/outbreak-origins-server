DROP TABLE IF EXISTS regions_status;
DROP TABLE IF EXISTS status;
DROP TABLE IF EXISTS regions;

ALTER TABLE users
DROP COLUMN IF EXISTS status;
