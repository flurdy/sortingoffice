-- Fix NULL timestamp columns to allow NULL values
-- This migration updates the schema to match databases that were manually updated
-- with NULL defaults for MySQL 5.5 compatibility

-- Update domains table
ALTER TABLE domains 
MODIFY COLUMN created DATETIME NULL,
MODIFY COLUMN modified DATETIME NULL;

-- Update users table  
ALTER TABLE users
MODIFY COLUMN created DATETIME NULL,
MODIFY COLUMN modified DATETIME NULL;

-- Update aliases table
ALTER TABLE aliases
MODIFY COLUMN created DATETIME NULL,
MODIFY COLUMN modified DATETIME NULL;

-- Update backups table
ALTER TABLE backups
MODIFY COLUMN created DATETIME NULL,
MODIFY COLUMN modified DATETIME NULL;

-- Update relays table
ALTER TABLE relays
MODIFY COLUMN created DATETIME NULL,
MODIFY COLUMN modified DATETIME NULL;

-- Update relocated table
ALTER TABLE relocated
MODIFY COLUMN created DATETIME NULL,
MODIFY COLUMN modified DATETIME NULL;
