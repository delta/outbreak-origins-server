-- This file should undo anything in `up.sql`
ALTER TABLE users
   drop column is_level_active;
