# Scripts

This directory contains utility scripts for managing and testing the Sorting Office application.

## Available Scripts

### `curl-helpers.sh` - API Testing and Automation

A comprehensive curl-based utility for testing and interacting with the Sorting Office API.

**Features:**
- **Authentication**: Login, logout, and session management
- **Backup Operations**: Create, list, download, and delete database backups
- **Resource Management**: List aliases, domains, users
- **System Operations**: Health checks, statistics, configuration access
- **Session Persistence**: Maintains login sessions across commands

**Usage Examples:**
```bash
# Login with default credentials
./scripts/curl-helpers.sh login

# Login with custom credentials
./scripts/curl-helpers.sh login admin mypassword

# List database backups
./scripts/curl-helpers.sh backup-list

# Create a backup
./scripts/curl-helpers.sh backup-create primary

# Download a backup file
./scripts/curl-helpers.sh backup-download primary_sortingoffice_20250726_182756.sql

# List aliases
./scripts/curl-helpers.sh aliases-list

# Check application health
./scripts/curl-helpers.sh health
```

**Environment Variables:**
- `SORTINGOFFICE_HOST` - Default host (default: localhost:3000)
- `SORTINGOFFICE_USER` - Default username (default: admin)
- `SORTINGOFFICE_PASSWORD` - Default password (default: admin123)

### `generate_password_hash.sh` - Password Hash Generation

Generate bcrypt password hashes for admin user configuration.

**Usage:**
```bash
./scripts/generate_password_hash.sh "your_password"
```

**Features:**
- Supports Python 3 and Node.js
- Generates bcrypt hashes with cost factor 12
- Provides fallback instructions if dependencies missing

### `list-databases.sh` - Database Configuration Listing

List all configured databases from the config.toml file.

**Usage:**
```bash
./scripts/list-databases.sh
```

**Output:**
```
ID      Label   URL
primary Main Server      mysql://sortingoffice:***@localhost:3306/sortingoffice
backup1 Backup Server    mysql://backupuser:***@backuphost:3306/backupdb
```

### `set-env.sh` - Environment Setup

Set up environment variables for development and testing.

**Usage:**
```bash
source ./scripts/set-env.sh
```

**Features:**
- Configures database connections
- Sets up test environment variables
- Manages Docker Compose configurations

### `refactor.sh` - Code Refactoring Tool

A simple wrapper for common refactoring tasks in the Rust codebase.

**Usage:**
```bash
# Move functions between modules
./scripts/refactor.sh move utils database_helpers get_current_db_pool

# Rename a function
./scripts/refactor.sh rename function old_name new_name

# Fix compilation errors
./scripts/refactor.sh fix

# Clean up unused imports
./scripts/refactor.sh cleanup

# Get help
./scripts/refactor.sh help
```

**Features:**
- Move functions between modules
- Rename functions and modules
- Fix common compilation errors
- Clean up unused imports
- Safe function reference handling

### `refactor_generic.sh` - Generic Refactoring Engine

The underlying engine that handles all refactoring operations.

**Operations:**
- `move-functions` - Move functions between modules
- `move-module` - Move entire modules
- `rename-function` - Rename functions
- `rename-module` - Rename modules
- `update-imports` - Update import paths
- `fix-compilation` - Fix common compilation errors
- `cleanup-imports` - Remove unused imports

**Usage:**
```bash
./scripts/refactor_generic.sh move-functions utils database_helpers get_current_db_pool
./scripts/refactor_generic.sh rename-function old_name new_name
./scripts/refactor_generic.sh fix-compilation
```

### `health-check.sh` - Comprehensive Health Check

A detailed health check script with verbose output and configuration options.

**Usage:**
```bash
# Basic health check
./scripts/health-check.sh

# Verbose output with timing
./scripts/health-check.sh -v

# Custom host and timeout
./scripts/health-check.sh -h 192.168.1.100:3000 -t 5
```

**Features:**
- Colored output with status indicators
- Configurable timeout and host
- Verbose mode for debugging
- Environment variable support
- Detailed timing information

### `health-check-simple.sh` - Simple Health Check

