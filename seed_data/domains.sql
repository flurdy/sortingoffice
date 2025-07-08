-- Seed data for domains table
-- This file contains initial domain data for development/testing

-- Insert seed data for domains
INSERT INTO domains (domain, transport, enabled) VALUES
('example.com', 'virtual:', 1),
('example.org', 'virtual:', 1),
('test.com', 'smtp:localhost', 1); 
