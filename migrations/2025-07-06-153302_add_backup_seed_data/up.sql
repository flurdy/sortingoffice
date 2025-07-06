-- Your SQL goes here

-- Add seed data for backup MX servers

-- Insert seed data for backup MX servers
INSERT INTO backups (domain, transport, enabled) VALUES
('backup.example.com', 'smtp:[]', 1),
('mx2.example.org', 'smtp:relay.example.org', 1),
('fallback.example.net', 'smtp:backup.example.net', 0);
