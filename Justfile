set dotenv-load := true
set dotenv-path := ".env"

server_dir := "apps/server"

# Read DATABASE_URL directly from .env so it always wins over shell env
db_url := `grep '^DATABASE_URL=' .env | cut -d= -f2-`

# List available recipes
default:
    @just --list

# ── Database ──────────────────────────────────────────────────────────────────

# Run pending migrations
migrate:
    cd {{server_dir}} && DATABASE_URL="{{db_url}}" cargo sqlx migrate run

# Roll back the last migration
migrate-down:
    cd {{server_dir}} && DATABASE_URL="{{db_url}}" cargo sqlx migrate revert

# Show migration status
migrate-status:
    cd {{server_dir}} && DATABASE_URL="{{db_url}}" cargo sqlx migrate info

# ── sqlx ──────────────────────────────────────────────────────────────────────

# Generate .sqlx offline query cache (required after adding/changing queries)
prepare:
    cd {{server_dir}} && DATABASE_URL="{{db_url}}" cargo sqlx prepare

# ── Build ─────────────────────────────────────────────────────────────────────

# Check compilation without producing a binary
check:
    cd {{server_dir}} && SQLX_OFFLINE=true cargo check

# Build the server (debug)
build:
    cd {{server_dir}} && SQLX_OFFLINE=true cargo build

# Build the server (release)
build-release:
    cd {{server_dir}} && SQLX_OFFLINE=true cargo build --release

# ── Run ───────────────────────────────────────────────────────────────────────

# Run the server locally (debug)
dev:
    cd {{server_dir}} && DATABASE_URL="{{db_url}}" cargo run

# ── Test ──────────────────────────────────────────────────────────────────────

# Run all tests
test:
    cd {{server_dir}} && DATABASE_URL="{{db_url}}" cargo test

# ── Lint ──────────────────────────────────────────────────────────────────────

# Run clippy
lint:
    cd {{server_dir}} && SQLX_OFFLINE=true cargo clippy -- -D warnings

# ── Setup ─────────────────────────────────────────────────────────────────────

# First-time setup: migrate then generate sqlx cache
setup: migrate prepare
    @echo "Setup complete. Run 'just dev' to start the server."

# ── Ephemeral DB (testcontainer-style) ────────────────────────────────────

container_name := "family-center-sqlx-dev"
local_db_url   := "postgres://family:family@localhost:54329/family_center?sslmode=disable"

# Spin up a throwaway Postgres, migrate, generate .sqlx cache, tear down
sqlx-cache:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "▸ Starting ephemeral Postgres…"
    docker rm -f {{container_name}} 2>/dev/null || true
    docker run -d --name {{container_name}} \
      -e POSTGRES_USER=family \
      -e POSTGRES_PASSWORD=family \
      -e POSTGRES_DB=family_center \
      -p 54329:5432 \
      postgres:16-alpine \
      > /dev/null

    cleanup() { echo "▸ Tearing down…"; docker rm -f {{container_name}} > /dev/null 2>&1 || true; }
    trap cleanup EXIT

    echo "▸ Waiting for Postgres to be ready…"
    for i in $(seq 1 30); do
      docker exec {{container_name}} pg_isready -U family -d family_center > /dev/null 2>&1 && break
      sleep 0.5
    done

    echo "▸ Applying schema…"
    docker exec -i {{container_name}} psql -U family -d family_center < {{server_dir}}/migrations/20260408052756_initial_schema.up.sql

    echo "▸ Generating .sqlx offline cache…"
    cd {{server_dir}} && DATABASE_URL="{{local_db_url}}" cargo sqlx prepare

    echo "✓ .sqlx cache generated. Builds will now use offline mode."
