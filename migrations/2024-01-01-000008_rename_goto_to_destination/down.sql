-- Rename destination column back to goto in aliases table
-- Reverse operation for renaming goto to destination
ALTER TABLE aliases CHANGE COLUMN destination goto VARCHAR(255) NOT NULL; 
