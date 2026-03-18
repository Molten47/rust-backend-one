-- Update prices on existing books to 10k-18k range
UPDATE books SET price = 12000.00 WHERE title = 'Things Fall Apart';
UPDATE books SET price = 13500.00 WHERE title = 'Purple Hibiscus';
UPDATE books SET price = 14000.00 WHERE title = 'Half of a Yellow Sun';
UPDATE books SET price = 11000.00 WHERE title = 'The Alchemist';
UPDATE books SET price = 15000.00 WHERE title = 'Americanah';
UPDATE books SET price = 16500.00 WHERE title = 'Atomic Habits';
UPDATE books SET price = 14500.00 WHERE title = 'Deep Work';
UPDATE books SET price = 11500.00 WHERE title = 'Think and Grow Rich';
UPDATE books SET price = 17000.00 WHERE title = 'Zero to One';
UPDATE books SET price = 12500.00 WHERE title = 'Rich Dad Poor Dad';
UPDATE books SET price = 15500.00 WHERE title = 'Educated';
UPDATE books SET price = 18000.00 WHERE title = 'Sapiens';
UPDATE books SET price = 13000.00 WHERE title = 'Born a Crime';
UPDATE books SET price = 16000.00 WHERE title = 'Engineering Mathematics';
UPDATE books SET price = 17500.00 WHERE title = 'Introduction to Algorithms';

-- Add new Nigerian books
INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Only Big Bumbum Matter Tomorrow', 'Damilare Kuku', 14500.00,
  c.id, b.id,
  'A sharp, darkly comic novel about love, ambition and survival in contemporary Lagos.',
  '📘', '#2B1A3D', 4.9
FROM categories c, bookstores b
WHERE c.slug = 'fiction' AND b.name = 'Lekki Reads';

INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Ponmo Is a Bird and other short stories', 'Abigail Serwaa and more..', 13000.00,
  c.id, b.id,
  'A lyrical debut exploring identity, belonging and what it means to return home.',
  '📗', '#1A3D2B', 4.8
FROM categories c, bookstores b
WHERE c.slug = 'fiction' AND b.name = 'Terra Kulture Books';

INSERT INTO books (title, author, price, category_id, bookstore_id, description, cover_emoji, cover_color, rating)
SELECT
  'Dream Count', 'Chimamanda Ngozi Adichie', 16000.00,
  c.id, b.id,
  'Chimamanda''s long-awaited new novel — a meditation on love, loss and longing.',
  '📙', '#3D1A2B', 5.0
FROM categories c, bookstores b
WHERE c.slug = 'fiction' AND b.name = 'Quintessence Books';