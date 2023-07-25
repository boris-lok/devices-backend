-- Add up migration script here
CREATE TABLE device_types (
  id uuid not null,
  name varchar(128) not null,
  PRIMARY KEY(id)
);
