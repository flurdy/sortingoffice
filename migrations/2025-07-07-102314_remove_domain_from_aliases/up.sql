-- Your SQL goes here

-- Remove domain column from aliases table
-- The domain can be derived from the mail field (everything after @)

-- First drop the foreign key constraint
ALTER TABLE aliases DROP FOREIGN KEY aliases_ibfk_1;

-- Then drop the domain column
ALTER TABLE aliases DROP COLUMN domain;
