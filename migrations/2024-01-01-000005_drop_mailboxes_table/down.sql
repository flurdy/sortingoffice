-- Migration: Recreate mailboxes table
-- Reverse operation for dropping mailboxes table

CREATE TABLE mailboxes (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    maildir VARCHAR(255) NOT NULL,
    quota BIGINT NOT NULL DEFAULT 0,
    domain VARCHAR(255) NOT NULL,
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    INDEX idx_username (username),
    INDEX idx_domain (domain),
    INDEX idx_active (active),
    FOREIGN KEY (domain) REFERENCES domains(domain) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci; 
