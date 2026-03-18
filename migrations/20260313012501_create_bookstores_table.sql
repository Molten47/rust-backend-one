CREATE TABLE IF NOT EXISTS bookstores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    address TEXT NOT NULL,
    location_id UUID NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
    rating DECIMAL(3, 2) DEFAULT 0.00,
    total_reviews INTEGER DEFAULT 0,
    delivery_time_minutes INTEGER DEFAULT 30,
    delivery_fee DECIMAL(10, 2) DEFAULT 0.00,
    minimum_order DECIMAL(10, 2) DEFAULT 0.00,
    is_open BOOLEAN DEFAULT true,
    image_emoji VARCHAR(10) DEFAULT '📚',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_bookstores_location ON bookstores(location_id);
CREATE INDEX idx_bookstores_rating ON bookstores(rating);