-- This file should undo anything in `up.sql`
alter table users
drop column lastname;

alter table users
drop column firstname;

alter table users
add email text not null unique;
