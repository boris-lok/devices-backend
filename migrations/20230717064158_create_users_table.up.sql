-- Add up migration script here
create table users (
  id uuid not null,
  username varchar(256) not null,
  password_hash varchar(4096) not null,
  primary key(id)
);
