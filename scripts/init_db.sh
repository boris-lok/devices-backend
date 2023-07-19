#!/usr/bin/env bash 

# If we want to skip crate another dockerized `Postgres`
# We can run `SKIP_DOCKER=true ./scripts/init_db.sh` 

# `set -x` enables a mode of the shell where executed commands
# are printed to the terminal.
set -x
# `set -e` causes a script to exit if any of the processes it
# calls generate a non-zero return.
# `set -o` and `pipefail` to pipe the contents of a file that
# doesn't exist
set -eo pipefail

# Check if the command `psql` exist
if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  exit 1
fi

# Check if the command `sqlx` exist
if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "    cargo install --version=0.6.0 sqlx-cli --no-default-features --features postgres"
  echo >&2 "to install it."
  exit 1
fi

# Check if a custom user has been set, otherwise default to 'postgres'
DB_USER=${POSTGRES_USER:=postgres}
# Check if a custom password has been set, otherwise default to 'yeework'
DB_PASSWORD=${POSTGRES_PASSWORD:=yeework}
# Check if a custom database has been set, otherwise default to 'devices'
DB_NAME=${POSTGRES_DB:=devices}
# Check if a custom port has been set, otherwise default to '25432'
DB_PORT=${POSTGRES_PORT:=25432}
# Check if a custom host has been set, otherwise default to '127.0.0.1'
DB_HOST=${POSTGRES_HOST:=127.0.0.1}

# Allow to skip Docker if a dockerized `Postgres` database is already running
if [[ -z "${SKIP_DOCKER}" ]]
then
  # Launch postgres using Docker
docker run \
  -e POSTGRES_USER=${DB_USER} \
  -e POSTGRES_PASSWORD=${DB_PASSWORD} \
  -e POSTGRES_DB=${DB_NAME} \
  -p "${DB_PORT}":5432 \
  -d postgres \
  postgres -N 1000
  # ^ Increased maximum number of connections for testing purposes.
fi

# Keep pinging `Postgres` until it's ready to accept commands
until PGPASSWORD="${DB_PASSWORD}" psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c "\q"; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT} - running migrations now!"

# set environment `DATABASE_URL` for migration
export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@127.0.0.1:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"
