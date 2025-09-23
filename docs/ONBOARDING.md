# Sorting Office - Onboarding Guide

This guide provides step-by-step instructions for setting up and running the Sorting Office application, whether you're deploying to production or setting up a development environment.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Getting the Code](#getting-the-code)
3. [Building the Application](#building-the-application)
4. [Database Setup](#database-setup)
5. [Configuration](#configuration)
6. [Schema Preparation](#schema-preparation)
7. [Production Safety](#production-safety)
8. [Starting the Application](#starting-the-application)
9. [First Login](#first-login)
10. [Troubleshooting](#troubleshooting)

## Prerequisites

### Backup

> ⚠️ **Warning**  
> All the data in your production mail databases are at risk!

Before you let this application anywhere near your live mail databases,
back them up. 

Once up and runnig there is backup feature in this app as well.

You never know if there any accidental database commands or broken migration 
which deletes any existing data, so better be safe and do an initial manual backup.

### Tools

Before you begin, ensure you have the following installed:

- **Rust** (latest stable version)
- **MySQL** (5.7+ or 8.0+)
- **Docker** and **Docker Compose** (optional, for containerized setup)
- **Make** (Optional, for using the provided Makefile commands)

#### Rust

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

## Getting the Code

### Option 1: Fork and Clone (Recommended for Development)

1. **Fork the repository** on GitHub
2. **Clone your fork**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/sortingoffice.git
   cd sortingoffice
   ```

### Option 2: Direct Clone (For Quick Testing)

```bash
git clone https://github.com/flurdy/sortingoffice.git
cd sortingoffice
```

### Option 3: Download Release (For Production)

1. Go to the [releases page](https://github.com/flurdy/sortingoffice/releases)
2. Download the latest release archive
3. Extract and navigate to the directory

## Building the Application

### Option 1: Build from Source (Development)

```bash
# Install dependencies and build
cargo build --release

# The binary will be available at target/release/sortingoffice
```

### Option 2: Docker Build (Recommended for Production)

```bash
# Build the Docker image
docker build -t sortingoffice:latest .

# Or use the development Dockerfile
docker build -f Dockerfile.dev -t sortingoffice:dev .
```

### Option 3: Use Pre-built Image

```bash
# Pull the official image (if available)
docker pull flurdy/sortingoffice:latest
```

## Database Setup

### Option 1: Docker Compose (Recommended)

The project includes Docker Compose configurations for easy database setup:

```bash
# Start all services (database + application)
docker-compose up -d

# Or start just the database
docker-compose up -d mysql
```

### Option 2: Manual MySQL Setup

1. **Create the database**:
   ```sql
   CREATE DATABASE sortingoffice CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
   ```

2. **Create a user**:
   ```sql
   CREATE USER 'sortingoffice'@'%' IDENTIFIED BY 'your_secure_password';
   GRANT ALL PRIVILEGES ON sortingoffice.* TO 'sortingoffice'@'%';
   FLUSH PRIVILEGES;
   ```

3. **Set environment variables**:
   ```bash
   export MYSQL_HOST=localhost
   export MYSQL_PORT=3306
   export MYSQL_DATABASE=sortingoffice
   export MYSQL_USER=sortingoffice
   export MYSQL_PASSWORD=your_secure_password
   ```

### Option 3: Use Makefile Commands

```bash
# Create and setup production database
make prod-db-create
make prod-db-setup

# Or for development (includes seed data)
make dev-db-setup
```

## Configuration

### 1. Copy the Example Configuration

```bash
cp config/config.toml.example config/config.toml
```

### 2. Edit the Configuration File

Open `config/config.toml` and update the following sections:

#### Database Connections
```toml
[[databases]]
id = "primary"
label = "Main Server"
url = "mysql://sortingoffice:your_password@localhost:3306/sortingoffice"
```

#### Admin Users
```toml
[[admins]]
username = "admin"
# Generate a new password hash using the provided script
password_hash = "$2a$12$..."  # Use scripts/generate_password_hash.sh
role = "edit"
```

#### Production Safety Settings
```toml
[global_features]
read_only = false
no_new_users = false
no_new_domains = false
no_password_updates = false

[databases.features]
read_only = false
no_seeding = true      # Prevent seeding in production
no_migrations = true   # Prevent migrations in production
```

### 3. Generate Password Hash

```bash
# Use the provided script
./scripts/generate_password_hash.sh "your_admin_password"
```

### 4. Environment Variables (Optional)

Create a `.env` file for additional configuration:

```bash
cp env.example .env
```

Edit `.env` to set:
- `ENVIRONMENT=production` (for production safety)
- `RUST_LOG=info` (for logging)
- Database connection details

## Schema Preparation

### Check Required Fields

The application expects certain fields in your database tables. If your existing database doesn't have these fields, you'll need to add them:

#### Created and Modified Columns
```sql
ALTER TABLE users 
ADD COLUMN created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
ADD COLUMN modified DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP;
```

Apply the same to the other relevant tables: `domains`, `backups`, `aliases`, `relays`, `clients` and `relocated` if present.

> **Note for MySQL 5.5 and earlier**: If you're using MySQL 5.5 or earlier, the `CURRENT_TIMESTAMP` default values won't work. Use `DATETIME` with NULL defaults instead:
> ```sql
> ALTER TABLE backups 
> ADD COLUMN created DATETIME NULL,
> ADD COLUMN modified DATETIME NULL;
> ```
> Apply the same pattern to all other tables as above. The application will automatically set these timestamps when creating and updating records.


### Run Migrations (Development Only)

```bash
# Run migrations
make migrate

# Or for all databases
make migrate-all
```

**⚠️ Important**: Migrations are automatically blocked in production for safety.

## Production Safety

The application includes several safety features to prevent accidental data loss:

### Automatic Protections

The following operations are automatically blocked in production:
- **Seeding**: Adding test/initial data
- **Migrations**: Database schema changes
- **Database resets**: Clearing all data

### Environment Detection

Set `ENVIRONMENT=production` to enable all safety features:
```bash
export ENVIRONMENT=production
```

### Force Commands (Use with Caution)

If you need to bypass protections:
```bash
# Force seeding
make seed-force

# Force migrations
make migrate-force

# Force database reset
make migrate-reset-force
```

## Starting the Application

### Option 1: Direct Binary (Development)

```bash
# Check compilation first
cargo check

# Run the application
cargo run

# Or run in release mode
cargo run --release
```

### Option 2: Docker Compose (Recommended for Production)

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f sortingoffice
```

### Option 3: Docker Run

```bash
docker run -d \
  --name sortingoffice \
  -p 3000:3000 \
  -v $(pwd)/config:/app/config \
  -e ENVIRONMENT=production \
  sortingoffice:latest
```

### Option 4: Systemd Service (Production)

Create `/etc/systemd/system/sortingoffice.service`:
```ini
[Unit]
Description=Sorting Office
After=network.target mysql.service

[Service]
Type=simple
User=sortingoffice
WorkingDirectory=/opt/sortingoffice
ExecStart=/opt/sortingoffice/sortingoffice
Restart=always
Environment=ENVIRONMENT=production

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable sortingoffice
sudo systemctl start sortingoffice
```

## First Login

1. **Access the application**: Open your browser to `http://localhost:3000`

2. **Login with admin credentials**:
   - Username: `admin` (or as configured in config.toml)
   - Password: The password you used to generate the hash

3. **Verify functionality**:
   - Check the dashboard loads
   - Navigate to different sections
   - Verify database connections are working

## Troubleshooting

### Common Issues

#### Database Connection Errors
```bash
# Check database connectivity
make list-databases

# Test connection
mysql -h localhost -u sortingoffice -p sortingoffice
```

#### Configuration Errors
```bash
# Validate configuration
cargo check

# Check configuration loading
RUST_LOG=debug cargo run
```

#### Permission Issues
```bash
# Ensure proper file permissions
chmod 644 config/config.toml
chmod 755 sortingoffice
```

#### Port Already in Use
```bash
# Check what's using port 3000
lsof -i :3000

# Kill existing process
pkill -f sortingoffice
```

### Logs and Debugging

#### Enable Debug Logging
```bash
export RUST_LOG=debug
cargo run
```

#### Docker Logs
```bash
# View application logs
docker-compose logs sortingoffice

# View database logs
docker-compose logs mysql
```

#### Systemd Logs
```bash
# View service logs
sudo journalctl -u sortingoffice -f
```

### Getting Help

1. **Check the documentation**: Review other docs in the `/docs` directory
2. **Run tests**: Ensure everything is working with `make test-unit`
3. **Check issues**: Look for similar problems in the GitHub issues
4. **Create issue**: If the problem persists, create a detailed issue report

## Next Steps

After successful onboarding:

1. **Review the configuration**: Ensure all settings match your requirements
2. **Set up monitoring**: Consider adding health checks and monitoring
3. **Backup strategy**: Implement regular database backups (see [DATABASE_BACKUP.md](DATABASE_BACKUP.md))
4. **Security review**: Ensure proper firewall and access controls
5. **Documentation**: Update this guide with any environment-specific notes

## Security Considerations

- **Change default passwords**: Always change the default admin password
- **Use HTTPS**: Set up SSL/TLS for production deployments
- **Firewall**: Restrict access to the application port
- **Database security**: Use strong passwords and limit database access
- **Regular updates**: Keep the application and dependencies updated

---

For additional help, see the [main README](../README.md) or other documentation in the `/docs` directory. 
