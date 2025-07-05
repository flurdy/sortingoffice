#!/bin/bash

# Test runner script for sortingoffice
set -e

echo "🧪 Running unit tests for sortingoffice..."

# Check if DATABASE_URL is set, if not use default test database
if [ -z "$DATABASE_URL" ]; then
    export DATABASE_URL="mysql://root:password@localhost/sortingoffice_test"
    echo "⚠️  DATABASE_URL not set, using default test database: $DATABASE_URL"
fi

# Set test environment
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Run the tests
echo "📋 Running tests with cargo..."
cargo test --verbose

echo "✅ All tests completed successfully!" 
