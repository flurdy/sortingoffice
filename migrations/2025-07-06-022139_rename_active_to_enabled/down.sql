-- This file should undo anything in `up.sql`

-- Revert aliases table
ALTER TABLE aliases 
CHANGE COLUMN enabled active BOOLEAN NOT NULL DEFAULT TRUE;

-- Revert users table
ALTER TABLE users 
CHANGE COLUMN enabled active BOOLEAN NOT NULL DEFAULT TRUE;

-- Revert domains table
ALTER TABLE domains 
CHANGE COLUMN enabled active BOOLEAN NOT NULL DEFAULT TRUE;
