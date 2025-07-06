-- This file should undo anything in `up.sql`

-- Drop the backups table
DROP TABLE IF EXISTS backups;

-- Revert the domains table transport default
ALTER TABLE domains MODIFY COLUMN transport VARCHAR(255);
