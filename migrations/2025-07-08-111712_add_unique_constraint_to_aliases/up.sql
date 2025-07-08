-- Your SQL goes here

-- Add unique constraint to aliases.mail to prevent duplicate aliases
ALTER TABLE aliases ADD UNIQUE (mail);
