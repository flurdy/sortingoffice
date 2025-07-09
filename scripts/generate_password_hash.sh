#!/bin/bash

# Generate bcrypt password hash for Sorting Office admin users
# Usage: ./scripts/generate_password_hash.sh "your_password"

if [ $# -eq 0 ]; then
    echo "Usage: $0 \"your_password\""
    echo "Example: $0 \"mypassword123\""
    exit 1
fi

PASSWORD="$1"

# Check if Python 3 is available
if command -v python3 &> /dev/null; then
    echo "Generating bcrypt hash using Python 3..."
    python3 -c "import bcrypt; print(bcrypt.hashpw('$PASSWORD'.encode('utf-8'), bcrypt.gensalt()).decode('utf-8'))"
elif command -v node &> /dev/null; then
    echo "Generating bcrypt hash using Node.js..."
    node -e "const bcrypt = require('bcrypt'); bcrypt.hash('$PASSWORD', 12).then(hash => console.log(hash))"
else
    echo "Error: Neither Python 3 nor Node.js found."
    echo "Please install one of them or use an online bcrypt generator:"
    echo "https://bcrypt.online/"
    echo ""
    echo "For Python 3: pip install bcrypt"
    echo "For Node.js: npm install bcrypt"
    exit 1
fi

echo ""
echo "Copy the hash above into your config.toml file as the password_hash value." 
