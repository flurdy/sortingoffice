#!/bin/bash

# Environment setup script for Sorting Office
# This script helps manage different database configurations

set -e

# Default values
DEFAULT_HOST="127.0.0.1"
DEFAULT_USER="root"
DEFAULT_PASSWORD="password"
DEFAULT_PORT="3306"

# Function to show usage
show_usage() {
    echo "Usage: $0 [environment]"
    echo ""
    echo "Environments:"
    echo "  dev     - Development environment (sortingoffice database)"
    echo "  test    - Test environment (sortingoffice_test database)"
    echo "  docker  - Docker environment (using docker.env)"
    echo "  custom  - Custom database configuration"
    echo ""
    echo "Examples:"
    echo "  $0 dev"
    echo "  $0 test"
    echo "  $0 docker"
    echo "  $0 custom"
}

# Function to set environment for development
set_dev_env() {
    echo "🔧 Setting up development environment..."
    export DATABASE_URL="mysql://${DEFAULT_USER}:${DEFAULT_PASSWORD}@${DEFAULT_HOST}:${DEFAULT_PORT}/sortingoffice"
    export RUST_LOG="debug"
    echo "✅ Development environment set:"
    echo "   DATABASE_URL=$DATABASE_URL"
    echo "   RUST_LOG=$RUST_LOG"
    echo ""
    echo "To use this environment, run:"
    echo "  source scripts/set-env.sh dev"
    echo "  cargo run"
}

# Function to set environment for testing
set_test_env() {
    echo "🧪 Setting up test environment..."
    export DATABASE_URL="mysql://${DEFAULT_USER}:${DEFAULT_PASSWORD}@${DEFAULT_HOST}:${DEFAULT_PORT}/sortingoffice_test"
    export RUST_LOG="debug"
    echo "✅ Test environment set:"
    echo "   DATABASE_URL=$DATABASE_URL"
    echo "   RUST_LOG=$RUST_LOG"
    echo ""
    echo "To use this environment, run:"
    echo "  source scripts/set-env.sh test"
    echo "  cargo test"
}

# Function to set environment for Docker
set_docker_env() {
    echo "🐳 Setting up Docker environment..."
    if [ -f "docker.env" ]; then
        export $(cat docker.env | grep -v '^#' | xargs)
        echo "✅ Docker environment loaded from docker.env"
        echo "   DATABASE_URL=$DATABASE_URL"
        echo "   RUST_LOG=$RUST_LOG"
    else
        echo "❌ docker.env file not found!"
        exit 1
    fi
}

# Function to set custom environment
set_custom_env() {
    echo "🔧 Setting up custom environment..."
    echo "Enter database configuration:"
    
    read -p "Host [$DEFAULT_HOST]: " host
    host=${host:-$DEFAULT_HOST}
    
    read -p "User [$DEFAULT_USER]: " user
    user=${user:-$DEFAULT_USER}
    
    read -p "Password [$DEFAULT_PASSWORD]: " password
    password=${password:-$DEFAULT_PASSWORD}
    
    read -p "Port [$DEFAULT_PORT]: " port
    port=${port:-$DEFAULT_PORT}
    
    read -p "Database name: " database
    if [ -z "$database" ]; then
        echo "❌ Database name is required!"
        exit 1
    fi
    
    export DATABASE_URL="mysql://${user}:${password}@${host}:${port}/${database}"
    export RUST_LOG="debug"
    
    echo "✅ Custom environment set:"
    echo "   DATABASE_URL=$DATABASE_URL"
    echo "   RUST_LOG=$RUST_LOG"
}

# Main script logic
case "${1:-}" in
    "dev")
        set_dev_env
        ;;
    "test")
        set_test_env
        ;;
    "docker")
        set_docker_env
        ;;
    "custom")
        set_custom_env
        ;;
    "help"|"-h"|"--help")
        show_usage
        ;;
    *)
        echo "❌ Invalid environment: $1"
        echo ""
        show_usage
        exit 1
        ;;
esac 
