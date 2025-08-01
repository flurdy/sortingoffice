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
    echo "  all               Run all tests (unit + integration + security + api + UI)"
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
    echo "  $0 all            # Run all tests"
}

# Function to run unit tests
run_unit_tests() {
    print_status "Running unit tests for sortingoffice..."
    
    # Set test environment
    export RUST_LOG=error
    export RUST_BACKTRACE=1
    export RUST_TEST_THREADS=1
    export TESTCONTAINERS_LOG_LEVEL=error
    export BOLLARD_LOG_LEVEL=error

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
    TEST_THREADS=8

    # Run only the integration tests (excluding UI tests)
    print_status "Running integration tests with cargo (threads: $TEST_THREADS)..."
    start_time=$(date +%s)
    if cargo test --test integration --test handlers --test testcontainers_test -- --test-threads=$TEST_THREADS; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_success "Integration tests completed successfully in ${duration}s!"
        echo "[CI-SUMMARY] INTEGRATION PASS ${duration}s"
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_error "Integration tests failed in ${duration}s!"
        echo "[CI-SUMMARY] INTEGRATION FAIL ${duration}s"
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

    # Set environment variables
    export RUST_TEST_THREADS="${TEST_THREADS:-8}"
    export RUST_LOG=info
    export RUST_BACKTRACE=0

    # Run the containerized UI tests (uses testcontainers for database and Selenium)
    print_status "Running containerized UI tests with testcontainers..."
    if cargo test --test ui_containerized -- --nocapture --test-threads=$RUST_TEST_THREADS; then
        print_success "Containerized UI tests passed!"
    else
        print_error "Containerized UI tests failed!"
        exit 1
    fi

    echo ""
    print_success "Containerized UI tests completed successfully! 🎉"
}

# Function to run smoke test
run_smoke_test() {
    print_status "Running end-to-end smoke test for sortingoffice..."

    # Set environment variables
    export RUST_LOG=info
    export RUST_BACKTRACE=0

    # Run the smoke test (uses testcontainers for Selenium)
    print_status "Running smoke test with testcontainers Selenium..."
    if cargo test ui_smoke_e2e_flow -- --ignored --nocapture; then
        print_success "Smoke test passed!"
    else
        print_error "Smoke test failed!"
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
    if cargo test ui_smoke_containerized_e2e_flow -- --ignored --nocapture; then
        print_success "Smoke test passed!"   
    else
        print_error "Smoke test failed!"
        exit 1
    fi

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
        run_smoke_test
        ;;
    "smoke-containerized")
        run_smoke_test_containerized
        ;;
    "all")
        run_all_tests
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
