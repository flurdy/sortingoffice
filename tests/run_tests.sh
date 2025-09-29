#!/bin/bash

# Test runner script for sortingoffice
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
show_usage() {
    echo "🧪 SortingOffice Test Runner"
    echo ""
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  unit              Run only unit tests (tests in source files)"
    echo "  integration       Run only integration tests (tests in tests/ directory)"
    echo "  security          Run security tests (SQL injection, auth bypass, etc.)"
    echo "  api               Run API tests (authentication, authorization, etc.)"
    echo "  ui                Run containerized UI tests (app + db in containers)"
    echo "  smoke             Run end-to-end smoke test against running app"
    echo "  smoke-containerized Run end-to-end smoke test with testcontainers"
    echo "  all               Run all tests (unit + integration + security + api + UI)"
    echo "  single TEST_NAME  Run individual test with cleanup"
    echo "  single-ui TEST_NAME Run individual UI test with cleanup"
    echo "  cleanup           Clean up test resources only"
    echo "  help              Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                # Run unit tests"
    echo "  $0 unit           # Run unit tests"
    echo "  $0 integration    # Run integration tests"
    echo "  $0 security       # Run security tests"
    echo "  $0 api            # Run API tests"
    echo "  $0 ui             # Run containerized UI tests"
    echo "  $0 smoke          # Run end-to-end smoke test"
    echo "  $0 smoke-containerized # Run end-to-end smoke test with testcontainers"
    echo "  $0 all            # Run all tests"
    echo "  $0 single test_homepage_loads_containerized # Run individual test"
    echo "  $0 single-ui test_homepage_loads_containerized # Run individual UI test"
    echo "  $0 cleanup        # Clean up test resources"
}

# Function to run unit tests
run_unit_tests() {
    print_status "Running unit tests for sortingoffice..."

    # Set test environment
    export RUST_LOG=error
    export RUST_BACKTRACE=1
    export TESTCONTAINERS_LOG_LEVEL=error
    export BOLLARD_LOG_LEVEL=error

    # Reduce test threads for CI environment to prevent resource contention
    if [ "$CI" = "true" ]; then
        export RUST_TEST_THREADS=2
        print_warning "CI environment detected, reducing unit test threads to 2"
    else
        export RUST_TEST_THREADS=1
    fi

    # Run only the unit tests (tests in source files)
    print_status "Running unit tests with cargo..."
    start_time=$(date +%s)
    if cargo test --lib; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_success "Unit tests completed successfully in ${duration}s!"
        echo "[CI-SUMMARY] UNIT PASS ${duration}s"
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_error "Unit tests failed in ${duration}s!"
        echo "[CI-SUMMARY] UNIT FAIL ${duration}s"
        exit 1
    fi
}

# Function to run integration tests
run_integration_tests() {
    print_status "Running integration tests for sortingoffice..."

    # Check if DATABASE_URL is set, if not use default test database
    if [ -z "$DATABASE_URL" ]; then
        export DATABASE_URL="mysql://root:password@localhost/sortingoffice_test"
        print_warning "DATABASE_URL not set, using default test database: $DATABASE_URL"
    fi

    # Set test environment
    export RUST_LOG=error
    export RUST_BACKTRACE=0
    export TESTCONTAINERS_LOG_LEVEL=error
    export BOLLARD_LOG_LEVEL=error

    # Reduce test threads for CI environment to prevent resource contention
    if [ "$CI" = "true" ]; then
        # Honor externally provided settings; default to 1 if unset
        : "${TEST_THREADS:=1}"
        : "${RUST_TEST_THREADS:=1}"
        print_warning "CI environment detected, using test threads: $TEST_THREADS"
    else
        : "${TEST_THREADS:=8}"
        : "${RUST_TEST_THREADS:=${TEST_THREADS}}"
    fi

    # Run only the integration tests (excluding UI tests)
    print_status "Running integration tests with cargo (threads: $TEST_THREADS)..."
    start_time=$(date +%s)
    if cargo test --test integration --test handlers --test testcontainers_test --test cross_database_domain_tests -- --test-threads=$TEST_THREADS; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_success "Integration tests completed successfully in ${duration}s!"
        echo "[CI-SUMMARY] INTEGRATION PASS ${duration}s"

        # Clean up test resources after successful completion
        cleanup_test_resources
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_error "Integration tests failed in ${duration}s!"
        echo "[CI-SUMMARY] INTEGRATION FAIL ${duration}s"

        # Clean up resources even if tests fail
        cleanup_test_resources
        exit 1
    fi
}

