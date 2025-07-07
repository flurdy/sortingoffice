# Database Management Guide

This guide explains how to manage databases for different environments in Sorting Office.

## Overview

Sorting Office supports multiple database environments:
- **Development**: Main database for development work
- **Test**: Separate database for running tests
- **Production**: Production database (when deployed)

## Quick Start

### 1. Setup Development Environment

```bash
# Set up development database with seed data
make prod-db-setup

# Or manually:
make migrate
make seed
```

### 2. Setup Test Environment

```bash
# Set up test database
make test-db-setup

# Run tests (automatically sets up test database)
make test-unit
```

### 3. Run the Application

```bash
# For development
cargo run

# For testing
cargo test
```

## Environment Configuration

### Using Environment Scripts

The easiest way to manage environments is using the provided script:

```bash
# Development environment
source scripts/set-env.sh dev
cargo run

# Test environment
source scripts/set-env.sh test
cargo test

# Docker environment
source scripts/set-env.sh docker
cargo run
```

### Manual Environment Setup

Create a `.env` file in the project root:

```bash
# Copy the example
cp env.local.example .env

# Edit the file with your database settings
nano .env
```

Example `.env` file:
```env
# Development database
DATABASE_URL=mysql://root:password@127.0.0.1:3306/sortingoffice

# Test database (separate from development)
TEST_DATABASE_URL=mysql://root:password@127.0.0.1:3306/sortingoffice_test

# Application settings
RUST_LOG=debug
HOST=127.0.0.1
PORT=3000
```

## Database Commands

### Getting Help

```bash
# Show all available database commands
make db-help
```

### Development Database

```bash
# Setup development database with seed data
make prod-db-setup

# Run migrations only
make migrate

# Seed database with initial data
make seed

# Reset development database (WARNING: destructive)
make prod-db-reset
```

### Test Database

```bash
# Setup test database
make test-db-setup

# Clean test database
make test-db-clean

# Reset test database
make test-db-reset
```

### General Database Operations

```bash
# Run pending migrations
make migrate

# Revert last migration
make migrate-revert

# Reset database (revert all, then run all)
make migrate-reset

# Create a new seed migration
make create-seed-migration
```

## Seed Data Management

### What is Seed Data?

Seed data is initial data that gets loaded into the database when you first set it up. This includes:
- Example domains
- Required email aliases
- Default users
- Configuration data

### Creating Seed Data

1. **Automatic creation**:
   ```bash
   make create-seed-data
   ```

2. **Manual creation**:
   ```bash
   # Create seed data files
   mkdir -p seed_data
   
   # Create individual seed files
   nano seed_data/domains.sql
   nano seed_data/users.sql
   nano seed_data/aliases.sql
   nano seed_data/backups.sql
   ```

3. **Example seed data**:
   ```sql
   -- seed_data/domains.sql
   INSERT INTO domains (domain, transport, enabled) VALUES 
       ('example.com', 'virtual', 1),
       ('test.com', 'smtp:localhost', 1);
   
   -- seed_data/users.sql
   INSERT INTO users (id, crypt, name, maildir, enabled) VALUES 
       ('admin@example.com', '$2b$12$...', 'Admin User', '/var/mail/example.com/admin', 1);
   
   -- seed_data/aliases.sql
   INSERT INTO aliases (mail, destination, enabled) VALUES 
       ('postmaster@example.com', 'admin@example.com', 1),
       ('abuse@example.com', 'admin@example.com', 1);
   ```

### Running Seed Data

```bash
# Run all seed data
make seed

# Run individual tables
make seed-domains
make seed-users
make seed-aliases
make seed-backups

# Manual loading
mysql -u root -p sortingoffice < seed_data/all.sql
```

## Test Database Isolation

### Why Separate Test Database?

Tests should never affect your development data. The test system:

1. **Uses a separate database**: `sortingoffice_test` vs `sortingoffice`
2. **Cleans up after each test**: Removes all test data
3. **Runs in isolation**: Each test starts with a clean state

### Test Database Configuration

Tests automatically use:
1. `TEST_DATABASE_URL` environment variable (if set)
2. `DATABASE_URL` environment variable (if `TEST_DATABASE_URL` not set)
3. Default test database: `mysql://root:password@localhost/sortingoffice_test`

### Running Tests Safely

```bash
# This automatically sets up the test database
make test-unit

# Or manually:
make test-db-setup
cargo test
```

## Troubleshooting

### Common Issues

#### 1. "Database doesn't exist" error

```bash
# Create the database
make test-db-setup  # for test database
make prod-db-setup  # for development database
```

#### 2. "Access denied" error

Check your MySQL credentials in the `.env` file:
```env
DATABASE_URL=mysql://username:password@host:port/database
```

#### 3. Tests affecting development data

Ensure you're using separate databases:
```env
DATABASE_URL=mysql://root:password@127.0.0.1:3306/sortingoffice
TEST_DATABASE_URL=mysql://root:password@127.0.0.1:3306/sortingoffice_test
```

#### 4. Seed data missing after tests

This is expected! Tests clean up after themselves. To restore seed data:
```bash
make seed
```

### Database Inspection

```bash
# List all domains
make list-domains

# Count all domains
make count-domains

# List all aliases
make list-aliases

# Count all aliases
make count-aliases

# List all users
make list-users

# Count all users
make count-users
```

### Manual Database Access

```bash
# Access MySQL shell
make db-shell

# Or directly:
mysql -u root -p sortingoffice
```

## Best Practices

1. **Always use separate databases** for development and testing
2. **Never run tests against production data**
3. **Use seed data** for consistent development environment
4. **Backup important data** before running destructive operations
5. **Use environment variables** for database configuration
6. **Run tests before committing** to ensure database changes work

## Migration Management

### Migration Consolidation

The project uses a consolidated migration approach where all schema changes are combined into a single migration file. This simplifies the migration history and makes it easier to set up new environments.

**To consolidate migrations**:
```bash
# Run the consolidation script
./scripts/consolidate-migrations.sh

# Or manually:
# 1. Keep only the consolidated migration
# 2. Remove old migration files
# 3. Run migrate-reset to apply the consolidated schema
```

### Creating Migrations

```bash
# Create a new migration
diesel migration generate migration_name

# Edit the generated files:
# - up.sql: Changes to apply
# - down.sql: Changes to revert
```

### Migration Consolidation Process

1. **Create consolidated migration**:
   ```bash
   diesel migration generate consolidated_schema_final
   ```

2. **Add all schema changes** to the `up.sql` file

3. **Add rollback logic** to the `down.sql` file

4. **Remove old migrations** (after testing)

5. **Update seed data** in the `seed_data/` directory

### Running Migrations

```bash
# Apply pending migrations
make migrate

# Revert last migration
make migrate-revert

# Reset all migrations
make migrate-reset
```

### Migration Best Practices

1. **Always include down.sql**: Make migrations reversible
2. **Test migrations**: Run `make migrate-reset` to test
3. **Backup before migrations**: In production environments
4. **Use descriptive names**: `add_user_table` not `migration_001` 
