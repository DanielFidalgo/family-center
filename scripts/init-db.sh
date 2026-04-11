#!/usr/bin/env bash
set -e

DB_URL="${DATABASE_URL:-postgres://family:family@localhost:5432/family_center}"

echo "Creating database..."
psql "${DB_URL%/*}" -c "CREATE DATABASE family_center;" 2>/dev/null || echo "Database already exists"

echo "Running migrations..."
cd apps/server
export DATABASE_URL="$DB_URL"
cargo sqlx migrate run

echo "Done."