# Function to run security tests
run_security_tests() {
    print_status "Running security tests for sortingoffice..."

    # Set test environment
    export RUST_LOG=error
    export RUST_BACKTRACE=0
    export TESTCONTAINERS_LOG_LEVEL=error
    export BOLLARD_LOG_LEVEL=error

    # Reduce test threads for CI environment to prevent resource contention
    if [ "$CI" = "true" ]; then
        export RUST_TEST_THREADS=2
        print_warning "CI environment detected, reducing security test threads to 2"
    else
        export RUST_TEST_THREADS=1
    fi

    # Run security tests
    print_status "Running security tests with cargo..."
    start_time=$(date +%s)
    if cargo test --test security; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_success "Security tests completed successfully in ${duration}s!"
        echo "[CI-SUMMARY] SECURITY PASS ${duration}s"
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_error "Security tests failed in ${duration}s!"
        echo "[CI-SUMMARY] SECURITY FAIL ${duration}s"
        exit 1
    fi
}

# Function to run API tests
run_api_tests() {
    print_status "Running API tests for sortingoffice..."

    # Set test environment
    export RUST_LOG=error
    export RUST_BACKTRACE=0
    export TESTCONTAINERS_LOG_LEVEL=error
    export BOLLARD_LOG_LEVEL=error

    # Reduce test threads for CI environment to prevent resource contention
    if [ "$CI" = "true" ]; then
        export RUST_TEST_THREADS=2
        print_warning "CI environment detected, reducing API test threads to 2"
    else
        export RUST_TEST_THREADS=1
    fi

    # Run API tests
    print_status "Running API tests with cargo..."
    start_time=$(date +%s)
    if cargo test --test api; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_success "API tests completed successfully in ${duration}s!"
        echo "[CI-SUMMARY] API PASS ${duration}s"
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_error "API tests failed in ${duration}s!"
        echo "[CI-SUMMARY] API FAIL ${duration}s"
        exit 1
    fi
}

