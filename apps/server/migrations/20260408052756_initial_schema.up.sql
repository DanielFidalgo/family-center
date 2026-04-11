CREATE SCHEMA IF NOT EXISTS family_center;
SET search_path TO family_center;

-- Households
CREATE TABLE IF NOT EXISTS households (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Devices
CREATE TABLE IF NOT EXISTS devices (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    household_id UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    name         TEXT NOT NULL DEFAULT 'Wall Display',
    last_seen_at TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- People
CREATE TABLE IF NOT EXISTS people (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    household_id UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    color        TEXT NOT NULL DEFAULT '#4A90D9',
    avatar_url   TEXT,
    sort_order   INTEGER NOT NULL DEFAULT 0,
    is_active    BOOLEAN NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_people_household ON people(household_id);

-- Google accounts
CREATE TABLE IF NOT EXISTS google_accounts (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    household_id     UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    email            TEXT NOT NULL,
    display_name     TEXT,
    avatar_url       TEXT,
    access_token     TEXT,
    refresh_token    TEXT,
    token_expires_at TIMESTAMPTZ,
    is_active        BOOLEAN NOT NULL DEFAULT TRUE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(household_id, email)
);

-- Calendar sources
CREATE TABLE IF NOT EXISTS calendar_sources (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    google_account_id UUID NOT NULL REFERENCES google_accounts(id) ON DELETE CASCADE,
    calendar_id       TEXT NOT NULL,
    name              TEXT NOT NULL,
    description       TEXT,
    color_hex         TEXT,
    is_selected       BOOLEAN NOT NULL DEFAULT FALSE,
    access_role       TEXT NOT NULL DEFAULT 'reader',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(google_account_id, calendar_id)
);

CREATE INDEX IF NOT EXISTS idx_calendar_sources_account ON calendar_sources(google_account_id);

-- Source events (synced from Google Calendar)
CREATE TABLE IF NOT EXISTS source_events (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    calendar_source_id UUID NOT NULL REFERENCES calendar_sources(id) ON DELETE CASCADE,
    google_event_id    TEXT NOT NULL,
    ical_uid           TEXT,
    title              TEXT NOT NULL,
    description        TEXT,
    location           TEXT,
    start_at           TIMESTAMPTZ NOT NULL,
    end_at             TIMESTAMPTZ NOT NULL,
    is_all_day         BOOLEAN NOT NULL DEFAULT FALSE,
    recurrence_rule    TEXT,
    recurring_event_id TEXT,
    organizer          TEXT,
    attendees          TEXT[],
    raw_json           JSONB NOT NULL DEFAULT '{}',
    synced_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(calendar_source_id, google_event_id)
);

CREATE INDEX IF NOT EXISTS idx_source_events_calendar ON source_events(calendar_source_id);
CREATE INDEX IF NOT EXISTS idx_source_events_start ON source_events(start_at);
CREATE INDEX IF NOT EXISTS idx_source_events_ical_uid ON source_events(ical_uid) WHERE ical_uid IS NOT NULL;

-- Merged event groups (deduplicated view)
CREATE TABLE IF NOT EXISTS merged_event_groups (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    household_id    UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    canonical_title TEXT NOT NULL,
    canonical_start TIMESTAMPTZ NOT NULL,
    canonical_end   TIMESTAMPTZ NOT NULL,
    is_all_day      BOOLEAN NOT NULL DEFAULT FALSE,
    person_id       UUID REFERENCES people(id) ON DELETE SET NULL,
    lane_override   UUID REFERENCES people(id) ON DELETE SET NULL,
    dupe_tier       TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_merged_groups_household ON merged_event_groups(household_id);
CREATE INDEX IF NOT EXISTS idx_merged_groups_start ON merged_event_groups(canonical_start);
CREATE INDEX IF NOT EXISTS idx_merged_groups_person ON merged_event_groups(person_id);

CREATE TABLE IF NOT EXISTS merged_event_sources (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merged_event_group_id UUID NOT NULL REFERENCES merged_event_groups(id) ON DELETE CASCADE,
    source_event_id       UUID NOT NULL REFERENCES source_events(id) ON DELETE CASCADE,
    is_primary            BOOLEAN NOT NULL DEFAULT FALSE,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(merged_event_group_id, source_event_id)
);

CREATE INDEX IF NOT EXISTS idx_merged_sources_group ON merged_event_sources(merged_event_group_id);

-- Lane assignment rules
CREATE TABLE IF NOT EXISTS lane_assignment_rules (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    household_id       UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    calendar_source_id UUID REFERENCES calendar_sources(id) ON DELETE CASCADE,
    email_pattern      TEXT,
    person_id          UUID REFERENCES people(id) ON DELETE CASCADE,
    lane_target        TEXT NOT NULL DEFAULT 'shared',
    priority           INTEGER NOT NULL DEFAULT 100,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_lane_rules_household ON lane_assignment_rules(household_id);

-- Local activities (user-defined, not from Google)
CREATE TABLE IF NOT EXISTS local_activities (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    household_id UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    person_id    UUID REFERENCES people(id) ON DELETE SET NULL,
    title        TEXT NOT NULL,
    description  TEXT,
    color        TEXT,
    start_at     TIMESTAMPTZ,
    end_at       TIMESTAMPTZ,
    is_all_day   BOOLEAN NOT NULL DEFAULT FALSE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_local_activities_household ON local_activities(household_id);
CREATE INDEX IF NOT EXISTS idx_local_activities_person ON local_activities(person_id);

CREATE TABLE IF NOT EXISTS local_activity_recurrences (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    local_activity_id UUID NOT NULL REFERENCES local_activities(id) ON DELETE CASCADE UNIQUE,
    freq              TEXT NOT NULL,
    interval_val      INTEGER NOT NULL DEFAULT 1,
    by_day_of_week    INTEGER[],
    by_day_of_month   INTEGER[],
    until             DATE,
    count             INTEGER,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Settings
CREATE TABLE IF NOT EXISTS settings (
    household_id       UUID PRIMARY KEY REFERENCES households(id) ON DELETE CASCADE,
    default_view       TEXT NOT NULL DEFAULT 'week',
    week_starts_monday BOOLEAN NOT NULL DEFAULT TRUE,
    dedupe_mode        TEXT NOT NULL DEFAULT 'exact_only',
    display_timezone   TEXT NOT NULL DEFAULT 'UTC',
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Sync checkpoints
CREATE TABLE IF NOT EXISTS sync_checkpoints (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    calendar_source_id UUID NOT NULL REFERENCES calendar_sources(id) ON DELETE CASCADE UNIQUE,
    sync_token         TEXT,
    full_sync_at       TIMESTAMPTZ,
    next_page_token    TEXT,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
