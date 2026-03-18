-- ── Orders ──────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS orders (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bookstore_id  UUID        NOT NULL REFERENCES bookstores(id),
    status        TEXT        NOT NULL DEFAULT 'pending'
                              CHECK (status IN ('pending','confirmed','preparing','in_transit','delivered','cancelled')),
    total_amount  NUMERIC(10,2) NOT NULL,
    delivery_fee  NUMERIC(10,2) NOT NULL DEFAULT 0,
    address       TEXT        NOT NULL,
    notes         TEXT,
    placed_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS order_items (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id   UUID        NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    book_id    UUID        NOT NULL REFERENCES books(id),
    quantity   INT         NOT NULL DEFAULT 1 CHECK (quantity > 0),
    unit_price NUMERIC(10,2) NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_orders_user_id      ON orders(user_id);
CREATE INDEX IF NOT EXISTS idx_orders_status       ON orders(status);
CREATE INDEX IF NOT EXISTS idx_order_items_order   ON order_items(order_id);

-- ── Events ───────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS events (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    title        TEXT        NOT NULL,
    description  TEXT,
    location     TEXT        NOT NULL,
    bookstore_id UUID        REFERENCES bookstores(id) ON DELETE SET NULL,
    starts_at    TIMESTAMPTZ NOT NULL,
    ends_at      TIMESTAMPTZ,
    cover_url    TEXT,
    is_free      BOOLEAN     NOT NULL DEFAULT true,
    price        NUMERIC(10,2),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_events_starts_at    ON events(starts_at);
CREATE INDEX IF NOT EXISTS idx_events_bookstore    ON events(bookstore_id);

-- ── Vendor Applications ──────────────────────────────────────────

CREATE TABLE IF NOT EXISTS vendor_applications (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    store_name      TEXT        NOT NULL,
    store_address   TEXT        NOT NULL,
    city            TEXT        NOT NULL,
    phone           TEXT        NOT NULL,
    description     TEXT,
    instagram       TEXT,
    website         TEXT,
    status          TEXT        NOT NULL DEFAULT 'pending'
                                CHECK (status IN ('pending','reviewing','approved','rejected')),
    submitted_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_at     TIMESTAMPTZ,
    reviewer_notes  TEXT
);

CREATE INDEX IF NOT EXISTS idx_vendor_apps_user_id ON vendor_applications(user_id);
CREATE INDEX IF NOT EXISTS idx_vendor_apps_status  ON vendor_applications(status);