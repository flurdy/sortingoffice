#!/bin/bash

# Generic Refactoring Script
# This script can handle any type of refactoring operation
# Usage: ./scripts/refactor_generic.sh <operation> [options]
#
# Operations:
#   move-functions <old_module> <new_module> <function1> [function2] ...
#   move-module <old_path> <new_path>
#   rename-function <old_name> <new_name>
#   rename-module <old_name> <new_name>
#   update-imports <old_path> <new_path>
#   fix-compilation
#   cleanup-imports

set -e

OPERATION=$1
shift

case $OPERATION in
    "move-functions")
        if [ $# -lt 3 ]; then
            echo "Usage: $0 move-functions <old_module> <new_module> <function1> [function2] ..."
            exit 1
        fi
        OLD_MODULE=$1
        NEW_MODULE=$2
        shift 2
        FUNCTIONS=("$@")
        
        echo "Moving functions from $OLD_MODULE to $NEW_MODULE..."
        for function in "${FUNCTIONS[@]}"; do
            echo "  Moving function: $function"
            
            # Update import statements
            find src -name "*.rs" -exec sed -i "s|use crate::handlers::${OLD_MODULE}::${function}|use crate::handlers::${NEW_MODULE}::${function}|g" {} \;
            
            # Update function calls with module prefix
            find src -name "*.rs" -exec sed -i "s|crate::handlers::${OLD_MODULE}::${function}(|crate::handlers::${NEW_MODULE}::${function}(|g" {} \;
            
            # Update function calls without module prefix (carefully)
            find src -name "*.rs" -exec sed -i "s|([[:space:]]*${function}(|(\1${NEW_MODULE}::${function}(|g" {} \;
            find src -name "*.rs" -exec sed -i "s|^[[:space:]]*${function}(|${NEW_MODULE}::${function}(|g" {} \;
            find src -name "*.rs" -exec sed -i "s|[[:space:]]${function}(| ${NEW_MODULE}::${function}(|g" {} \;
        done
        ;;
        
    "move-module")
        if [ $# -ne 2 ]; then
            echo "Usage: $0 move-module <old_path> <new_path>"
            exit 1
        fi
        OLD_PATH=$1
        NEW_PATH=$2
        
        echo "Moving module from $OLD_PATH to $NEW_PATH..."
        
        # Create new directory if it doesn't exist
        mkdir -p $(dirname "$NEW_PATH")
        
        # Move the file
        mv "$OLD_PATH" "$NEW_PATH"
        
        # Update all references
        find src -name "*.rs" -exec sed -i "s|mod ${OLD_PATH}|mod ${NEW_PATH}|g" {} \;
        find src -name "*.rs" -exec sed -i "s|use crate::${OLD_PATH}|use crate::${NEW_PATH}|g" {} \;
        ;;
        
    "rename-function")
        if [ $# -ne 2 ]; then
            echo "Usage: $0 rename-function <old_name> <new_name>"
            exit 1
        fi
        OLD_NAME=$1
        NEW_NAME=$2
        
        echo "Renaming function from $OLD_NAME to $NEW_NAME..."
        
        # Update function definitions
        find src -name "*.rs" -exec sed -i "s|fn ${OLD_NAME}(|fn ${NEW_NAME}(|g" {} \;
        
        # Update function calls
        find src -name "*.rs" -exec sed -i "s|${OLD_NAME}(|${NEW_NAME}(|g" {} \;
        
        # Update imports
        find src -name "*.rs" -exec sed -i "s|${OLD_NAME},|${NEW_NAME},|g" {} \;
        find src -name "*.rs" -exec sed -i "s|, ${OLD_NAME}|, ${NEW_NAME}|g" {} \;
        ;;
        
    "rename-module")
        if [ $# -ne 2 ]; then
            echo "Usage: $0 rename-module <old_name> <new_name>"
            exit 1
        fi
        OLD_NAME=$1
        NEW_NAME=$2
        
        echo "Renaming module from $OLD_NAME to $NEW_NAME..."
        
        # Update module declarations
        find src -name "*.rs" -exec sed -i "s|mod ${OLD_NAME};|mod ${NEW_NAME};|g" {} \;
        
        # Update use statements
        find src -name "*.rs" -exec sed -i "s|use crate::handlers::${OLD_NAME}|use crate::handlers::${NEW_NAME}|g" {} \;
        
        # Update function calls
        find src -name "*.rs" -exec sed -i "s|${OLD_NAME}::|${NEW_NAME}::|g" {} \;
        ;;
        
    "update-imports")
        if [ $# -ne 2 ]; then
            echo "Usage: $0 update-imports <old_path> <new_path>"
            exit 1
        fi
        OLD_PATH=$1
        NEW_PATH=$2
        
        echo "Updating imports from $OLD_PATH to $NEW_PATH..."
        find src -name "*.rs" -exec sed -i "s|${OLD_PATH}|${NEW_PATH}|g" {} \;
        ;;
        
    "fix-compilation")
        echo "Fixing common compilation errors..."
        
        # Fix malformed function definitions
        find src -name "*.rs" -exec sed -i 's/pub async fn \([a-z_]*\)::\([a-z_]*\)(/pub async fn \2(/g' {} \;
        find src -name "*.rs" -exec sed -i 's/pub fn \([a-z_]*\)::\([a-z_]*\)(/pub fn \2(/g' {} \;
        
        # Fix double module references
        find src -name "*.rs" -exec sed -i 's|crate::handlers::\([a-z_]*\)::\1::|crate::handlers::\1::|g' {} \;
        
        # Fix self-references in modules
        find src -name "*.rs" -exec sed -i 's|\([a-z_]*\)::\([a-z_]*\)(|\2(|g' {} \;
        ;;
        
    "cleanup-imports")
        echo "Cleaning up unused imports..."
        
        # Remove common unused imports
        find src -name "*.rs" -exec sed -i '/use crate::models::PaginatedResult;/d' {} \;
        find src -name "*.rs" -exec sed -i '/use diesel::result::Error;/d' {} \;
        find src -name "*.rs" -exec sed -i '/use tracing::info;/d' {} \;
        ;;
        
    *)
        echo "Unknown operation: $OPERATION"
        echo "Available operations:"
        echo "  move-functions <old_module> <new_module> <function1> [function2] ..."
        echo "  move-module <old_path> <new_path>"
        echo "  rename-function <old_name> <new_name>"
        echo "  rename-module <old_name> <new_name>"
        echo "  update-imports <old_path> <new_path>"
        echo "  fix-compilation"
        echo "  cleanup-imports"
        exit 1
        ;;
esac

echo "Refactoring complete!" 
