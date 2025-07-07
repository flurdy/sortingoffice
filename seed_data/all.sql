-- Master seed data file for Sorting Office
-- This file contains all seed data in the correct order (respecting foreign key constraints)
-- Run this file after running migrations to populate the database with initial data

-- Seed data for domains (must be first due to foreign key constraints)
INSERT INTO domains (domain, transport, enabled) VALUES
('example.com', 'virtual', 1),
('example.org', 'virtual', 1),
('test.com', 'smtp:localhost', 1);

-- Seed data for users (depends on domains)
INSERT INTO users (id, crypt, name, maildir, domain, enabled) VALUES
('admin@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj4J/HS.iQeO', 'Administrator', '/var/mail/example.com/admin', 'example.com', 1),
('user1@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj4J/HS.iQeO', 'Test User 1', '/var/mail/example.com/user1', 'example.com', 1),
('user2@example.org', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj4J/HS.iQeO', 'Test User 2', '/var/mail/example.org/user2', 'example.org', 1);

-- Seed data for aliases (no foreign key constraints)
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

-- Seed data for backups (no foreign key constraints)
INSERT INTO backups (domain, transport, enabled) VALUES
('backup.example.com', 'smtp:[]', 1),
('mx2.example.org', 'smtp:relay.example.org', 1),
('fallback.example.net', 'smtp:backup.example.net', 0);

-- Note: The crypt field contains bcrypt hashes for 'password123' 
