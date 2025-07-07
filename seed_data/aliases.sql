-- Seed data for aliases table
-- This file contains initial alias data for development/testing

-- Insert seed data for aliases
INSERT INTO aliases (mail, destination, enabled) VALUES
('admin@example.com', 'admin@example.com', 1),
('postmaster@example.com', 'admin@example.com', 1),
('abuse@example.com', 'admin@example.com', 1),
('webmaster@example.com', 'admin@example.com', 1),
('@example.com', 'admin@example.com', 1),
('user1@example.com', 'user1@example.com', 1),
('user2@example.org', 'user2@example.org', 1),
('@example.org', 'user2@example.org', 1),
('postmaster@example.org', 'user2@example.org', 1); 
