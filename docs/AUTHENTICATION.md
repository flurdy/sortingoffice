# Authentication System

Sorting Office implements a secure, role-based authentication system with support for multiple admin users and different permission levels.

## Overview

The authentication system provides:
- **Multiple Admin Support**: Configure multiple admin users with different roles
- **Role-Based Access Control**: Two distinct permission levels (Read-Only and Edit)
- **Secure Password Storage**: bcrypt hashing for all passwords
- **Session Management**: HTTP-only cookies with expiration
- **Backward Compatibility**: Support for legacy single-admin configurations

## Roles and Permissions

### AdminRole::ReadOnly
Users with read-only permissions can:
- View all data (domains, users, aliases, backups, etc.)
- Access dashboard and statistics
- View configuration settings
- Access reports and system information

**Routes accessible to read-only users:**
- `GET /` (dashboard)
- `GET /domains` (list)
- `GET /domains/{id}` (show)
- `GET /users` (list)
- `GET /users/{id}` (show)
- `GET /aliases` (list)
- `GET /aliases/{id}` (show)
- `GET /backups/{id}` (show)
- `GET /stats`
- `GET /reports`
- `GET /about`
- `GET /config`

### AdminRole::Edit
Users with edit permissions can perform all read-only operations plus:
- Create, update, and delete domains
- Create, update, and delete users
- Create, update, and delete aliases
- Create, update, and delete backups
- Toggle enabled/disabled status for all resources
- Access all edit forms and actions

**Additional routes accessible to edit users:**
- `POST /domains` (create)
- `PUT /domains/{id}` (update)
- `DELETE /domains/{id}` (delete)
- `GET /domains/new` (create form)
- `GET /domains/{id}/edit` (edit form)
- `POST /domains/{id}/toggle` (toggle status)
- Similar patterns for users, aliases, and backups

## Configuration

### Single Admin Configuration (Legacy)

For backward compatibility, you can still use the old single-admin format:

```toml
[admin]
username = "admin"
password_hash = "$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW"
role = "edit"
```

### Multiple Admins Configuration (Recommended)

For production use, configure multiple admins with different roles:

```toml
[[admins]]
username = "admin"
password_hash = "$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW"
role = "edit"

[[admins]]
username = "viewer"
password_hash = "$2b$12$AnotherHashHere..."
role = "read-only"

[[admins]]
username = "operator"
password_hash = "$2b$12$YetAnotherHash..."
role = "edit"
```

### Configuration File Location

The authentication configuration is loaded from:
1. `config/config.toml` (if exists)
2. Environment variables (if config file not found)
3. Default values (fallback)

### Environment Variables

You can also configure admins using environment variables:

```bash
# Single admin (legacy)
export ADMIN_USERNAME="admin"
export ADMIN_PASSWORD_HASH="$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW"
export ADMIN_ROLE="edit"

# Multiple admins (JSON format)
export ADMINS='[
  {
    "username": "admin",
    "password_hash": "$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW",
    "role": "edit"
  },
  {
    "username": "viewer", 
    "password_hash": "$2b$12$AnotherHashHere...",
    "role": "read-only"
  }
]'
```

## Password Management

### Generating Password Hashes

To generate a bcrypt hash for a new password, you can use:

```bash
# Using Python
python3 -c "import bcrypt; print(bcrypt.hashpw('your_password'.encode('utf-8'), bcrypt.gensalt()).decode('utf-8'))"

# Using Node.js
node -e "const bcrypt = require('bcrypt'); bcrypt.hash('your_password', 12).then(hash => console.log(hash))"

# Using online bcrypt generators (for development only)
# https://bcrypt.online/
```

### Default Credentials

**Development Default:**
- Username: `admin`
- Password: `admin123`
- Hash: `$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW`

**⚠️ Important:** Change these credentials in production!

## Session Management

### Cookie Configuration

