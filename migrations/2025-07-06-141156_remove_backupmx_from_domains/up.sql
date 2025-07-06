-- Your SQL goes here

-- Remove backupmx field from domains table
ALTER TABLE domains DROP COLUMN backupmx;
