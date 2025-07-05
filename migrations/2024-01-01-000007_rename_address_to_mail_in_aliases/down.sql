-- Migration: Rename mail column back to address in aliases table
-- Reverse operation for renaming address to mail

ALTER TABLE aliases DROP INDEX idx_mail;
ALTER TABLE aliases CHANGE COLUMN mail address VARCHAR(255) NOT NULL;
ALTER TABLE aliases ADD INDEX idx_address (address); 
