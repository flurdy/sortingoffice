-- Rename goto column to destination in aliases table
ALTER TABLE aliases CHANGE COLUMN goto destination VARCHAR(255) NOT NULL; 
