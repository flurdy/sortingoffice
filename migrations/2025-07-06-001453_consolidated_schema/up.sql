-- Consolidated migration: Create all tables with pkid primary keys and seed data

-- Create domains table
CREATE TABLE domains (
    pkid INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    domain VARCHAR(255) NOT NULL UNIQUE,
    description VARCHAR(255),
    aliases INT NOT NULL DEFAULT 10,
    maxquota BIGINT NOT NULL DEFAULT 0,
    quota BIGINT NOT NULL DEFAULT 0,
    transport VARCHAR(255),
    backupmx BOOLEAN NOT NULL DEFAULT FALSE,
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    active BOOLEAN NOT NULL DEFAULT TRUE
);

-- Create users table
CREATE TABLE users (
    pkid INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    maildir VARCHAR(255) NOT NULL,
    quota BIGINT NOT NULL DEFAULT 0,
    domain VARCHAR(255) NOT NULL,
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    FOREIGN KEY (domain) REFERENCES domains(domain)
);

-- Create aliases table
CREATE TABLE aliases (
    pkid INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    mail VARCHAR(255) NOT NULL,
    destination VARCHAR(255) NOT NULL,
    domain VARCHAR(255) NOT NULL,
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    FOREIGN KEY (domain) REFERENCES domains(domain)
);

-- Insert seed data for domains
INSERT INTO domains (domain, description, aliases, maxquota, quota, transport, backupmx, active) VALUES
('example.com', 'Example domain for testing', 10, 0, 0, 'virtual', FALSE, TRUE),
('example.org', 'Another example domain', 5, 0, 0, 'virtual', FALSE, TRUE);

-- Insert seed data for users
INSERT INTO users (username, password, name, maildir, quota, domain, active) VALUES
('admin', 'admin123', 'Administrator', '/var/mail/example.com/admin', 1073741824, 'example.com', TRUE),
('user1', 'password123', 'Test User 1', '/var/mail/example.com/user1', 1073741824, 'example.com', TRUE),
('user2', 'password456', 'Test User 2', '/var/mail/example.org/user2', 1073741824, 'example.org', TRUE);

-- Insert seed data for aliases
INSERT INTO aliases (mail, destination, domain, active) VALUES
('admin@example.com', 'admin@example.com', 'example.com', TRUE),
('postmaster@example.com', 'admin@example.com', 'example.com', TRUE),
('abuse@example.com', 'admin@example.com', 'example.com', TRUE),
('@example.com', 'admin@example.com', 'example.com', TRUE),
('user1@example.com', 'user1@example.com', 'example.com', TRUE),
('user2@example.org', 'user2@example.org', 'example.org', TRUE),
('@example.org', 'user2@example.org', 'example.org', TRUE);
