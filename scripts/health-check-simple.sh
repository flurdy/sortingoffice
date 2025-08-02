#!/bin/bash

# Super Simple One-Liner Health Check for Sorting Office
# Usage: ./scripts/health-check-simple.sh [host]

HOST="${1:-localhost:3000}"

# Ensure HOST has protocol
if [[ ! "$HOST" =~ ^https?:// ]]; then
    HOST="http://$HOST"
fi

# Simple health check
if curl -s -f --connect-timeout 5 --max-time 10 "$HOST/health" > /dev/null 2>&1; then
    echo "✅ Health check PASSED - $HOST is healthy"
    exit 0
else
    echo "❌ Health check FAILED - $HOST is not responding"
    exit 1
fi 
