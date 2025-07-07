-- This file should undo anything in `up.sql`

-- Drop tables in reverse order (due to foreign key constraints)
DROP TABLE IF EXISTS backups;
DROP TABLE IF EXISTS aliases;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS domains;
