-- Your SQL goes here
alter table users
drop column username;

alter table users
add firstname text not null;

alter table users
add lastname text not null;
