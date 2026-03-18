 -- ── Categories ───────────────────────────────────────────────────
INSERT INTO categories (name, slug, emoji, description) VALUES
  ('Fiction', 'fiction', '📖', 'Novels, short stories and imaginative writing'),
  ('Non-Fiction', 'non-fiction', '🧠', 'True stories, biographies and factual writing'),
  ('Self Help', 'self-help', '⭐', 'Personal development and growth'),
  ('Textbooks', 'textbooks', '📝', 'Academic and educational books'),
  ('Children', 'children', '🎨', 'Books for young readers'),
  ('Business', 'business', '💼', 'Entrepreneurship, finance and business strategy'),
  ('Science', 'science', '🔬', 'Science, technology and research'),
  ('History', 'history', '🏛️', 'World history and historical accounts')
ON CONFLICT (slug) DO NOTHING;

-- ── Locations ────────────────────────────────────────────────────
INSERT INTO locations (city, district, state, latitude, longitude) VALUES
  ('Lagos', 'Victoria Island', 'Lagos', 6.4281, 3.4219),
  ('Lagos', 'Ikeja', 'Lagos', 6.5954, 3.3383),
  ('Lagos', 'Lekki', 'Lagos', 6.4698, 3.5852),
  ('Lagos', 'Surulere', 'Lagos', 6.5020, 3.3531),
  ('Abuja', 'Wuse 2', 'FCT', 9.0765, 7.4892),
  ('Abuja', 'Garki', 'FCT', 9.0574, 7.4898),
  ('Abuja', 'Maitama', 'FCT', 9.0837, 7.4836),
  ('Ibadan', 'Bodija', 'Oyo', 7.4198, 3.9083),
  ('Ibadan', 'UI', 'Oyo', 7.4474, 3.8966),
  ('Port Harcourt', 'GRA', 'Rivers', 4.8156, 7.0498),
  ('Port Harcourt', 'Trans Amadi', 'Rivers', 4.8242, 7.0137)
ON CONFLICT DO NOTHING;

-- ── BOOKSTORES ───────────────────────────────────────────────────
-- Lagos Book stores -------
INSERT INTO bookstores (name, address, location_id, rating, total_reviews, delivery_time_minutes, delivery_fee, minimum_order, image_emoji)
SELECT
  'Quintessence Books',
  '23 Ademola Adetokunbo Street, Victoria Island',
  l.id, 4.8, 312, 25, 500.00, 1500.00, '🏛️'
FROM locations l WHERE l.district = 'Victoria Island' AND l.city = 'Lagos';

INSERT INTO bookstores (name, address, location_id, rating, total_reviews, delivery_time_minutes, delivery_fee, minimum_order, image_emoji)
SELECT
  'Terra Kulture Books',
  '1376 Tiamiyu Savage Street, Victoria Island',
  l.id, 4.7, 289, 30, 500.00, 2000.00, '🎭'
FROM locations l WHERE l.district = 'Victoria Island' AND l.city = 'Lagos';

INSERT INTO bookstores (name, address, location_id, rating, total_reviews, delivery_time_minutes, delivery_fee, minimum_order, image_emoji)
SELECT
  'Laterna Ventures',
  '13 Oko-Awo Close, Victoria Island',
  l.id, 4.6, 198, 35, 600.00, 1500.00, '📚'
FROM locations l WHERE l.district = 'Victoria Island' AND l.city = 'Lagos';

INSERT INTO bookstores (name, address, location_id, rating, total_reviews, delivery_time_minutes, delivery_fee, minimum_order, image_emoji)
SELECT
  'Ikeja Book Hub',
  '14 Allen Avenue, Ikeja',
  l.id, 4.5, 156, 30, 400.00, 1000.00, '📖'
FROM locations l WHERE l.district = 'Ikeja' AND l.city = 'Lagos';

INSERT INTO bookstores (name, address, location_id, rating, total_reviews, delivery_time_minutes, delivery_fee, minimum_order, image_emoji)
SELECT
  'Lekki Reads',
  '5 Admiralty Way, Lekki Phase 1',
  l.id, 4.9, 421, 20, 700.00, 2000.00, '🌟'

FROM locations l WHERE l.district = 'Lekki' AND l.city = 'Lagos';

-- Abuja bookstores ---
INSERT INTO bookstores (name, address, location_id, rating, total_reviews, delivery_time_minutes, delivery_fee, minimum_order, image_emoji)
SELECT
  'Wuse Book Corner',
  '22 Aminu Kano Crescent, Wuse 2',
  l.id, 4.6, 203, 25, 400.00, 1500.00, '🏙️'
FROM locations l WHERE l.district = 'Wuse 2' AND l.city = 'Abuja';

INSERT INTO bookstores (name, address, location_id, rating, total_reviews, delivery_time_minutes, delivery_fee, minimum_order, image_emoji)
SELECT
  'Maitama Library Store',
  '8 Diplomatic Drive, Maitama',
  l.id, 4.8, 176, 30, 500.00, 2000.00, '🎓'
FROM locations l WHERE l.district = 'Maitama' AND l.city = 'Abuja';
--Ibadan and PH book stores -
INSERT INTO bookstores (name, address, location_id, rating, total_reviews, delivery_time_minutes, delivery_fee, minimum_order, image_emoji)
SELECT
  'Bodija Books',
  '12 Bodija Market Road, Ibadan',
  l.id, 4.4, 134, 35, 300.00, 1000.00, '📗'
FROM locations l WHERE l.district = 'Bodija' AND l.city = 'Ibadan';

