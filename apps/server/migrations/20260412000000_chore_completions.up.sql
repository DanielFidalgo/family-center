-- Add category and time-bound flag to local activities
ALTER TABLE family_center.local_activities
    ADD COLUMN category TEXT,
    ADD COLUMN is_time_bound BOOLEAN NOT NULL DEFAULT FALSE;

-- Track per-date completions for recurring chores
CREATE TABLE IF NOT EXISTS family_center.activity_completions (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    local_activity_id UUID NOT NULL REFERENCES family_center.local_activities(id) ON DELETE CASCADE,
    completed_date    DATE NOT NULL,
    completed_by      UUID REFERENCES family_center.people(id) ON DELETE SET NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(local_activity_id, completed_date)
);

CREATE INDEX IF NOT EXISTS idx_activity_completions_date
    ON family_center.activity_completions(local_activity_id, completed_date);
