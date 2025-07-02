# Sorting Office

A modern web-based admin tool for managing mail server data based on [flurdy's "How to set up a mail server on a GNU / Linux system"](http://flurdy.com/docs/postfix/).

## Features

- **Domain Management**: Add, edit, and remove mail domains with quota and transport settings
- **User Management**: Manage mail users with password hashing and quota allocation
- **Alias Management**: Create and manage email aliases for forwarding
- **Mailbox Management**: Handle IMAP/POP3 mailboxes with individual settings
- **Statistics Dashboard**: View system-wide and per-domain statistics
- **Modern UI**: Built with Tailwind CSS for a clean, responsive interface
- **HTMX Integration**: Dynamic updates without full page reloads
- **Secure Authentication**: Password hashing and session management

## Technology Stack

- **Backend**: Rust with Axum web framework
- **Database**: MySQL with Diesel ORM
- **Frontend**: HTMX for dynamic interactions
- **Styling**: Tailwind CSS for modern UI
- **Templates**: Askama template engine
- **Authentication**: bcrypt password hashing

## Prerequisites

- Rust 1.70+ and Cargo
- MySQL 8.0+ or MariaDB 10.5+
- Diesel CLI (`cargo install diesel_cli --no-default-features --features mysql`)

## Installation

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd sortingoffice
   ```

2. **Set up the database**:
   ```bash
   # Create MySQL database
   mysql -u root -p
   CREATE DATABASE sortingoffice CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
   CREATE USER 'sortingoffice'@'localhost' IDENTIFIED BY 'your_password';
   GRANT ALL PRIVILEGES ON sortingoffice.* TO 'sortingoffice'@'localhost';
   FLUSH PRIVILEGES;
   EXIT;
   ```

3. **Configure environment**:
   ```bash
   cp env.example .env
   # Edit .env with your database credentials
   ```

4. **Run database migrations**:
   ```bash
   diesel setup
   diesel migration run
   ```

5. **Build and run**:
   ```bash
   cargo build --release
   cargo run
   ```

The application will be available at `http://localhost:3000`.

## Default Login

- **Username**: `admin`
- **Password**: `admin`

**Important**: Change these credentials in production!

## Database Schema

The application uses the following tables based on flurdy's mail server schema:

### Domains
- `id`: Primary key
- `domain`: Domain name (unique)
- `description`: Optional description
- `aliases`: Maximum number of aliases
- `mailboxes`: Maximum number of mailboxes
- `maxquota`: Maximum quota in bytes
- `quota`: Current quota in bytes
- `transport`: Mail transport method
- `backupmx`: Backup MX flag
- `active`: Active status
- `created`/`modified`: Timestamps

### Users
- `id`: Primary key
- `username`: Username (unique)
- `password`: bcrypt hashed password
- `name`: Display name
- `maildir`: Mail directory path
- `quota`: User quota in bytes
- `domain`: Associated domain
- `active`: Active status
- `created`/`modified`: Timestamps

### Aliases
- `id`: Primary key
- `address`: Email address (unique)
- `goto`: Forwarding address
- `domain`: Associated domain
- `active`: Active status
- `created`/`modified`: Timestamps

### Mailboxes
- `id`: Primary key
- `username`: Username (unique)
- `password`: bcrypt hashed password
- `name`: Display name
- `maildir`: Mail directory path
- `quota`: Mailbox quota in bytes
- `domain`: Associated domain
- `active`: Active status
- `created`/`modified`: Timestamps

## Usage

### Dashboard
The main dashboard shows:
- System statistics (total domains, users, aliases, mailboxes)
- Quick action buttons for common tasks
- Overview of mail server health

### Domain Management
- View all domains in a table format
- Add new domains with quota and transport settings
- Edit existing domain configurations
- Enable/disable domains
- Set backup MX status

### User Management
- Create new mail users
- Set individual quotas
- Manage user passwords securely
- Enable/disable users
- View user details and statistics

### Alias Management
- Create email forwarding aliases
- Point aliases to multiple destinations
- Manage alias domains
- Enable/disable aliases

### Mailbox Management
- Create IMAP/POP3 mailboxes
- Set mailbox-specific quotas
- Manage mailbox passwords
- Configure mail directories

## Development

### Project Structure
```
src/
├── main.rs              # Application entry point
├── models.rs            # Database models
├── schema.rs            # Diesel schema
├── db.rs                # Database operations
├── handlers/            # HTTP request handlers
│   ├── mod.rs
│   ├── dashboard.rs
│   ├── auth.rs
│   ├── domains.rs
│   ├── users.rs
│   ├── aliases.rs
│   ├── mailboxes.rs
│   └── stats.rs
└── templates/           # Askama template definitions
    ├── mod.rs
    ├── base.rs
    ├── dashboard.rs
    ├── auth.rs
    ├── domains.rs
    ├── users.rs
    ├── aliases.rs
    ├── mailboxes.rs
    └── stats.rs

templates/               # HTML templates
├── layout.html
├── login.html
├── dashboard.html
├── domains/
├── users/
├── aliases/
├── mailboxes/
└── stats.html

migrations/              # Database migrations
└── *.sql
```

### Adding New Features

1. **Database Changes**: Create new migration files
2. **Models**: Add new models in `src/models.rs`
3. **Database Operations**: Add functions in `src/db.rs`
4. **Handlers**: Create new handlers in `src/handlers/`
5. **Templates**: Add template definitions and HTML files
6. **Routes**: Update routes in `src/main.rs`

### Testing
```bash
# Run tests
cargo test

# Run with specific log level
RUST_LOG=debug cargo run
```

## Security Considerations

- All passwords are hashed using bcrypt
- SQL injection protection via Diesel ORM
- CSRF protection for forms
- Input validation and sanitization
- Secure session management

## Production Deployment

1. **Environment**: Set production environment variables
2. **Database**: Use production MySQL/MariaDB instance
3. **Reverse Proxy**: Configure nginx/Apache as reverse proxy
4. **SSL/TLS**: Enable HTTPS with proper certificates
5. **Authentication**: Implement proper admin authentication
6. **Backup**: Set up regular database backups
7. **Monitoring**: Configure logging and monitoring

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Based on [flurdy's Postfix mail server guide](http://flurdy.com/docs/postfix/)
- Built with modern Rust web development tools
- UI inspired by modern admin dashboard designs
