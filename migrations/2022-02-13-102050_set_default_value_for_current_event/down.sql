-- This file should undo anything in `up.sql`
alter table status
alter current_event drop default;

alter table regions
alter simulation_params drop default;
