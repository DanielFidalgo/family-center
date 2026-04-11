#!/usr/bin/env bash
set -e
echo "Starting family-center dev environment..."
echo ""
echo "1. Ensure PostgreSQL is running"
echo "2. Copy .env.example to apps/server/.env and fill in values"
echo "3. Run: bash scripts/init-db.sh"
echo "4. Run: npm run dev:server (terminal 1)"
echo "5. Run: npm run dev:mobile (terminal 2)"
echo ""
echo "Or use: npm run dev (starts both with concurrently)"
