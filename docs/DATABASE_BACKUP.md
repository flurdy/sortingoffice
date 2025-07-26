# Database Backup Feature

The Sorting Office application includes a comprehensive database backup feature that allows administrators to create, download, and manage database backups directly through the web interface.

## Overview

The Database Backup feature provides:
- **Web-based backup creation** for any configured database
- **Automatic backup file management** with timestamped naming
- **Secure download** of backup files
- **Backup listing and deletion** capabilities
- **Multi-language support** for all backup operations

## Accessing the Backup Feature

1. **Navigate to**: `/backup` in your web browser
2. **Authentication**: Requires admin login with appropriate permissions
3. **Database Selection**: Choose from any configured database in your `config.toml`

## Features

### Creating Backups

1. **Select Database**: Choose the database you want to backup from the dropdown
2. **Create Backup**: Click the "Create Backup" button
3. **Progress Tracking**: The interface shows real-time progress during backup creation
4. **Success Confirmation**: Upon completion, the backup appears in the existing backups list

### Backup File Naming

Backup files are automatically named using the format:
```
{database_id}_{database_name}_{YYYYMMDD}_{HHMMSS}.sql
```

**Examples:**
- `primary_sortingoffice_20250726_175827.sql`
- `backup1_sortingoffice_backup_20250726_175827.sql`

### Managing Existing Backups

The backup interface displays a comprehensive table showing:

| Column | Description |
|--------|-------------|
| **Database** | Database name and ID |
| **Created** | Timestamp when backup was created |
| **Size** | File size in human-readable format |
| **Filename** | Full backup filename |
| **Actions** | Download and Delete buttons |

### Downloading Backups

1. **Click Download**: Use the download button in the Actions column
2. **File Download**: The backup file downloads to your local machine
3. **Security**: Downloads require authentication and are validated

### Deleting Backups

1. **Click Delete**: Use the delete button in the Actions column
2. **Confirmation**: Confirm the deletion in the browser dialog
3. **Removal**: The backup file is permanently deleted from the server

## Technical Details

### Backup Process

1. **mysqldump Command**: Uses MySQL's `mysqldump` utility
2. **Minimal Privileges**: Configured to work with basic SELECT privileges
3. **File Storage**: Backups stored in application's backup directory
4. **Error Handling**: Comprehensive error reporting and user feedback

### Security Features

- **Authentication Required**: All backup operations require admin login
- **File Validation**: Backup files are validated before download
- **Path Security**: Prevents directory traversal attacks
- **Session Management**: Proper session handling for all operations

### Database Compatibility

The backup feature works with:
- **MySQL 5.7+** and **MariaDB 10.2+**
- **Any configured database** in your `config.toml`
- **Databases with underscores** in their names (properly parsed)

## Configuration

### Backup Directory

Backups are stored in the application's backup directory. Ensure:
- **Writable Directory**: The application has write permissions
- **Sufficient Space**: Adequate disk space for backup files
- **Backup Retention**: Implement your own backup rotation strategy

### Database Permissions

The backup feature requires minimal database privileges:
```sql
-- Basic privileges needed for backup
GRANT SELECT ON database_name.* TO 'username'@'host';
```

**Note**: The feature is designed to work without requiring `PROCESS` privilege.

## Internationalization

The backup interface supports multiple languages:
- **English** (en-US)
- **Spanish** (es-ES) 
- **German** (de-DE)
- **French** (fr-FR)
- **Norwegian** (nb-NO)

All table headers, buttons, and messages are properly translated.

## API Endpoints

The backup feature provides these REST endpoints:

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/backup` | Backup management page |
| `POST` | `/backup/create-htmx` | Create new backup (HTMX) |
| `GET` | `/backup/list` | List existing backups (JSON) |
| `GET` | `/backup/download/{filename}` | Download backup file |
| `DELETE` | `/backup/delete/{filename}` | Delete backup file |

## Troubleshooting

### Common Issues

1. **Permission Denied**
   - Ensure the backup directory is writable
   - Check database user permissions

2. **mysqldump Not Found**
   - Install MySQL client tools
   - Verify `mysqldump` is in PATH

3. **Backup Creation Fails**
   - Check database connectivity
   - Verify database user has SELECT privileges
   - Review application logs for detailed error messages

4. **Download Fails**
   - Verify file exists on server
   - Check file permissions
   - Ensure proper authentication

### Logging

Enable debug logging to troubleshoot backup issues:
```bash
export RUST_LOG=debug
```

Look for backup-related log entries in the application output.

## Best Practices

1. **Regular Backups**: Schedule regular database backups
2. **Test Restores**: Periodically test backup restoration
3. **Offsite Storage**: Store backups in multiple locations
4. **Retention Policy**: Implement backup rotation and cleanup
5. **Monitoring**: Monitor backup success/failure rates

## Integration with Existing Features

The Database Backup feature integrates seamlessly with:
- **Multi-database support**: Backup any configured database
- **Authentication system**: Uses existing admin authentication
- **Internationalization**: Supports all configured languages
- **Configuration management**: Respects database configuration settings

## Future Enhancements

Potential future improvements:
- **Scheduled backups**: Automated backup scheduling
- **Backup compression**: Gzip compression for smaller files
- **Backup encryption**: Encrypted backup storage
- **Cloud storage**: Integration with cloud backup services
- **Backup verification**: Automatic backup integrity checking 
