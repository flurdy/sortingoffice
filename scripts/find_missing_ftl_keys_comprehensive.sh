#!/usr/bin/env bash
# Find missing translation keys (used in code but not defined in FTL files) - Comprehensive version

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
        if [[ $line =~ \[\\\"([a-zA-Z0-9_-]+)\\\"\] ]]; then
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

# Function to detect dynamic key patterns
detect_dynamic_patterns() {
    local file="$1"
    local patterns=()
    
    # Look for format! patterns that generate keys
    while IFS= read -r line; do
        if [[ $line =~ format!.*\"([a-zA-Z0-9_-]+)\{([a-zA-Z0-9_-]+)\}\" ]]; then
            local base_pattern="${BASH_REMATCH[1]}"
            local variable="${BASH_REMATCH[2]}"
            patterns+=("$base_pattern-{entity}")
        fi
    done < "$file"
    
    # Look for common dynamic patterns
    while IFS= read -r line; do
        if [[ $line =~ error-duplicate- ]]; then
            patterns+=("error-duplicate-{entity}")
        fi
        if [[ $line =~ error-.*- ]]; then
            patterns+=("error-{type}-{entity}")
        fi
    done < "$file"
    
    printf '%s\n' "${patterns[@]}" | sort | uniq
}

# Collect all keys used in code
USED_KEYS=()
DYNAMIC_PATTERNS=()

while IFS= read -r -d '' file; do
    if [[ "$file" =~ \.(rs|html)$ ]]; then
        keys=$(extract_keys_from_code "$file")
        while IFS= read -r key; do
            if [[ -n "$key" ]]; then
                USED_KEYS+=("$key")
            fi
        done <<< "$keys"
        
        patterns=$(detect_dynamic_patterns "$file")
        while IFS= read -r pattern; do
            if [[ -n "$pattern" ]]; then
                DYNAMIC_PATTERNS+=("$pattern")
            fi
        done <<< "$patterns"
    fi
done < <(find src/ templates/ -type f \( -name "*.rs" -o -name "*.html" \) -print0)

# Remove duplicates and sort
USED_KEYS_UNIQUE=$(printf '%s\n' "${USED_KEYS[@]}" | sort | uniq)
DYNAMIC_PATTERNS_UNIQUE=$(printf '%s\n' "${DYNAMIC_PATTERNS[@]}" | sort | uniq)

MISSING=()
MISSING_DYNAMIC=()

# Check each used key against defined keys
while IFS= read -r key; do
    if [[ -n "$key" ]] && ! echo "$DEFINED_KEYS" | grep -q "^$key$"; then
        MISSING+=("$key")
    fi
done <<< "$USED_KEYS_UNIQUE"

# Check for missing dynamic patterns
while IFS= read -r pattern; do
    if [[ -n "$pattern" ]]; then
        # Check if any keys matching this pattern are missing
        case "$pattern" in
            "error-duplicate-{entity}")
                # Check for common entity types
                for entity in domain user backup alias relay relocated; do
                    key="error-duplicate-$entity"
                    if ! echo "$DEFINED_KEYS" | grep -q "^$key$"; then
                        MISSING_DYNAMIC+=("$key (from pattern: $pattern)")
                    fi
                done
                ;;
            "error-{type}-{entity}")
                # Check for common error types and entities
                for error_type in duplicate constraint; do
                    for entity in domain user backup alias relay relocated; do
                        key="error-$error_type-$entity"
                        if ! echo "$DEFINED_KEYS" | grep -q "^$key$"; then
                            MISSING_DYNAMIC+=("$key (from pattern: $pattern)")
                        fi
                    done
                done
                ;;
        esac
    fi
done <<< "$DYNAMIC_PATTERNS_UNIQUE"

echo "=== STATIC MISSING KEYS ==="
if [ ${#MISSING[@]} -eq 0 ]; then
    echo "No static missing keys found."
else
    echo "Missing translation keys (used in code but not defined in $FTL_FILE):"
    for key in "${MISSING[@]}"; do
        echo "  $key"
    done
fi

echo ""
echo "=== DYNAMIC MISSING KEYS ==="
if [ ${#MISSING_DYNAMIC[@]} -eq 0 ]; then
    echo "No dynamic missing keys found."
else
    echo "Missing translation keys (from dynamic patterns):"
    for key in "${MISSING_DYNAMIC[@]}"; do
        echo "  $key"
    done
fi

echo ""
echo "=== SUMMARY ==="
echo "Total static missing keys: ${#MISSING[@]}"
echo "Total dynamic missing keys: ${#MISSING_DYNAMIC[@]}"
echo "Total missing keys: $(( ${#MISSING[@]} + ${#MISSING_DYNAMIC[@]} ))" 
