-- This file should undo anything in `up.sql`
ALTER TABLE users
DROP COLUMN curr_level_score;