# Function to check if application is healthy
check_app_health() {
    local max_attempts=5  # Reduced attempts since we're failing fast
    local attempt=1
    local wait_time=1

    print_status "Checking if application is healthy..."
    while [ $attempt -le $max_attempts ]; do
        # Try the health check endpoint
        local response=$(curl -s -w "%{http_code}" -o /dev/null http://localhost:3000/health)
        if [ "$response" = "200" ]; then
            print_success "Application is healthy and responding on localhost:3000"
            return 0
        elif [ -n "$response" ] && [ "$response" != "000" ]; then
            # If we got a response but it's not 200, fail fast
            print_error "Application health check failed with status $response"
            return 1
        fi
        print_status "Waiting for application to be ready (attempt $attempt/$max_attempts)..."
        sleep $wait_time
        attempt=$((attempt + 1))
    done

    print_error "Application is not responding after $max_attempts attempts"
    return 1
}

# Function to cleanup test containers and networks
cleanup_test_resources() {
    print_status "Cleaning up test resources..."

    # Clean up test containers using Docker commands
    if command -v docker > /dev/null 2>&1; then
        # Remove orphaned MySQL test containers
        docker ps -a --format "{{.ID}} {{.Image}} {{.Names}}" | grep " mysql" | grep -v "sortingoffice" | grep -E "(test|mysql)" | awk '{print $1}' | xargs -r docker rm -f 2>/dev/null || true

        # Remove orphaned Selenium test containers
        docker ps -a --format "{{.ID}} {{.Image}} {{.Names}}" | grep " selenium" | grep -v "sortingoffice" | grep -E "(test|selenium)" | awk '{print $1}' | xargs -r docker rm -f 2>/dev/null || true

        # Remove orphaned app test containers
        docker ps -a --format "{{.ID}} {{.Image}} {{.Names}}" | grep " sortingoffice" | grep "test" | awk '{print $1}' | xargs -r docker rm -f 2>/dev/null || true

        # Remove shared test network
        docker network rm sortingoffice-e2e 2>/dev/null || true

        print_success "Test resources cleaned up successfully!"
    else
        print_warning "Docker not available, skipping container cleanup"
    fi
}

# Function to run containerized UI tests
run_ui_tests() {
    print_status "Running containerized UI tests for sortingoffice..."

    # Check if Docker is available
    if ! command -v docker > /dev/null 2>&1; then
        print_error "Docker is not available. Please install Docker and try again."
        exit 1
    fi

    # Check if Docker daemon is running
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker daemon is not running. Please start Docker and try again."
        exit 1
    fi

    # Set environment variables - optimize for CI
    if [ "$CI" = "true" ]; then
        export RUST_TEST_THREADS=1
        export TEST_THREADS=1
        print_warning "CI environment detected, using single-threaded execution"
    else
        export RUST_TEST_THREADS="${TEST_THREADS:-8}"
        export TEST_THREADS="${TEST_THREADS:-8}"
    fi

    export RUST_LOG=info
    export RUST_BACKTRACE=0

    # Run the containerized UI tests (uses testcontainers for database and Selenium)
    print_status "Running containerized UI tests with testcontainers..."
    
    # Run ui_containerized tests
    print_status "Running ui_containerized tests..."
    if cargo test --test ui_containerized -- --nocapture --test-threads=$RUST_TEST_THREADS; then
        print_success "ui_containerized tests passed!"
    else
        print_error "ui_containerized tests failed!"
        cleanup_test_resources
        exit 1
    fi
    
    # Run duplicate_wizard_ui_tests
    print_status "Running duplicate_wizard_ui_tests..."
    if cargo test --test duplicate_wizard_ui_tests -- --nocapture --test-threads=$RUST_TEST_THREADS; then
        print_success "duplicate_wizard_ui_tests passed!"
    else
        print_error "duplicate_wizard_ui_tests failed!"
        cleanup_test_resources
        exit 1
    fi
    
    # Run wizard_tests
    print_status "Running wizard_tests..."
    if cargo test --test wizard_tests -- --nocapture --test-threads=$RUST_TEST_THREADS; then
        print_success "wizard_tests passed!"
    else
        print_error "wizard_tests failed!"
        cleanup_test_resources
        exit 1
    fi

    # Clean up test resources after successful completion
    cleanup_test_resources

    echo ""
    print_success "Containerized UI tests completed successfully! 🎉"
}

# Function to run smoke test
run_smoke_test() {
    local test_url="${1:-http://localhost:3000}"
    print_status "Running end-to-end smoke test for sortingoffice against $test_url..."

    # Check if application is running on the specified URL
    if curl -s "$test_url/health" > /dev/null 2>&1; then
        print_status "Found running application at $test_url, using environment-based smoke test..."
        
        # Set environment variables
        export RUST_LOG=info
        export RUST_BACKTRACE=0

        # Run the environment-based smoke test (uses testcontainers for Selenium only)
        print_status "Running smoke test with testcontainers Selenium..."
        if cargo test ui_smoke_e2e_flow -- --ignored --nocapture; then
            print_success "Smoke test passed!"
        else
            print_error "Smoke test failed!"
            exit 1
        fi
    else
        print_error "No application found at $test_url!"
        print_error "The smoke test requires a running SortingOffice application to test against."
        echo ""
        print_status "To fix this:"
        print_status "  1. Start the application: cargo run"
        print_status "  2. Or use: make run"
        print_status "  3. Ensure the application is accessible at $test_url"
        echo ""
        print_status "Alternative: Use 'make test-smoke-containerized' to test with containers"
        echo ""
        print_error "Smoke test failed - no application to test against!"
        exit 1
    fi

    echo ""
    print_success "Smoke test completed successfully! 🎉"
}

run_smoke_test_containerized() {
    print_status "Running end-to-end smoke test for sortingoffice..."

    # Check if Docker is available
    if ! command -v docker > /dev/null 2>&1; then
        print_error "Docker is not available. Please install Docker and try again."
        exit 1
    fi

    # Check if Docker daemon is running
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker daemon is not running. Please start Docker and try again."
        exit 1
    fi

    # Set environment variables
    export RUST_LOG=info
    export RUST_BACKTRACE=0

    # Run the smoke test
    print_status "Running smoke test in testcontainers..."
    if cargo test ui_smoke_containerized_e2e_flow -- --nocapture; then
        print_success "Smoke test passed!"
    else
        print_error "Smoke test failed!"
        # Clean up resources even if tests fail
        cleanup_test_resources
        exit 1
    fi

    # Clean up test resources after successful completion
    cleanup_test_resources

    echo ""
    print_success "Smoke test completed successfully! 🎉"
}

# Function to run all tests
run_all_tests() {
    print_status "Running all tests..."
    start_all=$(date +%s)
    run_unit_tests
    echo ""
    run_integration_tests
    echo ""
    run_security_tests
    echo ""
    run_api_tests
    echo ""
    run_ui_tests
    end_all=$(date +%s)
    duration_all=$((end_all - start_all))
    print_success "All tests completed in ${duration_all}s!"
    echo "[CI-SUMMARY] ALL PASS ${duration_all}s"
}

# Function to run individual test with cleanup
run_single_test() {
    local test_name="$1"
    if [ -z "$test_name" ]; then
        print_error "Test name is required for single test run"
        echo "Usage: $0 single TEST_NAME"
        echo "Example: $0 single test_homepage_loads_containerized"
        exit 1
    fi

    print_status "Running individual test: $test_name"
    echo "This will run the test and then clean up any test containers..."

    # Set test environment
    export RUST_LOG=error
    export RUST_BACKTRACE=0
    export TESTCONTAINERS_LOG_LEVEL=error
    export BOLLARD_LOG_LEVEL=error

    # Run the individual test
    start_time=$(date +%s)
    if cargo test "$test_name" -- --nocapture; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_success "Individual test '$test_name' passed in ${duration}s!"
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_error "Individual test '$test_name' failed in ${duration}s!"
    fi

    # Always clean up after individual test
    cleanup_test_resources
}

# Function to run individual UI test with cleanup
run_single_ui_test() {
    local test_name="$1"
    if [ -z "$test_name" ]; then
        print_error "Test name is required for single UI test run"
        echo "Usage: $0 single-ui TEST_NAME"
        echo "Example: $0 single-ui test_homepage_loads_containerized"
        exit 1
    fi

    print_status "Running individual UI test: $test_name"
    echo "This will run the UI test and then clean up any test containers..."

    # Set test environment
    export RUST_LOG=error
    export RUST_BACKTRACE=0
    export TESTCONTAINERS_LOG_LEVEL=error
    export BOLLARD_LOG_LEVEL=error

    # Run the individual UI test
    start_time=$(date +%s)
    
    # Try to run from ui_containerized first, then duplicate_wizard_ui_tests, then wizard_tests
    if cargo test --test ui_containerized "$test_name" -- --nocapture 2>/dev/null; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_success "Individual UI test '$test_name' passed in ${duration}s!"
    elif cargo test --test duplicate_wizard_ui_tests "$test_name" -- --nocapture 2>/dev/null; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_success "Individual UI test '$test_name' passed in ${duration}s!"
    elif cargo test --test wizard_tests "$test_name" -- --nocapture 2>/dev/null; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_success "Individual UI test '$test_name' passed in ${duration}s!"
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_error "Individual UI test '$test_name' failed in ${duration}s!"
        print_error "Test not found in ui_containerized, duplicate_wizard_ui_tests, or wizard_tests"
    fi

    # Always clean up after individual test
    cleanup_test_resources
}

# Main script logic
case "${1:-unit}" in
    "unit")
        run_unit_tests
        ;;
    "integration")
        run_integration_tests
        ;;
    "security")
        run_security_tests
        ;;
    "api")
        run_api_tests
        ;;
    "ui")
        run_ui_tests
        ;;
    "ui-containerized")
        run_ui_tests
        ;;
    "smoke")
        run_smoke_test "$2"
        ;;
    "smoke-containerized")
        run_smoke_test_containerized
        ;;
    "all")
        run_all_tests
        ;;
    "single")
        run_single_test "$2"
        ;;
    "single-ui")
        run_single_ui_test "$2"
        ;;
    "cleanup")
        cleanup_test_resources
        ;;
    "help"|"-h"|"--help")
        show_usage
        ;;
    *)
        print_error "Unknown option: $1"
        echo ""
        show_usage
        exit 1
        ;;
esac
