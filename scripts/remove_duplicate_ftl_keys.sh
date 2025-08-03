#!/bin/bash

# Script to remove duplicate translation keys from all language files
# This script removes duplicate keys while preserving the first occurrence

LANGUAGES=("en-US" "de-DE" "fr-FR" "es-ES" "nb-NO")

for lang in "${LANGUAGES[@]}"; do
    echo "Processing $lang..."
    
    LANG_FILE="resources/locales/$lang/messages.ftl"
    TEMP_FILE="${LANG_FILE}.tmp"
    
    # Create backup
    cp "$LANG_FILE" "${LANG_FILE}.bak.$(date +%Y%m%d_%H%M%S)"
    
    # Remove duplicates using awk
    # Store lines with their keys and maintain original order
    awk -F'=' '
    {
        key = $1
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", key)
        if (key != "") {
            # Store the line with its original key for later sorting
            lines[key] = $0
            order[++count] = key
        }
    }
    END {
        # Output unique keys in original order
        for (i = 1; i <= count; i++) {
            key = order[i]
            if (key in seen) continue
            seen[key] = 1
            print lines[key]
        }
    }' "$LANG_FILE" > "$TEMP_FILE"
    
    # Replace original with deduplicated version
    mv "$TEMP_FILE" "$LANG_FILE"
    
    echo "Removed duplicates from $lang"
done

echo "Done! All duplicate keys have been removed."
echo "Backups saved as .bak files." 
