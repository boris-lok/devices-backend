-- Add up migration script here
CREATE TABLE teams (
  id uuid not null,
  name varchar(1024) not null,
  description text,
  PRIMARY KEY(id)
);
