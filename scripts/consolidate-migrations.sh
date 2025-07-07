#!/bin/bash

# Migration Consolidation Script
# This script helps consolidate multiple migrations into a single migration

set -e

echo "🔄 Migration Consolidation Script"
echo "================================="
echo ""

# Check if we're in the right directory
if [ ! -d "migrations" ]; then
    echo "❌ Error: migrations directory not found!"
    echo "Please run this script from the project root directory."
    exit 1
fi

echo "📋 Current migrations:"
ls -la migrations/
echo ""

echo "⚠️  WARNING: This will consolidate all migrations into a single migration."
echo "This is a destructive operation that will remove old migration files."
echo ""
echo "Are you sure you want to continue? (y/N)"
read -p "" confirm

if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
    echo "Operation cancelled."
    exit 0
fi

echo ""
echo "🔧 Starting migration consolidation..."

# Create backup directory
BACKUP_DIR="migrations_backup_$(date +%Y%m%d_%H%M%S)"
echo "📦 Creating backup in $BACKUP_DIR..."
mkdir -p "$BACKUP_DIR"
cp -r migrations/* "$BACKUP_DIR/"

# Remove old migrations (keep only the consolidated one)
echo "🗑️  Removing old migration files..."
find migrations/ -mindepth 1 -maxdepth 1 -type d ! -name "*consolidated_schema_final*" -exec rm -rf {} \;

echo ""
echo "✅ Migration consolidation complete!"
echo ""
echo "📁 Current migrations:"
ls -la migrations/
echo ""
echo "📦 Backup created in: $BACKUP_DIR"
echo ""
echo "🔄 Next steps:"
echo "1. Run 'make migrate-reset' to apply the consolidated migration"
echo "2. Run 'make seed' to load seed data"
echo "3. Test your application"
echo ""
echo "⚠️  Note: If something goes wrong, you can restore from the backup:"
echo "   cp -r $BACKUP_DIR/* migrations/" 
