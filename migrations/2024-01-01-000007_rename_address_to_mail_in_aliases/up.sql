-- Migration: Rename address column to mail in aliases table

ALTER TABLE aliases DROP INDEX idx_address;
ALTER TABLE aliases CHANGE COLUMN address mail VARCHAR(255) NOT NULL;
ALTER TABLE aliases ADD INDEX idx_mail (mail); 
