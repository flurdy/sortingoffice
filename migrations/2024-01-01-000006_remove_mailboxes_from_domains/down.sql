-- Migration: Add mailboxes column back to domains table
-- Reverse operation for removing mailboxes column

ALTER TABLE domains ADD COLUMN mailboxes INT NOT NULL DEFAULT 10; 