INSERT INTO bookstores (name, address, location_id, rating, total_reviews, delivery_time_minutes, delivery_fee, minimum_order, image_emoji)
SELECT
  'UI Campus Reads',
  'University of Ibadan Main Gate Road',
  l.id, 4.7, 267, 20, 200.00, 500.00, '🎓'
FROM locations l WHERE l.district = 'UI' AND l.city = 'Ibadan';

INSERT INTO bookstores (name, address, location_id, rating, total_reviews, delivery_time_minutes, delivery_fee, minimum_order, image_emoji)
SELECT
  'GRA Book Palace',
  '15 Peter Odili Road, GRA Phase 2',
  l.id, 4.5, 189, 30, 400.00, 1500.00, '👑'
FROM locations l WHERE l.district = 'GRA' AND l.city = 'Port Harcourt';

-- ── Books ────────────────────────────────────────────────────────

-- Fiction books
INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Things Fall Apart', 'Chinua Achebe', 2500.00,
  c.id, b.id,
  'A classic story of pre-colonial Igbo society and the arrival of Europeans.',
  '📘', '#1A3A5C', 4.9
FROM categories c, bookstores b
WHERE c.slug = 'fiction' AND b.name = 'Quintessence Books';

INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Purple Hibiscus', 'Chimamanda Ngozi Adichie', 3200.00,
  c.id, b.id,
  'A coming-of-age story set in post-colonial Nigeria.',
  '📗', '#1A3D2B', 4.8
FROM categories c, bookstores b
WHERE c.slug = 'fiction' AND b.name = 'Terra Kulture Books';

INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Half of a Yellow Sun', 'Chimamanda Ngozi Adichie', 3800.00,
  c.id, b.id,
  'A powerful novel set during the Nigerian-Biafran War.',
  '📙', '#3D2B1A', 4.9
FROM categories c, bookstores b
WHERE c.slug = 'fiction' AND b.name = 'Lekki Reads';

INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'The Alchemist', 'Paulo Coelho', 2900.00,
  c.id, b.id,
  'A magical story about following your dreams.',
  '📒', '#3D3A1A', 4.7
FROM categories c, bookstores b
WHERE c.slug = 'fiction' AND b.name = 'Wuse Book Corner';

INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Americanah', 'Chimamanda Ngozi Adichie', 4200.00,
  c.id, b.id,
  'A story of race, identity and belonging across two continents.',
  '📘', '#2B1A3D', 4.8
FROM categories c, bookstores b
WHERE c.slug = 'fiction' AND b.name = 'Laterna Ventures';

-- Self Help books
INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Atomic Habits', 'James Clear', 4500.00,
  c.id, b.id,
  'An easy and proven way to build good habits and break bad ones.',
  '⭐', '#1A2A3D', 4.9
FROM categories c, bookstores b
WHERE c.slug = 'self-help' AND b.name = 'Lekki Reads';

INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Deep Work', 'Cal Newport', 3800.00,
  c.id, b.id,
  'Rules for focused success in a distracted world.',
  '🧠', '#1A3D3A', 4.8
FROM categories c, bookstores b
WHERE c.slug = 'self-help' AND b.name = 'Maitama Library Store';

INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Think and Grow Rich', 'Napoleon Hill', 2800.00,
  c.id, b.id,
  'The landmark bestseller on achieving success and wealth.',
  '💡', '#3D2B1A', 4.7
FROM categories c, bookstores b
WHERE c.slug = 'self-help' AND b.name = 'Bodija Books';

-- Business books
INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Zero to One', 'Peter Thiel', 4800.00,
  c.id, b.id,
  'Notes on startups and how to build the future.',
  '🚀', '#1A1A3D', 4.8
FROM categories c, bookstores b
WHERE c.slug = 'business' AND b.name = 'Quintessence Books';

INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Rich Dad Poor Dad', 'Robert Kiyosaki', 3200.00,
  c.id, b.id,
  'What the rich teach their kids about money.',
  '💰', '#2B3D1A', 4.7
FROM categories c, bookstores b
WHERE c.slug = 'business' AND b.name = 'Ikeja Book Hub';

-- Non-Fiction books
INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Educated', 'Tara Westover', 4200.00,
  c.id, b.id,
  'A memoir about the transformative power of education.',
  '📚', '#3D1A2B', 4.9
FROM categories c, bookstores b
WHERE c.slug = 'non-fiction' AND b.name = 'UI Campus Reads';

INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Sapiens', 'Yuval Noah Harari', 5500.00,
  c.id, b.id,
  'A brief history of humankind.',
  '🌍', '#1A3D2B', 4.8
FROM categories c, bookstores b
WHERE c.slug = 'non-fiction' AND b.name = 'GRA Book Palace';

INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Born a Crime', 'Trevor Noah', 3900.00,
  c.id, b.id,
  'Stories from a South African childhood.',
  '🎭', '#2B1A3D', 4.9
FROM categories c, bookstores b
WHERE c.slug = 'non-fiction' AND b.name = 'Terra Kulture Books';

-- Textbooks
INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Engineering Mathematics', 'K.A. Stroud', 12000.00,
  c.id, b.id,
  'The most comprehensive maths textbook for engineers.',
  '📐', '#1A2B3D', 4.6
FROM categories c, bookstores b
WHERE c.slug = 'textbooks' AND b.name = 'UI Campus Reads';

INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Introduction to Algorithms', 'CLRS', 15000.00,
  c.id, b.id,
  'The definitive computer science algorithms textbook.',
  '💻', '#1A3A2B', 4.7
FROM categories c, bookstores b
WHERE c.slug = 'textbooks' AND b.name = 'Bodija Books';