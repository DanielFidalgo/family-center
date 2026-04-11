SET search_path TO family_center;

DROP TABLE IF EXISTS sync_checkpoints;
DROP TABLE IF EXISTS settings;
DROP TABLE IF EXISTS local_activity_recurrences;
DROP TABLE IF EXISTS local_activities;
DROP TABLE IF EXISTS lane_assignment_rules;
DROP TABLE IF EXISTS merged_event_sources;
DROP TABLE IF EXISTS merged_event_groups;
DROP TABLE IF EXISTS source_events;
DROP TABLE IF EXISTS calendar_sources;
DROP TABLE IF EXISTS google_accounts;
DROP TABLE IF EXISTS people;
DROP TABLE IF EXISTS devices;
DROP TABLE IF EXISTS households;

DROP SCHEMA IF EXISTS family_center CASCADE;
