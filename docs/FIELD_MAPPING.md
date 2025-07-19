# Field Mapping for Multi-Database Support

This document explains how to use the table-qualified field mapping system to support different database schemas across multiple databases.

## Overview

The field mapping system allows you to map logical field names to actual database field names for each database configuration. This is useful when different databases have different schema field names but you want to use consistent logical names in your application code.

## Configuration Format

Field mappings are configured in the `config.toml` file using table-qualified field names:

```toml
[[databases]]
id = "primary"
label = "Main Server"
url = "mysql://user:pass@localhost/db"

[databases.field_map]
# Users table field mappings
users.id = "user_id"
users.enabled = "is_active"
users.email = "email_address"

# Domains table field mappings
domains.id = "domain_id"
domains.enabled = "is_enabled"

# Aliases table field mappings
aliases.id = "alias_id"
aliases.enabled = "is_active"
```

## Field Mapping Syntax

### Table-Qualified Field Names

Use the format `table.field` to specify which table a field mapping applies to:

- `users.id` - Maps the `id` field in the `users` table
- `domains.enabled` - Maps the `enabled` field in the `domains` table
- `aliases.mail` - Maps the `mail` field in the `aliases` table

### Backward Compatibility

Simple field names without table qualification are still supported for backward compatibility:

```toml
[databases.field_map]
enabled = "is_active"  # Applies to all tables if not overridden
```

## Usage in Code

### Using the Field Mapping API

```rust
use crate::config::DatabaseConfig;

// Get the mapped field name for a specific table and field
let user_id_field = db_config.field_for_table("users", "id");
let domain_enabled_field = db_config.field_for_table("domains", "enabled");

// Backward compatibility - works with simple field names
let enabled_field = db_config.field("enabled");
```

### Database Functions with Field Mapping

The system provides field-mapped versions of database functions:

```rust
// Get users with field mapping
let users = get_users_with_field_map(&pool, &db_config)?;

// Get domains with field mapping
let domains = get_domains_with_field_map(&pool, &db_config)?;

// Get aliases with field mapping
let aliases = get_aliases_with_field_map(&pool, &db_config)?;
```

### Building Custom Queries

Use the helper function to build field-mapped queries:

```rust
use crate::db::build_field_mapped_query;

let fields = [
    ("id", "pkid"),
    ("domain", "domain"),
    ("enabled", "enabled"),
];

let sql = build_field_mapped_query("domains", &fields, &db_config);
// Result: "SELECT domain_id as pkid, domain as domain, is_enabled as enabled FROM domains"
```

## Field Mapping Resolution

The system resolves field mappings in the following order:

1. **Table-qualified field name**: `users.id` → `user_id`
2. **Simple field name**: `id` → `user_id` (if no table-qualified mapping exists)
3. **Original field name**: `id` → `id` (if no mapping exists)

## Example Configurations

### Different Schema Names

```toml
[[databases]]
id = "legacy"
label = "Legacy Database"
url = "mysql://user:pass@legacy/db"

[databases.field_map]
users.id = "user_id"
users.enabled = "is_active"
domains.id = "domain_id"
domains.enabled = "is_enabled"

[[databases]]
id = "modern"
label = "Modern Database"
url = "mysql://user:pass@modern/db"

[databases.field_map]
users.id = "id"
users.enabled = "enabled"
domains.id = "id"
domains.enabled = "enabled"
```

### Mixed Schema Support

```toml
[[databases]]
id = "mixed"
label = "Mixed Schema Database"
url = "mysql://user:pass@mixed/db"

[databases.field_map]
# Some tables use different field names
users.id = "user_id"
users.enabled = "is_active"

# Other tables use standard names
domains.id = "id"
domains.enabled = "enabled"
aliases.id = "id"
aliases.enabled = "enabled"
```

## Supported Tables

The field mapping system currently supports the following tables:

- `users` - User accounts
- `domains` - Domain configurations
- `aliases` - Email aliases
- `backups` - Backup domain configurations
- `relays` - Relay configurations
- `relocated` - Relocated email addresses
- `clients` - Client configurations

## Best Practices

1. **Use table-qualified field names** for clarity and to avoid conflicts
2. **Document your field mappings** in comments within the config file
3. **Test field mappings** with different database schemas
4. **Use consistent logical names** across your application code
5. **Provide fallback mappings** for backward compatibility when possible

## Migration from Simple Field Names

If you're migrating from the old simple field mapping system:

1. Update your config to use table-qualified field names
2. Test the new mappings thoroughly
3. Update any custom queries to use the new field mapping functions
4. Remove old simple field mappings once migration is complete

## Troubleshooting

### Common Issues

1. **Field not found**: Ensure the field mapping exists for the specific table
2. **Wrong field name**: Check that the mapped field name matches the actual database schema
3. **Case sensitivity**: Database field names are case-sensitive
4. **Table name mismatch**: Verify the table name in the field mapping matches your schema

### Debugging

Enable debug logging to see field mapping resolution:

```rust
tracing::debug!("Field mapping for users.id: {}", db_config.field_for_table("users", "id"));
``` 
