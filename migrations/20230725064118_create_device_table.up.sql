-- Add up migration script here
CREATE TABLE devices (
  id uuid not null,
  name varchar(1024) not null,
  owner_id uuid not null,
  board varchar(128),
  sn varchar(128),
  barcode varchar(128),
  received_date timestamptz,
  hw_phase varchar(128),
  note text,
  PRIMARY KEY(id)
);
