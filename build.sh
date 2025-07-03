#!/bin/bash

# Sorting Office Build Script
# This script helps set up and build the Sorting Office mail server admin tool

set -e

echo "🚀 Sorting Office Build Script"
echo "================================"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust first:"
    echo "   https://rustup.rs/"
    exit 1
fi

# Check if Diesel CLI is installed
if ! command -v diesel &> /dev/null; then
    echo "📦 Installing Diesel CLI..."
    cargo install diesel_cli --no-default-features --features mysql
fi

# Check if .env file exists
if [ ! -f .env ]; then
    echo "📝 Creating .env file from template..."
    cp env.example .env
    echo "⚠️  Please edit .env file with your database credentials before continuing"
    echo "   DATABASE_URL=mysql://username:password@localhost/sortingoffice"
    read -p "Press Enter when you've configured .env..."
fi

# Build the project
echo "🔨 Building project..."
cargo build --release

# Run database setup
echo "🗄️  Setting up database..."
diesel setup

# Run migrations
echo "📊 Running database migrations..."
diesel migration run

echo "✅ Build complete!"
echo ""
echo "To run the application:"
echo "  cargo run"
echo ""
echo "The application will be available at: http://localhost:3000"
echo "Default login: admin/admin"
echo ""
echo "⚠️  Remember to change the default credentials in production!" 
