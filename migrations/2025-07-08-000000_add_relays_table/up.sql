-- Your SQL goes here

-- Create relays table for Postfix relay configuration
-- This table stores information about relay recipients and their status
CREATE TABLE relays (
    pkid INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    recipient VARCHAR(255) NOT NULL,
    status VARCHAR(10) NOT NULL,
    enabled TINYINT(1) NOT NULL DEFAULT 1,
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

-- Add unique constraint on recipient to prevent duplicates
ALTER TABLE relays ADD UNIQUE KEY unique_recipient (recipient); 
