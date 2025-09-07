# Test Troubleshooting Guide

This guide covers common issues encountered when running tests and their solutions.

## 🚨 Critical Issues

### Database Connection Failures

**Symptoms:**

```
ERROR r2d2: Unknown database 'test_...'
DatabaseError(Unknown, "Unknown database 'test_...'")
```

**Causes:**

- Test database container failed to start
- Database schema creation failed
- Container cleanup happened too early
- Port conflicts with existing MySQL instances

**Solutions:**

1. **Check Docker status:**

   ```bash
   docker ps -a | grep mysql
   docker logs <container_id>
   ```

2. **Clean up orphaned containers:**

   ```bash
   make test-clean
   # or manually:
   docker ps -a | grep test | awk '{print $1}' | xargs -r docker rm -f
   ```

3. **Check port conflicts:**

   ```bash
   netstat -tlnp | grep 3306
   lsof -i :3306
   ```

4. **Verify testcontainers setup:**
   ```bash
   cargo test --test testcontainers_test
   ```

### Test Timeout Issues

**Symptoms:**

- Tests hang indefinitely
- CI builds fail with timeout errors
- Selenium container startup failures

**Solutions:**

1. **Increase timeouts in CI:**

   ```yaml
   # .github/workflows/ci.yml
   timeout-minutes: 30 # Increased from 20
   ```

2. **Reduce test parallelism in CI:**

   ```bash
   export CI=true
   export RUST_TEST_THREADS=1
   export TEST_THREADS=2
   ```

3. **Check system resources:**
   ```bash
   docker system df
   docker system prune -f
   ```

## 🔧 Common Issues

### Shell Command Dependencies

**Issue:** Hardcoded paths in mysqldump calls
**Solution:** ✅ **FIXED** - Implemented portable mysqldump detection

**Issue:** Linux-specific `ip` commands in smoke tests
**Solution:** ✅ **FIXED** - Cross-platform IP detection with environment variables

**Environment Variables for IP Detection:**

```bash
export HOST_IP=192.168.1.100
export DOCKER_HOST_IP=172.17.0.1
export HOST_BRIDGE_IP=172.18.0.1
export GATEWAY_IP=192.168.1.1
```

### Container Cleanup Issues

**Symptoms:**

- Containers accumulate over time
- Port conflicts between test runs
- Resource exhaustion

**Solutions:**

1. **Use comprehensive cleanup:**

   ```bash
   make test-clean
   ```

2. **Clean individual test resources:**

   ```bash
   make test-single TEST=test_name
   make test-single-ui TEST=test_name
   ```

3. **Manual cleanup:**
   ```bash
   docker ps -a | grep test | awk '{print $1}' | xargs -r docker rm -f
   docker network rm sortingoffice-e2e 2>/dev/null || true
   ```

### Selenium Container Issues

**Symptoms:**

- WebDriver connection failures
- Chrome crashes
- VNC connection issues

**Solutions:**

1. **Check Selenium container logs:**

   ```bash
   docker logs <selenium_container_id>
   ```

2. **Verify Chrome arguments:**

   ```rust
   // CI-optimized Chrome arguments
   "--disable-extensions",
   "--disable-plugins",
   "--disable-images",
   "--disable-javascript",
   "--no-sandbox",
   "--disable-dev-shm-usage"
   ```

3. **Check shared network setup:**
   ```bash
   docker network ls | grep sortingoffice
   ```

## 🐛 Test-Specific Issues

### Handler Tests

**Issue:** Database schema not found after creation
**Solution:** Ensure proper test isolation and cleanup order

**Issue:** Authentication failures in tests
**Solution:** Use proper test user setup and session management

### Integration Tests

**Issue:** Database connection pool exhaustion
**Solution:** Implement proper connection cleanup and pool management

**Issue:** Test data conflicts
**Solution:** Use unique identifiers and proper cleanup

### UI Tests

**Issue:** Element not found errors
**Solution:** Add proper wait conditions and element verification

**Issue:** Cross-platform compatibility
**Solution:** ✅ **FIXED** - Implemented cross-platform IP detection

## 🚀 Performance Optimization

### CI Environment

**Optimizations Applied:**

- Reduced test parallelism: 8 → 2 threads
- Single-threaded UI tests
- Faster container timeouts: 300s → 180s
- Disabled unnecessary Chrome features

**Environment Variables:**

```bash
export CI=true
export RUST_TEST_THREADS=1
export TEST_THREADS=2
```

### Local Development

**Recommended Settings:**

```bash
export RUST_TEST_THREADS=8
export TEST_THREADS=8
export RUST_LOG=info
```

## 📋 Debugging Commands

### Container Inspection

```bash
# List all containers
docker ps -a

# Check container logs
docker logs <container_id>

# Inspect container details
docker inspect <container_id>

# Check container resources
docker stats <container_id>
```

### Network Debugging

```bash
# List networks
docker network ls

# Inspect network
docker network inspect sortingoffice-e2e

# Check network connectivity
docker exec <container_id> ping <target_ip>
```

### Test Debugging

```bash
# Run single test with output
cargo test --test handlers -- tests::test_name --nocapture

# Run with specific log level
RUST_LOG=debug cargo test

# Run with backtrace
RUST_BACKTRACE=1 cargo test
```

## 🔍 Common Error Messages

### Database Errors

```
DatabaseError(Unknown, "Unknown database 'test_...'")
→ Check container startup and schema creation

DatabaseError(ConnectionFailed, "...")
→ Check network connectivity and port availability

DatabaseError(QueryFailed, "...")
→ Check SQL syntax and table existence
```

### Container Errors

```
No such container: ...
→ Container was removed before test completion

Port already in use
→ Check for port conflicts and cleanup

Network not found
→ Verify shared network creation
```

### Selenium Errors

```
WebDriver connection failed
→ Check Selenium container health

Element not found
→ Add proper wait conditions

Chrome crashed
→ Check Chrome arguments and resource limits
```

## 📞 Getting Help

### Check Logs

1. **Application logs:** `RUST_LOG=debug cargo test`
2. **Container logs:** `docker logs <container_id>`
3. **Test output:** Use `--nocapture` flag

### Common Solutions

1. **Clean up resources:** `make test-clean`
2. **Restart Docker:** `sudo systemctl restart docker`
3. **Check system resources:** `docker system df`
4. **Verify configuration:** Check environment variables

### Reporting Issues

When reporting test issues, include:

- Test command used
- Full error output
- System information (OS, Docker version)
- Container logs
- Environment variables set

## ✅ Recent Fixes

### Shell Command Dependencies

- ✅ **mysqldump hardcoded paths** - Now portable across systems
- ✅ **Linux-specific `ip` commands** - Cross-platform detection implemented
- ✅ **curl in shell scripts** - Kept as acceptable requirement

### Test Infrastructure

- ✅ **CI timeouts** - Increased and optimized
- ✅ **Test parallelism** - CI-specific optimizations
- ✅ **Container cleanup** - Comprehensive cleanup implemented

### Code Quality

- ✅ **Formatting** - Applied `cargo fmt`
- ✅ **Linting** - Fixed 45 clippy warnings
- ✅ **Cross-platform** - Improved compatibility
