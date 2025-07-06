-- Your SQL goes here

-- Rename active fields to enabled and change type to tinyint

-- Update domains table
ALTER TABLE domains 
CHANGE COLUMN active enabled TINYINT(1) NOT NULL DEFAULT 1;

-- Update users table  
ALTER TABLE users 
CHANGE COLUMN active enabled TINYINT(1) NOT NULL DEFAULT 1;

-- Update aliases table
ALTER TABLE aliases 
CHANGE COLUMN active enabled TINYINT(1) NOT NULL DEFAULT 1;
