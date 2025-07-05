-- Migration: Drop mailboxes table
-- The mailboxes table is redundant with the users table

DROP TABLE IF EXISTS mailboxes; 
