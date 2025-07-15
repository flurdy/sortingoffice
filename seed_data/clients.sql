-- Seed data for clients table
-- This file contains example client configurations for testing

INSERT IGNORE INTO clients (client, status) VALUES
('192.168.1.100', 'allowed'),
('192.168.1.101', 'allowed'),
('10.0.0.50', 'allowed'),
('172.16.0.25', 'rejected'),
('203.0.113.10', 'allowed'),
('198.51.100.5', 'rejected');
