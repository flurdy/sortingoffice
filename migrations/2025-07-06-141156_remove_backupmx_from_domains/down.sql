-- This file should undo anything in `up.sql`

-- Add backupmx field back to domains table
ALTER TABLE domains ADD COLUMN backupmx TINYINT(1) NOT NULL DEFAULT 0;
