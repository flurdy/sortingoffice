-- Your SQL goes here

-- Create relocated table for Postfix relocation configuration
-- This table stores information about users who have moved to new email addresses
CREATE TABLE relocated (
    pkid INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    old_address VARCHAR(255) NOT NULL,
    new_address VARCHAR(255) NOT NULL,
    enabled TINYINT(1) NOT NULL DEFAULT 1,
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

-- Add unique constraint on old_address to prevent duplicates
ALTER TABLE relocated ADD UNIQUE KEY unique_old_address (old_address);
