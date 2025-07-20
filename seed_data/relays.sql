-- Seed data for relays table
-- This file contains example relay recipients for testing

INSERT IGNORE INTO relays (recipient, status, enabled) VALUES
('relay1@example.com', 'OK', 1),
('relay2@example.org', 'OK', 1),
('relay3@test.com', 'REJECT', 0),
('backup-relay@example.com', 'OK', 1),
('external-relay@partner.com', 'REJECT', 1),
('blocked-relay@spam.com', 'REJECT', 1);
