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
    echo "  unit              Run only unit tests (default)"
    echo "  ui                Run only UI tests"
    echo "  ui-headless       Run only headless UI tests"
    echo "  ui-containerized  Run containerized UI tests (app + db in containers)"
    echo "  all               Run all tests (unit + UI)"
    echo "  ui-setup          Setup UI test environment"
    echo "  help              Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                # Run unit tests"
    echo "  $0 unit           # Run unit tests"
    echo "  $0 ui             # Run UI tests"
    echo "  $0 ui-headless    # Run headless UI tests"
    echo "  $0 ui-containerized # Run containerized UI tests"
    echo "  $0 all            # Run all tests"
    echo "  $0 ui-setup       # Setup UI test environment"
}

# Function to run unit tests
run_unit_tests() {
    print_status "Running unit tests for sortingoffice..."
    
    # Check if DATABASE_URL is set, if not use default test database
    if [ -z "$DATABASE_URL" ]; then
        export DATABASE_URL="mysql://root:password@localhost/sortingoffice_test"
        print_warning "DATABASE_URL not set, using default test database: $DATABASE_URL"
    fi

    # Set test environment
    export RUST_LOG=debug
    export RUST_BACKTRACE=1
    export RUST_TEST_THREADS=1

    # Run the unit tests (excluding UI tests)
    print_status "Running unit tests with cargo..."
    # Run only the integration tests from src/tests/ by running the main binary tests
    cargo test --bin sortingoffice --verbose

    print_success "Unit tests completed successfully!"
}

# Function to run UI tests
run_ui_tests() {
    print_status "Running UI tests for sortingoffice..."
    
    # Check if the application is running
    print_status "Checking if application is running..."
    if ! curl -s http://localhost:3000/ > /dev/null 2>&1; then
        print_warning "Application is not running on localhost:3000"
        print_status "Please start the application first:"
        echo "  cargo run"
        echo "  # or"
        echo "  docker-compose up"
        echo ""
        print_status "Then run this script again."
        exit 1
    fi

    print_success "Application is running on localhost:3000"

    # Ensure seed data is loaded for UI tests
    print_status "Ensuring seed data is loaded for UI tests..."
    if [ -f "seed_data/all.sql" ]; then
        # Try to load seed data using the default database configuration
        if mysql -uroot -prootpassword -h127.0.0.1 -P3306 sortingoffice < seed_data/all.sql 2>/dev/null; then
            print_success "Seed data loaded successfully!"
        else
            print_warning "Could not load seed data automatically. UI tests may fail if database is empty."
            print_status "You can manually load seed data with: make seed"
        fi
    else
        print_warning "No seed data found. UI tests may fail if database is empty."
        print_status "You can create seed data with: make create-seed-data"
    fi

    # Set environment variables
    export RUST_TEST_THREADS=1
    export RUST_LOG=info

    # Run the headless UI tests (now the only UI tests)
    print_status "Running headless UI tests with testcontainers..."
    if cargo test --test ui_headless -- --nocapture --test-threads=1; then
        print_success "Headless UI tests passed!"
    else
        print_error "Headless UI tests failed!"
        exit 1
    fi

    echo ""
    print_success "All UI tests completed successfully! 🎉"
}

# Function to run headless UI tests
run_headless_ui_tests() {
    print_status "Running headless UI tests for sortingoffice..."
    
    # Check if the application is running
    print_status "Checking if application is running..."
    if ! curl -s http://localhost:3000/ > /dev/null 2>&1; then
        print_warning "Application is not running on localhost:3000"
        print_status "Please start the application first:"
        echo "  cargo run"
        echo "  # or"
        echo "  docker-compose up"
        echo ""
        print_status "Then run this script again."
        exit 1
    fi

    print_success "Application is running on localhost:3000"

    # Ensure seed data is loaded for UI tests
    print_status "Ensuring seed data is loaded for UI tests..."
    if [ -f "seed_data/all.sql" ]; then
        # Try to load seed data using the default database configuration
        if mysql -uroot -prootpassword -h127.0.0.1 -P3306 sortingoffice < seed_data/all.sql 2>/dev/null; then
            print_success "Seed data loaded successfully!"
        else
            print_warning "Could not load seed data automatically. UI tests may fail if database is empty."
            print_status "You can manually load seed data with: make seed"
        fi
    else
        print_warning "No seed data found. UI tests may fail if database is empty."
        print_status "You can create seed data with: make create-seed-data"
    fi

    # Set environment variables
    export RUST_TEST_THREADS=1
    export RUST_LOG=info

    # Run the headless UI tests (uses testcontainers automatically)
    print_status "Running headless UI tests with testcontainers..."
    if cargo test --test ui_headless -- --nocapture --test-threads=1; then
        print_success "Headless UI tests passed!"
    else
        print_error "Headless UI tests failed!"
        exit 1
    fi

    echo ""
    print_success "Headless UI tests completed successfully! 🎉"
}

# Function to run containerized UI tests
run_containerized_ui_tests() {
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

    # Check if the application is running
    print_status "Checking if application is running..."
    if ! curl -s http://localhost:3000/ > /dev/null 2>&1; then
        print_warning "Application is not running on localhost:3000"
        print_status "Please start the application first:"
        echo "  cargo run"
        echo "  # or"
        echo "  docker-compose up"
        echo ""
        print_status "Then run this script again."
        exit 1
    fi

    print_success "Application is running on localhost:3000"

    # Set environment variables
    export RUST_TEST_THREADS=1
    export RUST_LOG=info

    # Run the containerized UI tests (uses testcontainers for database and Selenium)
    print_status "Running containerized UI tests with testcontainers..."
    if cargo test --test ui_headless_containerized -- --nocapture --test-threads=1; then
        print_success "Containerized UI tests passed!"
    else
        print_error "Containerized UI tests failed!"
        exit 1
    fi

    echo ""
    print_success "Containerized UI tests completed successfully! 🎉"
}

# Function to setup UI test environment
setup_ui_tests() {
    print_status "Setting up UI test environment..."
    
    # Check if Docker Compose is available
    if ! command -v docker-compose > /dev/null 2>&1 && ! docker compose version > /dev/null 2>&1; then
        print_error "Docker Compose is not available. Please install Docker Compose and try again."
        exit 1
    fi

    # Start Selenium service using Docker Compose
    print_status "Starting Selenium WebDriver with Docker Compose..."
    if docker compose version > /dev/null 2>&1; then
        docker compose --profile test up -d selenium
    else
        docker-compose --profile test up -d selenium
    fi

    if [ $? -eq 0 ]; then
        print_success "Selenium WebDriver started!"
        print_status "You can access VNC viewer at localhost:7900 (password: secret)"
        print_status "Now run: $0 ui"
    else
        print_error "Failed to start Selenium WebDriver"
        exit 1
    fi
}

# Function to run all tests
run_all_tests() {
    print_status "Running all tests..."
    
    # Run unit tests first
    run_unit_tests
    echo ""
    
    # Run UI tests
    run_ui_tests
}

# Main script logic
case "${1:-unit}" in
    "unit")
        run_unit_tests
        ;;
    "ui")
        run_ui_tests
        ;;
    "ui-headless")
        run_headless_ui_tests
        ;;
    "ui-containerized")
        run_containerized_ui_tests
        ;;
    "all")
        run_all_tests
        ;;
    "ui-setup")
        setup_ui_tests
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
