-- Your SQL goes here

-- Add backups table and update domains table

-- First, update the domains table to set transport default to "virtual:"
ALTER TABLE domains MODIFY COLUMN transport VARCHAR(255) DEFAULT 'virtual:';

-- Create backups table (duplicate of domains but without backupmx field)
CREATE TABLE backups (
    pkid INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    domain VARCHAR(255) NOT NULL UNIQUE,
    transport VARCHAR(255) DEFAULT ':[]',
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    enabled TINYINT(1) NOT NULL DEFAULT 1
);
