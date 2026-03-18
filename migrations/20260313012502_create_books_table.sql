CREATE TABLE IF NOT EXISTS books (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(300) NOT NULL,
    author VARCHAR(200) NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    bookstore_id UUID NOT NULL REFERENCES bookstores(id) ON DELETE CASCADE,
    description TEXT,
    cover_emoji VARCHAR(10) DEFAULT '📘',
    cover_color VARCHAR(20) DEFAULT '#1A1410',
    in_stock BOOLEAN DEFAULT true,
    rating DECIMAL(3, 2) DEFAULT 0.00,
    total_reviews INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_books_category ON books(category_id);
CREATE INDEX idx_books_bookstore ON books(bookstore_id);
CREATE INDEX idx_books_price ON books(price);
CREATE INDEX idx_books_rating ON books(rating);