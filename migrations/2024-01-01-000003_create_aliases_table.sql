-- Migration: Create aliases table
-- Based on flurdy's Postfix mail server schema

CREATE TABLE aliases (
    id INT AUTO_INCREMENT PRIMARY KEY,
    address VARCHAR(255) NOT NULL UNIQUE,
    goto VARCHAR(255) NOT NULL,
    domain VARCHAR(255) NOT NULL,
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    INDEX idx_address (address),
    INDEX idx_domain (domain),
    INDEX idx_active (active),
    FOREIGN KEY (domain) REFERENCES domains(domain) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci; 