A minimal one-liner health check for quick status checks.

**Usage:**
```bash
# Check localhost:3000
./scripts/health-check-simple.sh

# Check remote host
./scripts/health-check-simple.sh 192.168.1.100:3000
```

**Features:**
- Minimal output with emoji indicators
- Fast execution (5s timeout)
- Simple command line interface
- Exit codes for automation

### `consolidate-migrations.sh` - Migration Management

Consolidate multiple database migrations into a single migration file.

**Usage:**
```bash
./scripts/consolidate-migrations.sh
```

**Features:**
- Combines multiple migration files
- Maintains migration order
- Creates backup of original migrations

### `find_orphaned_ftl_keys.sh` - Translation Key Analysis

Find translation keys that are defined but not used in the codebase.

**Usage:**
```bash
./scripts/find_orphaned_ftl_keys.sh
```

**Features:**
- Scans all .ftl files for defined keys
- Searches codebase for key usage
- Reports orphaned/unused keys

### `bulk_remove_ftl_keys.sh` - Translation Key Cleanup

Remove orphaned translation keys from .ftl files.

**Usage:**
```bash
./scripts/bulk_remove_ftl_keys.sh
```

**Features:**
- Removes unused translation keys
- Creates backup before removal
- Supports multiple language files

### `remove_duplicate_ftl_keys.sh` - Translation Key Deduplication

Remove duplicate translation keys from all language files.

**Usage:**
```bash
./scripts/remove_duplicate_ftl_keys.sh
```

**Features:**
- Removes duplicate keys while preserving first occurrence
- Maintains original key order
- Creates backup before removal
- Supports all language files (en-US, de-DE, fr-FR, es-ES, nb-NO)

## Common Use Cases

### Testing API Endpoints

```bash
# Start with login
./scripts/curl-helpers.sh login

# Test various endpoints
./scripts/curl-helpers.sh backup-list
./scripts/curl-helpers.sh aliases-list
./scripts/curl-helpers.sh domains-list

# Create and manage backups
./scripts/curl-helpers.sh backup-create primary
./scripts/curl-helpers.sh backup-download filename.sql
./scripts/curl-helpers.sh backup-delete filename.sql

# Check system status
./scripts/curl-helpers.sh health
./scripts/curl-helpers.sh stats
```

### Development Workflow

```bash
# Set up environment
source ./scripts/set-env.sh

# List configured databases
./scripts/list-databases.sh

# Generate password hash for new admin
./scripts/generate_password_hash.sh "new_password"

# Test API functionality
./scripts/curl-helpers.sh login
./scripts/curl-helpers.sh backup-list
```

### Maintenance Tasks

```bash
# Find unused translation keys
./scripts/find_orphaned_ftl_keys.sh

# Clean up translation files
./scripts/bulk_remove_ftl_keys.sh

# Consolidate migrations
./scripts/consolidate-migrations.sh
```

## Requirements

- **Bash**: All scripts require bash shell
- **curl**: For API testing (curl-helpers.sh)
- **jq**: For JSON parsing (optional, for pretty output)
- **Python 3 or Node.js**: For password hash generation
- **Docker Compose**: For environment setup (set-env.sh)

## Security Notes

- Cookie files are stored in `/tmp/` and should be cleaned up automatically
- Password hashes should be generated on secure systems
- Environment variables may contain sensitive information
- Scripts should be run with appropriate permissions

## Troubleshooting

### Common Issues

1. **Permission Denied**: Make scripts executable with `chmod +x scripts/*.sh`
2. **Login Fails**: Check application is running and credentials are correct
3. **JSON Parsing Errors**: Install `jq` for better output formatting
4. **Missing Dependencies**: Install required tools (Python, Node.js, etc.)

### Debug Mode

Enable debug output by setting environment variables:
```bash
export RUST_LOG=debug
export SORTINGOFFICE_DEBUG=1
```

## Contributing

When adding new scripts:
1. Follow the existing naming convention
2. Include proper error handling
3. Add usage examples
4. Update this README
5. Make scripts executable
6. Test with different environments 
