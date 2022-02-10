-- This file should undo anything in `up.sql`
alter table users
drop column control_measure_level_data;
