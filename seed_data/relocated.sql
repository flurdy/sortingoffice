-- Seed data for relocated table
-- This file contains example email relocations for testing

INSERT IGNORE INTO relocated (old_address, new_address, enabled) VALUES
('olduser@example.com', 'newuser@example.org', 1),
('former.employee@example.com', 'hr@example.com', 1),
('support@oldcompany.com', 'help@newcompany.com', 1),
('info@legacy-domain.com', 'contact@current-domain.com', 0),
('admin@deprecated.com', 'administrator@active.com', 1);
