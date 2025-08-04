#!/bin/bash

# Simple Refactoring Wrapper
# Provides easy-to-use commands for common refactoring tasks
# Usage: ./scripts/refactor.sh <command> [args]

set -e

COMMAND=$1
shift

case $COMMAND in
    "move")
        # Move functions between modules
        # Usage: ./scripts/refactor.sh move <old_module> <new_module> <function1> [function2] ...
        ./scripts/refactor_generic.sh move-functions "$@"
        ;;
        
    "rename")
        # Rename functions or modules
        # Usage: ./scripts/refactor.sh rename function <old_name> <new_name>
        # Usage: ./scripts/refactor.sh rename module <old_name> <new_name>
        if [ "$1" = "function" ]; then
            shift
            ./scripts/refactor_generic.sh rename-function "$@"
        elif [ "$1" = "module" ]; then
            shift
            ./scripts/refactor_generic.sh rename-module "$@"
        else
            echo "Usage: $0 rename function <old_name> <new_name>"
            echo "   or: $0 rename module <old_name> <new_name>"
            exit 1
        fi
        ;;
        
    "fix")
        # Fix compilation errors
        ./scripts/refactor_generic.sh fix-compilation
        ;;
        
    "cleanup")
        # Clean up unused imports
        ./scripts/refactor_generic.sh cleanup-imports
        ;;
        
    "help"|"-h"|"--help")
        echo "Simple Refactoring Tool"
        echo ""
        echo "Commands:"
        echo "  move <old_module> <new_module> <function1> [function2] ..."
        echo "    Move functions between modules"
        echo ""
        echo "  rename function <old_name> <new_name>"
        echo "    Rename a function"
        echo ""
        echo "  rename module <old_name> <new_name>"
        echo "    Rename a module"
        echo ""
        echo "  fix"
        echo "    Fix common compilation errors"
        echo ""
        echo "  cleanup"
        echo "    Clean up unused imports"
        echo ""
        echo "Examples:"
        echo "  ./scripts/refactor.sh move utils database_helpers get_current_db_pool"
        echo "  ./scripts/refactor.sh rename function old_name new_name"
        echo "  ./scripts/refactor.sh fix"
        ;;
        
    *)
        echo "Unknown command: $COMMAND"
        echo "Run '$0 help' for usage information"
        exit 1
        ;;
esac 
