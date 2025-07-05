-- Migration: Remove mailboxes column from domains table
-- The mailboxes column is no longer needed since we removed the mailboxes table

ALTER TABLE domains DROP COLUMN mailboxes; 
