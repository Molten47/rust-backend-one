-- ── Performance indexes ─────────────────────────────────────────
-- Run: sqlx migrate run
-- These indexes are safe to add on a live database — CREATE INDEX CONCURRENTLY
-- would be preferred in production but requires a transaction-free context.
-- For local/small DB, standard CREATE INDEX IF NOT EXISTS is fine.

-- Orders: cursor pagination reads — user_id + placed_at DESC is the hot path
CREATE INDEX IF NOT EXISTS idx_orders_user_placed
    ON orders(user_id, placed_at DESC);

-- Orders: status filter combined with user + placed_at
CREATE INDEX IF NOT EXISTS idx_orders_user_status_placed
    ON orders(user_id, status, placed_at DESC);

-- Books: in-stock browsing sorted by rating — the main feed query
-- Partial index (WHERE in_stock = true) is smaller and faster than full index
CREATE INDEX IF NOT EXISTS idx_books_instock_rating
    ON books(rating DESC, id)
    WHERE in_stock = true;

-- Books: category filter + rating sort
CREATE INDEX IF NOT EXISTS idx_books_category_rating
    ON books(category_id, rating DESC)
    WHERE in_stock = true;

-- Cart items: user lookup via cart join — hot path on every page load
CREATE INDEX IF NOT EXISTS idx_cart_items_cart_id
    ON cart_items(cart_id);

-- Carts: user lookup
CREATE INDEX IF NOT EXISTS idx_carts_user_id
    ON carts(user_id);

-- Order items: order lookup
CREATE INDEX IF NOT EXISTS idx_order_items_order_id
    ON order_items(order_id);

-- Events: upcoming filter — no partial index since now() is volatile in Postgres
-- Just index starts_at + is_free so the WHERE clause in queries is fast
CREATE INDEX IF NOT EXISTS idx_events_starts_at_free
    ON events(starts_at ASC, is_free);

-- Saved addresses: user default lookup
CREATE INDEX IF NOT EXISTS idx_saved_addresses_user_default
    ON saved_addresses(user_id, is_default DESC);