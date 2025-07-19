-- Drop all tables in reverse order to avoid foreign key constraints

-- Drop clients table
DROP TABLE IF EXISTS clients;

-- Drop relocated table
DROP TABLE IF EXISTS relocated;

-- Drop relays table
DROP TABLE IF EXISTS relays;

-- Drop backups table
DROP TABLE IF EXISTS backups;

-- Drop aliases table
DROP TABLE IF EXISTS aliases;

-- Drop users table
DROP TABLE IF EXISTS users;

-- Drop domains table
DROP TABLE IF EXISTS domains;
