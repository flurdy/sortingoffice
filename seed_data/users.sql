-- Seed data for users table
-- This file contains initial user data for development/testing

-- Insert seed data for users
INSERT INTO users (id, crypt, name, maildir, domain, enabled) VALUES
('admin@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj4J/HS.iQeO', 'Administrator', '/var/mail/example.com/admin', 'example.com', 1),
('user1@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj4J/HS.iQeO', 'Test User 1', '/var/mail/example.com/user1', 'example.com', 1),
('user2@example.org', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj4J/HS.iQeO', 'Test User 2', '/var/mail/example.org/user2', 'example.org', 1);

-- Note: The crypt field contains bcrypt hashes for 'password123' 
