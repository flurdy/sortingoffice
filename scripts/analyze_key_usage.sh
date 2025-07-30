#!/usr/bin/env bash
# Analyze how translation keys are used in the codebase

KEY="$1"

if [ -z "$KEY" ]; then
    echo "Usage: $0 <key_name>"
    echo "Example: $0 error-duplicate-domain"
    exit 1
fi

echo "Analyzing usage of key: $KEY"
echo "=================================="

echo "1. Literal key name (word boundaries):"
grep -r -E "\\b$KEY\\b" src/ templates/ --include='*.rs' --include='*.html' || echo "No matches found"

echo ""
echo "2. Key in double quotes:"
grep -r "\"$KEY\"" src/ templates/ --include='*.rs' --include='*.html' || echo "No matches found"

echo ""
echo "3. Key in single quotes:"
grep -r "'$KEY'" src/ templates/ --include='*.rs' --include='*.html' || echo "No matches found"

echo ""
echo "4. Key in format! macro:"
grep -r "format!.*$KEY" src/ templates/ --include='*.rs' --include='*.html' || echo "No matches found"

echo ""
echo "5. Key in get_translation calls:"
grep -r "get_translation.*\"$KEY\"" src/ templates/ --include='*.rs' --include='*.html' || echo "No matches found"

echo ""
echo "6. Key in translation map access:"
grep -r "\[\"$KEY\"\]" src/ templates/ --include='*.rs' --include='*.html' || echo "No matches found"

echo ""
echo "7. Dynamic key patterns (for keys like error-duplicate-{entity}):"
if [[ $KEY =~ ^error-duplicate- ]]; then
    echo "Key follows error-duplicate- pattern, checking for dynamic generation:"
    grep -r "error-duplicate-.*entity" src/ templates/ --include='*.rs' --include='*.html' || echo "No dynamic generation patterns found"
else
    echo "Key does not follow error-duplicate- pattern"
fi

echo ""
echo "8. Any mention of the key (case insensitive):"
grep -r -i "$KEY" src/ templates/ --include='*.rs' --include='*.html' || echo "No matches found" 
