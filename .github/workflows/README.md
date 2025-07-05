# GitHub Actions Workflows

This directory contains GitHub Actions workflows for the Sorting Office project. These workflows automate building, testing, and deploying the application.

## Workflows Overview

### 1. CI (`ci.yml`)
**Triggers:** Push to `main`/`develop`, Pull Requests
**Purpose:** Continuous Integration pipeline

**Jobs:**
- **Test and Lint**: Runs unit tests, code formatting checks, and clippy linting
- **Security Audit**: Performs security vulnerability scanning with `cargo audit`
- **Docker Build**: Tests Docker image building

**Features:**
- MySQL database service for testing
- Rust dependency caching
- Code quality checks (fmt, clippy)
- Security scanning
- Build artifact upload

### 2. UI Tests (`ui-tests.yml`)
**Triggers:** Push to `main`/`develop`, Pull Requests, Manual dispatch
**Purpose:** End-to-end UI testing with Selenium

**Jobs:**
- **UI Tests with Selenium**: Runs comprehensive UI tests using Selenium WebDriver

**Features:**
- MySQL database service
- Selenium WebDriver service (Chrome)
- Application startup and health checks
- Basic and advanced UI test suites
- Screenshot capture on failure

### 3. Release (`release.yml`)
**Triggers:** Release published
**Purpose:** Automated release builds and Docker image publishing

**Jobs:**
- **Build and Push Docker Image**: Creates and publishes Docker images to GitHub Container Registry

**Features:**
- Automatic Docker image tagging
- Release artifact creation
- GitHub Container Registry integration
- Semantic versioning support

### 4. Nightly Build (`nightly.yml`)
**Triggers:** Daily at 2 AM UTC, Manual dispatch
**Purpose:** Comprehensive nightly testing and dependency monitoring

**Jobs:**
- **Nightly Comprehensive Test**: Full test suite with code coverage
- **Check for Dependency Updates**: Monitors for outdated dependencies

**Features:**
- Code coverage reporting
- Dependency update checking
- Extended testing scenarios
- Long-term artifact retention



## Usage

### Automatic Triggers
- **Push to main/develop**: Triggers CI and UI tests
- **Pull Requests**: Triggers CI and UI tests
- **Release creation**: Triggers release workflow
- **Daily schedule**: Triggers nightly build

### Manual Triggers
- **UI Tests**: Can be manually triggered via GitHub Actions UI
- **Nightly Build**: Can be manually triggered via GitHub Actions UI

## Environment Variables

### Required Secrets
- `GITHUB_TOKEN`: Automatically provided by GitHub

### Environment Variables
- `DATABASE_URL`: MySQL connection string for tests
- `RUST_LOG`: Logging level (debug/info)
- `RUST_BACKTRACE`: Backtrace generation

## Customization

### Adding New Tests
1. Add test files to `tests/` directory
2. Update `ci.yml` to include new test commands
3. Consider adding to UI tests if they require browser interaction



### Adding New Workflows
1. Create new `.yml` file in `.github/workflows/`
2. Follow existing patterns for consistency
3. Update this README with documentation

## Troubleshooting

### Common Issues
1. **MySQL connection failures**: Check service health checks and connection strings
2. **Selenium timeouts**: Verify Selenium service is running and accessible
3. **Docker build failures**: Check Dockerfile syntax and dependencies
4. **Test failures**: Review test logs and ensure all dependencies are available

### Debugging
- Enable debug logging with `RUST_LOG=debug`
- Check workflow logs in GitHub Actions UI
- Use manual triggers for testing workflow changes
- Review artifact uploads for additional debugging information

## Best Practices

1. **Keep workflows fast**: Use caching and parallel jobs where possible
2. **Fail fast**: Put critical checks early in the pipeline
3. **Use specific versions**: Pin action versions for stability
4. **Test locally**: Use `act` or similar tools to test workflows locally
5. **Monitor regularly**: Check workflow success rates and performance 
