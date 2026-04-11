#!/usr/bin/env bash
set -e

DB_URL="${DATABASE_URL:-postgres://family:family@localhost:5432/family_center}"

echo "Seeding dev data..."
psql "$DB_URL" << 'SQL'
INSERT INTO households (id, name) VALUES
  ('00000000-0000-0000-0000-000000000001', 'Demo Family')
ON CONFLICT DO NOTHING;

INSERT INTO people (id, household_id, name, color, sort_order) VALUES
  ('00000000-0000-0000-0000-000000000010', '00000000-0000-0000-0000-000000000001', 'Alice', '#4A90D9', 1),
  ('00000000-0000-0000-0000-000000000011', '00000000-0000-0000-0000-000000000001', 'Bob', '#E85555', 2),
  ('00000000-0000-0000-0000-000000000012', '00000000-0000-0000-0000-000000000001', 'Charlie', '#50C878', 3)
ON CONFLICT DO NOTHING;

INSERT INTO settings (household_id, default_view, week_starts_monday, dedupe_mode) VALUES
  ('00000000-0000-0000-0000-000000000001', 'week', true, 'exact_only')
ON CONFLICT DO NOTHING;
SQL

echo "Seed complete."
