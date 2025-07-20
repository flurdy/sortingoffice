-- Seed data for users table
-- This file contains initial user data for development/testing

-- Insert seed data for users
INSERT INTO users (id, crypt, name, maildir, enabled) VALUES
('admin@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj4J/HS.iQeO', 'admin', 'example.com/admin', 1),
('user1@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj4J/HS.iQeO', 'testuser1', 'example.com/user1', 1),
('user2@example.org', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj4J/HS.iQeO', 'testuser2', 'example.org/user2', 1);

-- Note: The crypt field contains bcrypt hashes for 'password123' 
