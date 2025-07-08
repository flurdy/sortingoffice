# Test Database Setup with Testcontainers

This document describes the test database setup for Sorting Office using Testcontainers, which provides isolated, real MySQL instances for each test.

## Overview

The project uses [Testcontainers](https://testcontainers.com/) to spin up isolated MySQL containers for unit tests. This approach provides:

- **True isolation**: Each test gets its own MySQL instance
- **Real database**: Tests run against actual MySQL, not mocks
- **Automatic cleanup**: Containers are automatically removed after tests
- **No external dependencies**: No need for a running MySQL server on the host

## Architecture

### Test Container Structure

```rust
pub struct TestContainer {
    pub pool: DbPool,                    // Diesel connection pool
    _docker: &'static Cli,              // Docker client (static reference)
    _container: Container<'static, Mysql>, // MySQL container
}
```

### Key Components

1. **Testcontainers Setup** (`src/tests/testcontainers_setup.rs`)
   - Manages Docker container lifecycle
   - Creates connection pools
   - Runs database migrations

2. **Test Module** (`src/tests/testcontainers_test.rs`)
   - Example tests demonstrating the setup
   - Shows isolation between tests

3. **Dependencies** (in `Cargo.toml`)
   ```toml
   [dev-dependencies]
   testcontainers = "0.15"
   testcontainers-modules = { version = "0.3", features = ["mysql"] }
   ```

## How It Works

### 1. Container Creation

When a test starts:

```rust
let container = setup_test_db();
```

The system:
1. Creates a Docker client (`Cli::default()`)
2. Spins up a MySQL 8.1 container with default settings
3. Waits for the container to be ready
4. Gets the mapped port (e.g., `32774`)
5. Creates a connection pool to `mysql://root@127.0.0.1:32774/mysql`

### 2. Database Connection

```rust
let pool = container.get_pool();
let mut conn = pool.get().expect("Failed to get connection");
```

The connection pool connects to the isolated MySQL container using:
- **Host**: `127.0.0.1` (Docker host)
- **Port**: Dynamically assigned (e.g., `32774`)
- **User**: `root` (no password)
- **Database**: `mysql` (default)

### 3. Migration Execution

Before tests run, the system automatically:
1. Connects to the container
2. Runs all pending Diesel migrations
3. Sets up the schema (domains, users, aliases, backups tables)

### 4. Test Isolation

Each test gets its own container:

```rust
#[test]
fn test_isolation() {
    let container1 = setup_test_db(); // Container A
    let container2 = setup_test_db(); // Container B (different instance)
    
    // Data in container1 is isolated from container2
}
```

### 5. Automatic Cleanup

When the `TestContainer` goes out of scope:
1. The connection pool is dropped
2. The Docker container is automatically stopped and removed
3. All test data is cleaned up

## Usage Examples

### Basic Test

```rust
#[test]
fn test_basic_connection() {
    let container = setup_test_db();
    let pool = container.get_pool();
    
    let mut conn = pool.get().expect("Failed to get connection");
    let result: TestResult = diesel::sql_query("SELECT 1 as result")
        .get_result(&mut conn)
        .expect("Failed to execute query");
    
    assert_eq!(result.result, 1);
    
    cleanup_test_db(&container);
}
```

### Database Operations Test

```rust
#[test]
fn test_domain_operations() {
    let container = setup_test_db();
    let pool = container.get_pool();
    
    // Test domain creation
    let new_domain = DomainForm {
        domain: "test.com".to_string(),
        transport: "virtual".to_string(),
        enabled: true,
    };
    
    let domain = crate::db::create_domain(pool, new_domain).unwrap();
    assert_eq!(domain.domain, "test.com");
    
    cleanup_test_db(&container);
}
```

### Isolation Test

```rust
#[test]
fn test_isolation() {
    let container1 = setup_test_db();
    let pool1 = container1.get_pool();
    
    let container2 = setup_test_db();
    let pool2 = container2.get_pool();
    
    // Insert data in first container
    let mut conn1 = pool1.get().expect("Failed to get connection");
    diesel::sql_query("INSERT INTO domains (domain, transport, enabled, created, modified) VALUES ('test1.com', 'virtual', 1, NOW(), NOW())")
        .execute(&mut conn1)
        .expect("Failed to insert test data");
    
    // Check data exists in first container
    let count1: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM domains")
        .get_result(&mut conn1)
        .expect("Failed to count domains");
    assert_eq!(count1.count, 1);
    
    // Check data does NOT exist in second container
    let mut conn2 = pool2.get().expect("Failed to get connection");
    let count2: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM domains")
        .get_result(&mut conn2)
        .expect("Failed to count domains");
    assert_eq!(count2.count, 0);
    
    cleanup_test_db(&container1);
    cleanup_test_db(&container2);
}
```

## Configuration

### Environment Variables

The test setup uses these environment variables:

- `TEST_DATABASE_URL`: Override the default connection string
- `RUST_LOG`: Set to "debug" for detailed logging

### Docker Requirements

- Docker must be running and accessible
- The `docker` command must be available in PATH
- Sufficient resources for MySQL containers

### Container Settings

The MySQL container uses these default settings:
- **Image**: `mysql:8.1`
- **Database**: `test` (created automatically)
- **User**: `root` (no password)
- **Port**: `3306` (mapped to random host port)

## Migration from Old Test Setup

### Before (Shared MySQL Database)

```rust
// Old approach - shared database
pub fn setup_test_db() -> DbPool {
    // Connected to shared MySQL instance
    // Tests could interfere with each other
    // Required external MySQL server
}
```

### After (Testcontainers)

```rust
// New approach - isolated containers
pub fn setup_test_db() -> TestContainer {
    // Each test gets its own MySQL container
    // Complete isolation between tests
    // No external dependencies
}
```

## Benefits

### 1. True Isolation
- Each test runs in its own MySQL instance
- No shared state between tests
- Tests can't interfere with each other

### 2. Real Database Testing
- Tests run against actual MySQL
- Catches real database issues
- Tests actual SQL queries and migrations

### 3. No External Dependencies
- No need for MySQL server on host
- Works in CI/CD environments
- Consistent across different machines

### 4. Automatic Cleanup
- Containers are automatically removed
- No leftover test data
- Clean test environment

### 5. Fast Setup
- Containers start quickly
- No manual database setup required
- Parallel test execution possible

## Troubleshooting

### Common Issues

#### 1. Docker Not Running
```
Error: Failed to create pool: Connection refused
```
**Solution**: Ensure Docker is running and accessible.

#### 2. Port Conflicts
```
Error: Port already in use
```
**Solution**: Testcontainers automatically assigns random ports, so this is rare.

#### 3. Container Startup Timeout
```
Error: Container failed to start within timeout
```
**Solution**: 
- Check Docker resources (memory, CPU)
- Ensure MySQL image can be pulled
- Check network connectivity

#### 4. Migration Failures
```
Error: Failed to run migrations
```
**Solution**:
- Check migration files are up to date
- Verify MySQL version compatibility
- Check connection string format

### Debug Mode

Enable detailed logging:

```bash
RUST_LOG=debug cargo test testcontainers_test
```

This shows:
- Docker commands being executed
- Container startup process
- Connection attempts
- Migration execution

### Performance Considerations

- **Container startup**: ~10-15 seconds per test
- **Memory usage**: ~200MB per MySQL container
- **Disk usage**: Minimal (containers are ephemeral)

For faster feedback during development, consider:
- Running fewer tests in parallel
- Using shared containers for related tests
- Caching Docker images

## Integration with CI/CD

The testcontainers setup works seamlessly in CI/CD environments:

### GitHub Actions Example

```yaml
- name: Run tests with testcontainers
  run: cargo test
  env:
    RUST_LOG: info
```

### Docker-in-Docker

For CI environments that need Docker-in-Docker:

```yaml
services:
  docker:
    image: docker:dind
    privileged: true
```

## Best Practices

### 1. Test Structure
- Keep tests focused and isolated
- Clean up resources explicitly when needed
- Use descriptive test names

### 2. Database Operations
- Test actual database operations
- Verify data persistence
- Test error conditions

### 3. Performance
- Don't create unnecessary containers
- Reuse containers when possible
- Monitor resource usage

### 4. Debugging
- Use debug logging for troubleshooting
- Check container logs when needed
- Verify network connectivity

## Future Enhancements

### Potential Improvements

1. **Container Reuse**: Share containers between related tests
2. **Custom Images**: Use optimized MySQL images for faster startup
3. **Data Seeding**: Pre-populate containers with test data
4. **Parallel Testing**: Optimize for concurrent test execution

### Alternative Approaches

1. **Mock Database**: For pure logic testing
2. **Shared Test Database**: For integration tests

## Conclusion

The testcontainers setup provides a robust, isolated testing environment that ensures tests are reliable and don't interfere with each other. It's particularly valuable for:

- **Unit tests**: Isolated database operations
- **Integration tests**: End-to-end database workflows
- **CI/CD pipelines**: Consistent test environments

This approach significantly improves test reliability and reduces flaky tests caused by shared database state. 
