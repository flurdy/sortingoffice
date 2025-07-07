-- Your SQL goes here

-- Remove domain column from users table
ALTER TABLE users DROP FOREIGN KEY users_ibfk_1;
ALTER TABLE users DROP COLUMN domain;
