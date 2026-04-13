DROP TABLE IF EXISTS family_center.activity_completions;
ALTER TABLE family_center.local_activities
    DROP COLUMN IF EXISTS category,
    DROP COLUMN IF EXISTS is_time_bound;
