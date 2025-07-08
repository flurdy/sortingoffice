-- This file should undo anything in `up.sql`

-- Remove unique constraint from aliases.mail
ALTER TABLE aliases DROP INDEX mail;
