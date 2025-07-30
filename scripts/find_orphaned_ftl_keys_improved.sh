#!/usr/bin/env bash
# Find orphaned translation keys in en-US/messages.ftl (Improved version)

FTL_FILE="resources/locales/en-US/messages.ftl"
CODE_DIRS=(src templates)

# Extract all keys from the .ftl file
KEYS=$(grep -E '^[a-zA-Z0-9_-]+\s*=' "$FTL_FILE" | sed 's/\s*=.*//' | sort | uniq)

ORPHANED=()

for key in $KEYS; do
    # Search for the key in multiple patterns:
    # 1. Literal key name
    # 2. Key in quotes (for template strings)
    # 3. Key in format! macro (for dynamic generation)
    # 4. Key in get_translation calls
    # 5. Key in translation map access
    
    FOUND=false
    
    # Pattern 1: Literal key name
    if grep -r -E "\\b$key\\b" src/ templates/ --include='*.rs' --include='*.html' > /dev/null 2>&1; then
        FOUND=true
    fi
    
    # Pattern 2: Key in quotes
    if grep -r "\"$key\"" src/ templates/ --include='*.rs' --include='*.html' > /dev/null 2>&1; then
        FOUND=true
    fi
    
    # Pattern 3: Key in format! macro (for dynamic keys like error-duplicate-{entity})
    if grep -r "format!.*$key" src/ templates/ --include='*.rs' --include='*.html' > /dev/null 2>&1; then
        FOUND=true
    fi
    
    # Pattern 4: Key in get_translation calls
    if grep -r "get_translation.*\"$key\"" src/ templates/ --include='*.rs' --include='*.html' > /dev/null 2>&1; then
        FOUND=true
    fi
    
    # Pattern 5: Key in translation map access (like form_translations["key"])
    if grep -r "\[\"$key\"\]" src/ templates/ --include='*.rs' --include='*.html' > /dev/null 2>&1; then
        FOUND=true
    fi
    
    # Pattern 6: Dynamic key patterns (for keys like error-duplicate-{entity})
    # Check if this key follows a pattern that's generated dynamically
    if [[ $key =~ ^error-duplicate- ]]; then
        # Check for dynamic generation patterns
        if grep -r "error-duplicate-.*entity" src/ templates/ --include='*.rs' --include='*.html' > /dev/null 2>&1; then
            FOUND=true
        fi
    fi
    
    # Pattern 7: Check for key patterns in string literals
    if grep -r "'$key'" src/ templates/ --include='*.rs' --include='*.html' > /dev/null 2>&1; then
        FOUND=true
    fi
    
    if [ "$FOUND" = false ]; then
        ORPHANED+=("$key")
    fi
done

if [ ${#ORPHANED[@]} -eq 0 ]; then
    echo "No orphaned keys found."
else
    echo "Orphaned keys in $FTL_FILE:"
    for key in "${ORPHANED[@]}"; do
        echo "$key"
    done
fi 
