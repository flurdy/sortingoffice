-- Seed data migration: Insert initial test data

-- Insert seed data for domains
INSERT INTO domains (domain, transport, backupmx, enabled) VALUES
('example.com', 'virtual', FALSE, 1),
('example.org', 'virtual', FALSE, 1);

-- Insert seed data for users
INSERT INTO users (id, crypt, name, maildir, domain, enabled) VALUES
('admin@example.com', 'admin123', 'Administrator', '/var/mail/example.com/admin', 'example.com', 1),
('user1@example.com', 'password123', 'Test User 1', '/var/mail/example.com/user1', 'example.com', 1),
('user2@example.org', 'password456', 'Test User 2', '/var/mail/example.org/user2', 'example.org', 1);

-- Insert seed data for aliases
INSERT INTO aliases (mail, destination, domain, enabled) VALUES
('admin@example.com', 'admin@example.com', 'example.com', 1),
('postmaster@example.com', 'admin@example.com', 'example.com', 1),
('abuse@example.com', 'admin@example.com', 'example.com', 1),
('@example.com', 'admin@example.com', 'example.com', 1),
('user1@example.com', 'user1@example.com', 'example.com', 1),
('user2@example.org', 'user2@example.org', 'example.org', 1),
('@example.org', 'user2@example.org', 'example.org', 1); 