Authentication cookies are configured with:
- **HttpOnly**: Prevents XSS attacks
- **SameSite=Lax**: CSRF protection
- **Path=/**: Available across the entire site
- **Expiration**: 24 hours from login
- **Format**: `authenticated={expiry}:{role}`

### Session Security

- Cookies are automatically cleared on logout
- Expired sessions redirect to login
- Invalid cookies are ignored
- Role information is embedded in the cookie

## Security Considerations

### Best Practices

1. **Strong Passwords**: Use strong, unique passwords for each admin
2. **Role Separation**: Use read-only accounts for monitoring and edit accounts for administration
3. **Regular Rotation**: Change passwords regularly
4. **HTTPS Only**: Always use HTTPS in production
5. **Network Security**: Restrict access to trusted networks

### Security Features

- **bcrypt Hashing**: Industry-standard password hashing with configurable cost
- **Session Expiration**: Automatic logout after 24 hours
- **CSRF Protection**: SameSite cookie attribute
- **XSS Protection**: HttpOnly cookies
- **Input Validation**: All inputs are validated and sanitized

## Error Handling

### Authentication Errors

- **Invalid Credentials**: Clear error message without revealing valid usernames
- **Expired Sessions**: Automatic redirect to login
- **Insufficient Permissions**: 403 Forbidden for unauthorized actions
- **Unknown Routes**: 403 for authenticated users, 404 for anonymous users

### Error Messages

The system provides localized error messages for:
- Invalid username/password combinations
- Expired sessions
- Insufficient permissions
- System errors

## Testing Authentication

### Unit Tests

The authentication system includes comprehensive tests:

```bash
# Run authentication tests
cargo test test_login_success
cargo test test_login_failure
cargo test test_role_based_access_control
cargo test test_is_authenticated_cookie
cargo test test_has_edit_permissions
```

### Manual Testing

1. **Login Test**: Try logging in with valid/invalid credentials
2. **Role Test**: Verify read-only users can't access edit functions
3. **Session Test**: Check that sessions expire correctly
4. **Logout Test**: Verify cookies are cleared on logout

## Troubleshooting

### Common Issues

1. **Login Fails**: Check password hash format and bcrypt cost
2. **Permission Denied**: Verify user has correct role for the action
3. **Session Expires**: Check system clock and cookie expiration
4. **Configuration Not Loaded**: Verify config file syntax and location

### Debug Mode

Enable debug logging to troubleshoot authentication issues:

```bash
RUST_LOG=debug cargo run
```

This will show:
- Configuration loading details
- Authentication attempts
- Session management
- Permission checks

## Migration from Legacy System

If you're upgrading from the old single-admin system:

1. **Backup Configuration**: Save your current admin credentials
2. **Update Config**: Convert to the new multiple-admin format
3. **Test Login**: Verify all admins can log in with correct roles
4. **Update Documentation**: Update any internal documentation

### Example Migration

**Old Format:**
```toml
[admin]
username = "admin"
password_hash = "$2b$12$oldhash..."
```

**New Format:**
```toml
[[admins]]
username = "admin"
password_hash = "$2b$12$oldhash..."
role = "edit"
```

## API Reference

### Authentication Functions

```rust
// Check if user is authenticated
pub fn is_authenticated(headers: &HeaderMap) -> bool

// Get user's role
pub fn get_user_role(headers: &HeaderMap) -> Option<AdminRole>

// Check if user has edit permissions
pub fn has_edit_permissions(headers: &HeaderMap) -> bool

// Verify admin credentials
pub fn verify_admin_credentials(&self, username: &str, password: &str) -> Option<AdminRole>
```

### Middleware

```rust
// Require authentication for all routes
pub async fn require_auth(...) -> Result<Response, StatusCode>

// Require edit permissions for modifying routes
pub async fn require_edit_permissions(...) -> Result<Response, StatusCode>
```

## Related Documentation

- [Database Management](DATABASE_MANAGEMENT.md)
- [Configuration Guide](../README.md#configuration)
- [Security Considerations](../README.md#security-considerations)
- [Production Deployment](../README.md#production-deployment) 
