-- This file should undo anything in `up.sql`

-- Add domain column back to aliases table
-- This migration adds back the domain column that was removed

ALTER TABLE aliases ADD COLUMN domain VARCHAR(255) NOT NULL DEFAULT '';
ALTER TABLE aliases ADD CONSTRAINT aliases_ibfk_1 FOREIGN KEY (domain) REFERENCES domains(domain);
