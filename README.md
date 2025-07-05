# Sorting Office

A modern web-based admin tool for managing mail server data based on [flurdy's "How to set up a mail server on a GNU / Linux system"](https://flurdy.com/docs/postfix/).

## Versions

- v3: The current Rust based setup
- v1: The original Play Framework with Scala based setup,
   refer to the [git tag *v1*](https://github.com/flurdy/sortingoffice/tree/v1)
 

## Features

- **Domain Management**: Add, edit, and remove mail domains with quota and transport settings
- **User Management**: Manage mail users with password hashing and quota allocation
- **Alias Management**: Create and manage email aliases for forwarding
- **Mailbox Management**: Handle IMAP/POP3 mailboxes with individual settings
- **Statistics Dashboard**: View system-wide and per-domain statistics
- **Modern UI**: Built with Tailwind CSS for a clean, responsive interface
- **Dark Mode Support**: Toggle between light and dark themes with persistent preference
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
- Diesel CLI

### Prerequisites hint
- Install rustup

      sudo dnf install rustup

  or 
      
       brew install rustup

- Initialise rustup
       
       rustup-init

- Install *cargo binstall*

       cargo install cargo-binstall

- install Diesel CLI binary

       cargo binstall diesel-cli

  Or MySQL only 

       cargo install diesel_cli --no-default-features --features mysql

## Installation

### Option 1: Docker (Recommended)

The easiest way to run Sorting Office is using Docker Compose:

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd sortingoffice
   ```

2. **Build and start services**:
   ```bash
   # Make the Docker script executable
   chmod +x docker.sh
   
   # Build and start all services
   ./docker.sh build
   ./docker.sh up
   ```

3. **Access the application**:
   - **Sorting Office**: http://localhost:3000
   - **phpMyAdmin**: http://localhost:8080
   - **Default login**: admin/admin

**Docker Management Commands**:
```bash
./docker.sh help          # Show all available commands
./docker.sh status        # Check service status
./docker.sh logs          # View logs
./docker.sh down          # Stop services
./docker.sh restart       # Restart services
./docker.sh clean         # Remove all containers and volumes
```

### Option 2: Local Development

1. **Prerequisites**:
   - Rust 1.70+ and Cargo
   - MySQL 8.0+ or MariaDB 10.5+
   - Diesel CLI

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

### Theme Toggle
The application supports both light and dark themes:

- **Toggle Button**: Click the sun/moon icon in the top navigation bar
- **Persistent Preference**: Your theme choice is saved in localStorage
- **Automatic Detection**: The theme preference is restored on page reload
- **Server Integration**: Theme changes are also sent to the server for potential future server-side persistence

The theme toggle works across all pages including:
- Dashboard
- Domain management
- User management
- Alias management
- Mailbox management
- Statistics
- Login page

## Docker

### Production Deployment

The Docker setup includes:

- **Application**: Rust application with optimized multi-stage build
- **Database**: MySQL 8.0 with persistent storage
- **phpMyAdmin**: Web-based database management interface
- **Networking**: Isolated network for secure communication
- **Health Checks**: Automatic health monitoring for all services

### Development Environment

For development with live code reloading:

```bash
# Start development environment
./docker.sh dev

# Stop development environment
./docker.sh dev-down
```

The development environment includes:
- Volume mounts for live code changes
- Debug logging enabled
- Exposed database ports for direct access
- Development-specific Dockerfile

### Docker Compose Files

- `docker-compose.yml`: Production configuration
- `docker-compose.dev.yml`: Development overrides
- `Dockerfile`: Production-optimized multi-stage build
- `Dockerfile.dev`: Development environment with tools

### Environment Variables

Docker environment variables are configured in `docker-compose.yml`:

```yaml
environment:
  DATABASE_URL: mysql://sortingoffice:sortingoffice@db:3306/sortingoffice
  RUST_LOG: info
  HOST: 0.0.0.0
  PORT: 3000
```

### Volumes

- `mysql_data`: Persistent MySQL data storage
- `./templates`: Template files (read-only)
- `./migrations`: Database migration files

### Ports

- `3000`: Sorting Office web application
- `3306`: MySQL database (exposed for development)
- `8080`: phpMyAdmin interface

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

- Based on [flurdy's Postfix mail server guide](https://flurdy.com/docs/postfix/)
- Built with modern Rust web development tools
- UI inspired by modern admin dashboard designs
