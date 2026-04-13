-- Create person_claim_tokens table for QR-based claim flow
CREATE TABLE IF NOT EXISTS family_center.person_claim_tokens (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id  UUID NOT NULL REFERENCES family_center.people(id) ON DELETE CASCADE UNIQUE,
    token      TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_claim_tokens_token
    ON family_center.person_claim_tokens(token);
