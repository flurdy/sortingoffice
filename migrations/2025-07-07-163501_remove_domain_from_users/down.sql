-- This file should undo anything in `up.sql`

-- Add domain column back to users table
ALTER TABLE users ADD COLUMN domain VARCHAR(255) NOT NULL DEFAULT '';
ALTER TABLE users ADD CONSTRAINT users_ibfk_1 FOREIGN KEY (domain) REFERENCES domains(domain);
