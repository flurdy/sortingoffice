#!/usr/bin/env bash
# Find missing translation keys (used in code but not defined in FTL files)

FTL_FILE="resources/locales/en-US/messages.ftl"
CODE_DIRS=(src templates)

# Extract all defined keys from the .ftl file
DEFINED_KEYS=$(grep -E '^[a-zA-Z0-9_-]+\s*=' "$FTL_FILE" | sed 's/\s*=.*//' | sort | uniq)

# Function to extract keys from code patterns
extract_keys_from_code() {
    local file="$1"
    local keys=()
    
    # Pattern 1: get_translation calls with string literals
    while IFS= read -r line; do
        if [[ $line =~ get_translation.*\"([a-zA-Z0-9_-]+)\" ]]; then
            keys+=("${BASH_REMATCH[1]}")
        fi
    done < "$file"
    
    # Pattern 2: Translation map access (like form_translations["key"])
    while IFS= read -r line; do
        if [[ $line =~ \[\"([a-zA-Z0-9_-]+)\"\] ]]; then
            keys+=("${BASH_REMATCH[1]}")
        fi
    done < "$file"
    
    # Pattern 3: Template string interpolation (like &field_translations["key"])
    while IFS= read -r line; do
        if [[ $line =~ \&[a-zA-Z_]+\[\\\"([a-zA-Z0-9_-]+)\\\"\] ]]; then
            keys+=("${BASH_REMATCH[1]}")
        fi
    done < "$file"
    
    # Pattern 4: Direct string literals that look like translation keys
    while IFS= read -r line; do
        if [[ $line =~ \"([a-zA-Z0-9_-]+)\" ]] && [[ ${BASH_REMATCH[1]} =~ ^[a-z] ]]; then
            # Only include if it looks like a translation key (starts with lowercase, contains hyphens)
            if [[ ${BASH_REMATCH[1]} =~ [a-z].*[-_][a-z] ]]; then
                keys+=("${BASH_REMATCH[1]}")
            fi
        fi
    done < "$file"
    
    # Pattern 5: format! macro with translation key patterns
    while IFS= read -r line; do
        if [[ $line =~ format!.*\"([a-zA-Z0-9_-]+)\" ]]; then
            keys+=("${BASH_REMATCH[1]}")
        fi
    done < "$file"
    
    # Return unique keys
    printf '%s\n' "${keys[@]}" | sort | uniq
}

# Collect all keys used in code
USED_KEYS=()
while IFS= read -r -d '' file; do
    if [[ "$file" =~ \.(rs|html)$ ]]; then
        keys=$(extract_keys_from_code "$file")
        while IFS= read -r key; do
            if [[ -n "$key" ]]; then
                USED_KEYS+=("$key")
            fi
        done <<< "$keys"
    fi
done < <(find src/ templates/ -type f \( -name "*.rs" -o -name "*.html" \) -print0)

# Remove duplicates and sort
USED_KEYS_UNIQUE=$(printf '%s\n' "${USED_KEYS[@]}" | sort | uniq)

MISSING=()

# Check each used key against defined keys
while IFS= read -r key; do
    if [[ -n "$key" ]] && ! echo "$DEFINED_KEYS" | grep -q "^$key$"; then
        MISSING+=("$key")
    fi
done <<< "$USED_KEYS_UNIQUE"

if [ ${#MISSING[@]} -eq 0 ]; then
    echo "No missing keys found."
else
    echo "Missing translation keys (used in code but not defined in $FTL_FILE):"
    for key in "${MISSING[@]}"; do
        echo "$key"
    done
fi 
