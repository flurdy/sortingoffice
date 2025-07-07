# Seed Data Directory

This directory contains seed data files for the Sorting Office database. These files are **NOT** run automatically by diesel migrations and must be applied manually.

## File Structure

```
seed_data/
├── README.md          # This file
├── all.sql            # Master seed file (includes all data in correct order)
├── domains.sql        # Domain seed data only
├── users.sql          # User seed data only
├── aliases.sql        # Alias seed data only
└── backups.sql        # Backup MX seed data only
```

## Usage

### Load All Seed Data
```bash
make seed
```

### Load Individual Tables
```bash
make seed-domains    # Load only domains
make seed-users      # Load only users
make seed-aliases    # Load only aliases
make seed-backups    # Load only backups
```

### Manual Loading
```bash
# Load all seed data
mysql -u root -p sortingoffice < seed_data/all.sql

# Load individual files
mysql -u root -p sortingoffice < seed_data/domains.sql
mysql -u root -p sortingoffice < seed_data/users.sql
mysql -u root -p sortingoffice < seed_data/aliases.sql
mysql -u root -p sortingoffice < seed_data/backups.sql
```

## Data Dependencies

The seed data files respect foreign key constraints:

1. **domains.sql** - Must be loaded first (no dependencies)
2. **users.sql** - No foreign key constraints (domain field removed)
3. **aliases.sql** - No foreign key constraints (domain derived from mail field)
4. **backups.sql** - No foreign key constraints

The `all.sql` file loads data in the correct order automatically.

## Default Data

The seed data includes:

### Domains
- `example.com` (virtual transport)
- `example.org` (virtual transport)
- `test.com` (smtp:localhost transport)

### Users
- `admin@example.com` (Administrator)
- `user1@example.com` (Test User 1)
- `user2@example.org` (Test User 2)

**Password**: All users have password `password123` (bcrypt hashed)

### Aliases
- Standard email aliases (postmaster, abuse, webmaster)
- Catch-all aliases (@example.com, @example.org)
- User aliases

### Backups
- `backup.example.com` (enabled)
- `mx2.example.org` (enabled)
- `fallback.example.net` (disabled)

## Customization

You can modify any of these files to add your own seed data:

1. **Edit the files** in this directory
2. **Run the seed command** to apply changes
3. **Test your application** to ensure everything works

## Creating New Seed Data

To create new seed data files:

```bash
# Create default seed data structure
make create-seed-data

# Edit the generated files
nano seed_data/domains.sql
nano seed_data/users.sql
# etc.

# Load the seed data
make seed
```

## Important Notes

- **Seed data is NOT part of migrations** - it won't be run automatically
- **Seed data can be run multiple times** - use `INSERT IGNORE` or `ON DUPLICATE KEY UPDATE` if needed
- **Seed data is for development/testing** - don't use production data in seed files
- **Backup your data** before running seed data on production databases 
