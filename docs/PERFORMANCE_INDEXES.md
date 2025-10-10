# Performance Index Recommendations

This document describes recommended database indexes for optimal performance, especially on production databases with large datasets.

## Recent Changes Report Indexes

The recent changes report queries the `modified` column on all tables with `ORDER BY modified DESC LIMIT 50`. Without indexes, these queries can be slow on large tables.

### Recommended Indexes

```sql
-- Add indexes on modified columns for faster ORDER BY operations
CREATE INDEX IF NOT EXISTS idx_domains_modified ON domains(modified DESC);
CREATE INDEX IF NOT EXISTS idx_users_modified ON users(modified DESC);
CREATE INDEX IF NOT EXISTS idx_aliases_modified ON aliases(modified DESC);
CREATE INDEX IF NOT EXISTS idx_backups_modified ON backups(modified DESC);
CREATE INDEX IF NOT EXISTS idx_relays_modified ON relays(modified DESC);
CREATE INDEX IF NOT EXISTS idx_relocated_modified ON relocated(modified DESC);
CREATE INDEX IF NOT EXISTS idx_clients_modified ON clients(modified DESC);
```

## Domain-based Lookups

Many queries filter by domain parts of email addresses. While these use LIKE queries which can't be fully indexed, partial indexes can still help:

```sql
-- Indexes for domain-based filtering (MySQL 8.0+ supports functional indexes)
CREATE INDEX IF NOT EXISTS idx_aliases_mail ON aliases(mail);
CREATE INDEX IF NOT EXISTS idx_users_id ON users(id);
CREATE INDEX IF NOT EXISTS idx_relays_recipient ON relays(recipient);
CREATE INDEX IF NOT EXISTS idx_relocated_old_address ON relocated(old_address);
```

## Applying Indexes

To apply these indexes to your production database:

```bash
mysql -u root -p sortingoffice < docs/PERFORMANCE_INDEXES.md
```

Or run each CREATE INDEX statement individually through your database admin tool.

## Monitoring

After adding indexes, monitor:
- Query execution time in application logs (look for timing logs)
- Database slow query log
- Index usage statistics: `SHOW INDEX FROM table_name;`

## Notes

- Indexes improve read performance but slightly slow down writes
- On small databases (< 10,000 records), indexes may not provide noticeable benefit
- Monitor disk space usage as indexes consume additional storage
- Consider running `ANALYZE TABLE` after creating indexes to update statistics
