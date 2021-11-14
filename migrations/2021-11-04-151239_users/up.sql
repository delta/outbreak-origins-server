-- Your SQL goes here

CREATE TABLE users (
  id serial primary key,
  username varchar(50) not null,
  password text,
  curlevel int not null default 0
);
