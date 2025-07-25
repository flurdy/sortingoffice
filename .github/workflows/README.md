# GitHub Actions Workflows

This directory contains GitHub Actions workflows for the Sorting Office project. These workflows automate building, testing, and deploying the application.

## Workflows Overview

### 1. CI (`ci.yml`)
**Triggers:** Push to `master`, Pull Requests
**Purpose:** Continuous Integration pipeline

**Jobs:**
- **Test and Lint**: Runs unit tests, integration tests, code formatting checks, and clippy linting
- **Security Audit**: Performs security vulnerability scanning with `cargo audit`
- **Docker Build**: Tests Docker image building

**Features:**
- Uses testcontainers for isolated database testing
- Rust dependency caching
- Code quality checks (fmt, clippy)
- Security scanning
- Build artifact upload

### 2. UI Tests (`ui-tests.yml`)
**Triggers:** Push to `master`, Pull Requests, Manual dispatch
**Purpose:** End-to-end UI testing with containerized environment

**Jobs:**
- **UI Tests (Containerized)**: Runs comprehensive UI tests using testcontainers

**Features:**
- Uses testcontainers for isolated database and application testing
- Docker-based test environment
- Application startup and health checks
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
- **Nightly Comprehensive Test**: Full test suite including unit, integration, and UI tests
- **Check for Dependency Updates**: Monitors for outdated dependencies

**Features:**
- Uses testcontainers for isolated testing
- Dependency update checking
- Extended testing scenarios
- Long-term artifact retention

## Usage

### Automatic Triggers
- **Push to master**: Triggers CI and UI tests
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
- `RUST_LOG`: Logging level (debug/info)
- `RUST_BACKTRACE`: Backtrace generation
- `RUST_TEST_THREADS`: Number of test threads (default: 2 for integration tests)

## Test Structure

### Test Types
- **Unit Tests**: Located in `src/` files, run with `make test-unit`
- **Integration Tests**: Located in `tests/` directory, run with `make test-integration`
- **UI Tests**: Containerized tests using testcontainers, run with `make test-ui`

### Test Isolation
All tests use testcontainers to ensure:
- Isolated database instances per test
- No interference between test runs
- Consistent test environment
- No external database dependencies

## Customization

### Adding New Tests
1. Add unit tests to `src/` files
2. Add integration tests to `tests/` directory
3. Add UI tests to `tests/ui_containerized.rs` or `tests/ui_smoke.rs`
4. Update `Makefile` if new test commands are needed

### Adding New Workflows
1. Create new `.yml` file in `.github/workflows/`
2. Follow existing patterns for consistency
3. Use testcontainers for any database-dependent tests
4. Update this README with documentation

## Troubleshooting

### Common Issues
1. **Testcontainers failures**: Check Docker daemon and container resource limits
2. **UI test timeouts**: Verify testcontainers are starting correctly
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
6. **Use testcontainers**: Ensure test isolation and reproducibility 
