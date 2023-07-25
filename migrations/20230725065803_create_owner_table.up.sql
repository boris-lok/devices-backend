-- Add up migration script here
CREATE TABLE owners (
  id uuid not null,
  name varchar(1024) not null,
  description varchar(4096),
  PRIMARY KEY(id)
);
