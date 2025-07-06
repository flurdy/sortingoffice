-- This file should undo anything in `up.sql`

-- Remove seed data for backup MX servers
DELETE FROM backups WHERE domain IN ('backup.example.com', 'mx2.example.org', 'fallback.example.net');
