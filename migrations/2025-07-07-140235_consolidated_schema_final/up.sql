-- Your SQL goes here

-- Consolidated migration: Create all tables with final schema

-- Create domains table
CREATE TABLE domains (
    pkid INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    domain VARCHAR(255) NOT NULL UNIQUE,
    transport VARCHAR(255) DEFAULT 'virtual:',
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    enabled TINYINT(1) NOT NULL DEFAULT 1
);

-- Create users table
CREATE TABLE users (
    pkid INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    id VARCHAR(255) NOT NULL,
    crypt VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    maildir VARCHAR(255) NOT NULL,
    home VARCHAR(255) NOT NULL DEFAULT '/var/spool/mail/virtual',
    uid SMALLINT(5) UNSIGNED NOT NULL DEFAULT 5000,
    gid SMALLINT(5) UNSIGNED NOT NULL DEFAULT 5000,
    domain VARCHAR(255) NOT NULL,
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    enabled TINYINT(1) NOT NULL DEFAULT 1,
    change_password TINYINT(1) NOT NULL DEFAULT 0,
    FOREIGN KEY (domain) REFERENCES domains(domain)
);

-- Create aliases table (without domain column - domain is derived from mail field)
CREATE TABLE aliases (
    pkid INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    mail VARCHAR(255) NOT NULL,
    destination VARCHAR(255) NOT NULL,
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    enabled TINYINT(1) NOT NULL DEFAULT 1
);

-- Create backups table
CREATE TABLE backups (
    pkid INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    domain VARCHAR(255) NOT NULL UNIQUE,
    transport VARCHAR(255) DEFAULT ':[]',
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    enabled TINYINT(1) NOT NULL DEFAULT 1
);
