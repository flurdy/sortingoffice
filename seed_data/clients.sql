-- Seed data for clients table
-- This file contains example client configurations for testing

INSERT IGNORE INTO clients (client, status) VALUES
('192.168.1.100', 'OK'),
('192.168.1.101', 'OK'),
('10.0.0.50', 'OK'),
('172.16.0.25', 'REJECT'),
('203.0.113.10', 'OK'),
('198.51.100.5', 'REJECT');
