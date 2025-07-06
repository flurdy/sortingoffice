-- This file should undo anything in `up.sql`

-- Remove seed data in reverse order (due to foreign key constraints)
DELETE FROM aliases WHERE domain IN ('example.com', 'example.org');
DELETE FROM users WHERE domain IN ('example.com', 'example.org');
DELETE FROM domains WHERE domain IN ('example.com', 'example.org'); 
