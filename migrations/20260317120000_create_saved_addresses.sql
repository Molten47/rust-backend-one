-- ── Saved Addresses ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS saved_addresses (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    label        TEXT        NOT NULL DEFAULT 'Home',
    address      TEXT        NOT NULL,
    city         TEXT        NOT NULL,
    phone        TEXT        NOT NULL,
    is_default   BOOLEAN     NOT NULL DEFAULT false,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_saved_addresses_user_id  ON saved_addresses(user_id);
CREATE INDEX IF NOT EXISTS idx_saved_addresses_default  ON saved_addresses(user_id, is_default);

-- Ensure only one default per user via partial unique index
CREATE UNIQUE INDEX IF NOT EXISTS idx_saved_addresses_one_default
    ON saved_addresses(user_id)
    WHERE is_default = true;