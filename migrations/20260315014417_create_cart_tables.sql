-- ── Carts ────────────────────────────────────────────────────────
CREATE TABLE carts (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    status      VARCHAR(20) NOT NULL DEFAULT 'active'
                CHECK (status IN ('active', 'checked_out', 'abandoned')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Cart Items ───────────────────────────────────────────────────
CREATE TABLE cart_items (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cart_id        UUID NOT NULL REFERENCES carts(id) ON DELETE CASCADE,
    book_id        UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    quantity       INTEGER NOT NULL DEFAULT 1 CHECK (quantity > 0),
    price_snapshot NUMERIC(10, 2) NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (cart_id, book_id)
);

-- ── Wishlists ────────────────────────────────────────────────────
CREATE TABLE wishlists (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Wishlist Items ───────────────────────────────────────────────
CREATE TABLE wishlist_items (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wishlist_id UUID NOT NULL REFERENCES wishlists(id) ON DELETE CASCADE,
    book_id     UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    added_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (wishlist_id, book_id)
);

-- ── Indexes ──────────────────────────────────────────────────────
CREATE INDEX idx_cart_items_cart_id    ON cart_items(cart_id);
CREATE INDEX idx_cart_items_book_id    ON cart_items(book_id);
CREATE INDEX idx_wishlist_items_wl_id  ON wishlist_items(wishlist_id);

-- ── Auto-update updated_at ───────────────────────────────────────
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER carts_updated_at
    BEFORE UPDATE ON carts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER cart_items_updated_at
    BEFORE UPDATE ON cart_items
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();