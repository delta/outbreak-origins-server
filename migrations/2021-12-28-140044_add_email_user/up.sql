-- Your SQL goes here
alter table users
add email text not null unique;
