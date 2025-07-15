-- Seed data for relays table
-- This file contains example relay recipients for testing

INSERT IGNORE INTO relays (recipient, status, enabled) VALUES
('relay1@example.com', 'allowed', 1),
('relay2@example.org', 'allowed', 1),
('relay3@test.com', 'rejected', 0),
('backup-relay@example.com', 'allowed', 1),
('external-relay@partner.com', 'allowed', 1),
('blocked-relay@spam.com', 'rejected', 1);
