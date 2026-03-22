-- Add vendor-owned bookstore fields to bookstores table
ALTER TABLE bookstores
  ADD COLUMN IF NOT EXISTS description     TEXT,
  ADD COLUMN IF NOT EXISTS genres          TEXT[],
  ADD COLUMN IF NOT EXISTS instagram       TEXT,
  ADD COLUMN IF NOT EXISTS website         TEXT,
  ADD COLUMN IF NOT EXISTS opening_hours   TEXT,
  ADD COLUMN IF NOT EXISTS banner_color    TEXT    NOT NULL DEFAULT '#1A1410',
  ADD COLUMN IF NOT EXISTS is_verified     BOOLEAN NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS is_published    BOOLEAN NOT NULL DEFAULT false;

-- All existing seed bookstores are published and verified
UPDATE bookstores
SET is_verified  = true,
    is_published = true;

-- Index for fast published-only queries
CREATE INDEX IF NOT EXISTS idx_bookstores_published
  ON bookstores(is_published)
  WHERE is_published = true;

-- Unique constraint on locations — safe to run multiple times
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
    WHERE conname = 'locations_city_district_unique'
  ) THEN
    ALTER TABLE locations
      ADD CONSTRAINT locations_city_district_unique UNIQUE (city, district);
  END IF;
END $$;