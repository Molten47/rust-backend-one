-- Add cover_url column to books — nullable so emoji fallback still works
ALTER TABLE books ADD COLUMN IF NOT EXISTS cover_url TEXT;

-- Index for quick null checks
CREATE INDEX IF NOT EXISTS idx_books_cover_url ON books(cover_url) WHERE cover_url IS NOT NULL;