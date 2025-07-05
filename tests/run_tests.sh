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
    echo "  all               Run all tests (unit + UI)"
    echo "  ui-setup          Setup UI test environment"
    echo "  help              Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                # Run unit tests"
    echo "  $0 unit           # Run unit tests"
    echo "  $0 ui             # Run UI tests"
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
    cargo test --verbose

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

    # Check if Selenium WebDriver is running
    print_status "Checking if Selenium WebDriver is running..."
    if ! curl -s http://localhost:4444/status > /dev/null 2>&1; then
        print_warning "Selenium WebDriver is not running on localhost:4444"
        print_status "Please start Selenium WebDriver first:"
        echo "  $0 ui-setup"
        echo "  # or manually:"
        echo "  docker compose --profile test up -d selenium"
        echo ""
        print_status "Then run this script again."
        exit 1
    fi

    print_success "Selenium WebDriver is running on localhost:4444"

    # Set environment variables
    export RUST_TEST_THREADS=1
    export RUST_LOG=info

    # Run the UI tests
    print_status "Running UI tests..."
    echo ""

    # Run basic UI tests
    print_status "Running basic UI tests..."
    if cargo test --test ui -- --nocapture --test-threads=1; then
        print_success "Basic UI tests passed!"
    else
        print_error "Basic UI tests failed!"
        exit 1
    fi

    echo ""

    # Run advanced UI tests
    print_status "Running advanced UI tests..."
    if cargo test --test ui_advanced -- --nocapture --test-threads=1; then
        print_success "Advanced UI tests passed!"
    else
        print_error "Advanced UI tests failed!"
        exit 1
    fi

    echo ""
    print_success "All UI tests completed successfully! 🎉"
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
