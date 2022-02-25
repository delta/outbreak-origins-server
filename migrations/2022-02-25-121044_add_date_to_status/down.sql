-- This file should undo anything in `up.sql`
ALTER TABLE status
DROP COLUMN cur_date;
