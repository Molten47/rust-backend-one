INSERT INTO bookstores (name, address, location_id, rating, total_reviews, delivery_time_minutes, delivery_fee, minimum_order, image_emoji)
SELECT
  'Ouida Lagos',
  '34 Ajanaku Street, Off Salvation Road, Ikeja',
  l.id, 4.9, 387, 25, 600.00, 2000.00, '📖'
FROM locations l WHERE l.district = 'Ikeja' AND l.city = 'Lagos';