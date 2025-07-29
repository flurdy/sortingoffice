# Contributing to Sorting Office

Thank you for your interest in contributing to Sorting Office! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Code Style](#code-style)
- [Internationalization](#internationalization)
- [Database Changes](#database-changes)
- [Documentation](#documentation)

## Code of Conduct

This project is committed to providing a welcoming and inclusive environment for all contributors. Please be respectful and considerate in all interactions.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/sortingoffice.git
   cd sortingoffice
   ```
3. **Add the upstream remote**:
   ```bash
   git remote add upstream https://github.com/flurdy/sortingoffice.git
   ```

## Development Setup

### Prerequisites

- Rust (latest stable version)
- Docker and Docker Compose
- MySQL (for local development)

### Quick Start

1. **Copy environment file**:
   ```bash
   cp env.example env.local
   ```

2. **Start the development environment**:
   ```bash
   make dev-db-setup
   make run
   ```

3. **Access the application** at `http://localhost:3000`

### Alternative Setup with Docker

```bash
# Start the full stack
docker-compose -f docker-compose.dev.yml up -d

# Run the application
cargo run
```

## Testing

### Running Tests

```bash
# Run all tests
make test-all

# Run specific test suites
make test-unit      # Unit tests
make test-integration  # Integration tests
make test-ui        # UI tests (headless)
make test-ui-containerized  # UI tests (with browser)
```

### Test Structure

- **Unit tests**: Located in `src/` files with `#[cfg(test)]` modules
- **Integration tests**: Located in `tests/integration.rs`
- **UI tests**: Located in `tests/ui_containerized.rs` and `tests/ui_smoke.rs`
- **Handler tests**: Located in `tests/handlers.rs`

### Writing Tests

- Follow the existing test patterns
- Use `TestUtils` helpers for common operations
- Use `TestData` for generating unique test data
- Clean up test data after each test

## Submitting Changes

### Before Submitting

1. **Run all tests** to ensure nothing is broken:
   ```bash
   make test-all
   ```

2. **Check code formatting**:
   ```bash
   cargo fmt
   ```

3. **Run linter checks**:
   ```bash
   cargo clippy
   ```

### Pull Request Process

1. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** and commit them:
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

3. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

4. **Create a Pull Request** on GitHub

### Commit Message Format

We follow the [Conventional Commits](https://www.conventionalcommits.org/) format:

- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `style:` Code style changes (formatting, etc.)
- `refactor:` Code refactoring
- `test:` Adding or updating tests
- `chore:` Maintenance tasks

## Code Style

### Rust

- Follow Rust conventions and idioms
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Prefer explicit types over type inference when it improves readability

### Templates (Askama)

- Use consistent indentation (2 spaces)
- Keep templates readable and well-structured
- Use meaningful variable names
- Add comments for complex logic

### CSS/Tailwind

- Use Tailwind CSS classes
- Follow responsive design principles
- Maintain dark mode compatibility
- Use consistent spacing and colors

## Internationalization

### Adding New Translations

1. **Add translation keys** to all language files in `resources/locales/`:
   - `en-US/messages.ftl`
   - `es-ES/messages.ftl`
   - `de-DE/messages.ftl`
   - `fr-FR/messages.ftl`
   - `nb-NO/messages.ftl`

2. **Update template structs** to include new translation fields

3. **Update handlers** to fetch and pass new translations

### Translation Guidelines

- Use descriptive key names (e.g., `domains-table-header-domain`)
- Keep translations concise but clear
- Consider cultural differences in translations
- Test with different languages

## Database Changes

### Migrations

1. **Create a new migration**:
   ```bash
   diesel migration generate your_migration_name
   ```

2. **Write the migration** in the generated files:
   - `up.sql` for changes
   - `down.sql` for rollback

3. **Test the migration**:
   ```bash
   make migrate
   make migrate-revert
   make migrate
   ```

### Schema Changes

- Update `src/schema.rs` after running migrations
- Update models in `src/models.rs` if needed
- Add appropriate database functions in `src/db.rs`

## Documentation

### Code Documentation

- Document public functions and structs
- Use `///` for documentation comments
- Include examples for complex functions

### User Documentation

- Update `README.md` for user-facing changes
- Add documentation in `docs/` for technical details
- Keep `todos.md` updated with completed tasks

## Getting Help

- **Issues**: Use GitHub Issues for bug reports and feature requests
- **Discussions**: Use GitHub Discussions for questions and general discussion
- **Code Review**: All changes go through pull request review

## Recognition

Contributors will be recognized in:
- The project's README.md
- Release notes
- GitHub contributors list

Thank you for contributing to Sorting Office! 
