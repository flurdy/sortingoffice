#!/usr/bin/env bash
# Find orphaned translation keys in en-US/messages.ftl

FTL_FILE="resources/locales/en-US/messages.ftl"
CODE_DIRS=(src templates)

# Extract all keys from the .ftl file
KEYS=$(grep -E '^[a-zA-Z0-9_-]+\s*=' "$FTL_FILE" | sed 's/\s*=.*//' | sort | uniq)

ORPHANED=()

for key in $KEYS; do
    # Search for the key in .rs and .html files
    if ! grep -r -E "\\b$key\\b" src/ templates/ --include='*.rs' --include='*.html' > /dev/null; then
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
