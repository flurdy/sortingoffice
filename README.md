# Sorting Office

A modern web-based admin tool for managing mail server data based on [flurdy's "How to set up a mail server on a GNU / Linux system"](https://flurdy.com/docs/postfix/).
## Versions
- v3: The current Rust based setup
- v1: The original Play Framework with Scala based setup,
   refer to the [git tag *v1*](https://github.com/flurdy/sortingoffice/tree/v1)
 
## Quick Start

The easiest way to run Sorting Office is using Docker:

```bash
# Clone and start
git clone <repository-url>
cd sortingoffice
chmod +x docker.sh
./docker.sh build
./docker.sh up
```

Access at http://localhost:3000 (admin/admin)

For detailed installation instructions, see [ONBOARDING.md](docs/ONBOARDING.md).

## Features

- **Domain Management**: Add, edit, and remove mail domains with quota and transport settings
- **User Management**: Manage mail users with password hashing and quota allocation
- **Alias Management**: Create and manage email aliases for forwarding
- **Statistics Dashboard**: View system-wide and per-domain statistics
- **Modern UI**: Built with Tailwind CSS for a clean, responsive interface
- **Dark Mode Support**: Toggle between light and dark themes
- **HTMX Integration**: Dynamic updates without full page reloads
- **Secure Authentication**: Role-based access control with multiple admin support

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

For detailed setup instructions, see [ONBOARDING.md](docs/ONBOARDING.md).

## Development

### Quick Commands

```bash
# Run tests
make test

# Run UI tests
make test-ui

# Database management
make db-help

# Health checks
./scripts/health-check.sh
```

### Project Structure

- `src/` - Rust application code
- `templates/` - HTML templates
- `migrations/` - Database migrations
- `docs/` - Detailed documentation
- `tests/` - Test suites

For detailed development information, see [TEST_ORGANIZATION.md](docs/TEST_ORGANIZATION.md).

## Documentation

- [ONBOARDING.md](docs/ONBOARDING.md) - Installation and setup
- [AUTHENTICATION.md](docs/AUTHENTICATION.md) - Authentication and security
- [DATABASE_MANAGEMENT.md](docs/DATABASE_MANAGEMENT.md) - Database operations
- [UI_TESTS.md](docs/UI_TESTS.md) - Testing information
- [CONTACT.md](docs/CONTACT.md) - Support and contact

## Security

- Role-based authentication with multiple admin support
- bcrypt password hashing
- SQL injection protection via Diesel ORM
- Comprehensive security headers
- HTTPS ready

For detailed security information, see [AUTHENTICATION.md](docs/AUTHENTICATION.md).

## Production Deployment

1. Set production environment variables
2. Use production MySQL/MariaDB instance
3. Configure reverse proxy (nginx/Apache)
4. Enable HTTPS with proper certificates
5. Set up regular database backups

For detailed deployment information, see [DATABASE_BACKUP.md](docs/DATABASE_BACKUP.md).

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

For detailed contribution guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md) and [CONTACT.md](docs/CONTACT.md).

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Based on [flurdy's Postfix mail server guide](https://flurdy.com/docs/postfix/)
- Built with modern Rust web development tools
- UI inspired by modern admin dashboard designs
