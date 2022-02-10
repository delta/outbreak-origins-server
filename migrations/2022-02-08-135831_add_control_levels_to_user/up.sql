-- Your SQL goes here
alter table users
add control_measure_level_data jsonb DEFAULT '{}'::jsonb NOT NULL;
