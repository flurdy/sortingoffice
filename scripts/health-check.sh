#!/bin/bash

# Super Simple Health Check Smoke Test for Sorting Office
# This script performs a basic health check using curl

set -e

# Configuration
DEFAULT_HOST="localhost:3000"
DEFAULT_TIMEOUT=10

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

show_usage() {
    cat << EOF
Super Simple Health Check Smoke Test for Sorting Office

Usage: $0 [options]

Options:
    -h, --host HOST:PORT     Target host (default: $DEFAULT_HOST)
    -t, --timeout SECONDS    Timeout in seconds (default: $DEFAULT_TIMEOUT)
    -v, --verbose            Verbose output
    --help                   Show this help message

Examples:
    $0                        # Check localhost:3000
    $0 -h 192.168.1.100:3000 # Check remote host
    $0 -t 5                  # 5 second timeout
    $0 -v                    # Verbose output

Environment Variables:
    SORTINGOFFICE_HOST       - Default host (default: $DEFAULT_HOST)
    SORTINGOFFICE_TIMEOUT    - Default timeout (default: $DEFAULT_TIMEOUT)

EOF
}

# Parse command line arguments
HOST="$DEFAULT_HOST"
TIMEOUT="$DEFAULT_TIMEOUT"
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--host)
            HOST="$2"
            shift 2
            ;;
        -t|--timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Load environment variables
if [ -n "$SORTINGOFFICE_HOST" ]; then
    HOST="$SORTINGOFFICE_HOST"
fi

if [ -n "$SORTINGOFFICE_TIMEOUT" ]; then
    TIMEOUT="$SORTINGOFFICE_TIMEOUT"
fi

# Ensure HOST has protocol
if [[ ! "$HOST" =~ ^https?:// ]]; then
    HOST="http://$HOST"
fi

# Health check function
perform_health_check() {
    local url="$HOST/health"
    local start_time=$(date +%s)
    
    log_info "Performing health check on $url"
    log_info "Timeout: ${TIMEOUT}s"
    
    if [ "$VERBOSE" = true ]; then
        log_info "Using verbose output"
    fi
    
    # Perform the health check
    local response
    local http_code
    local curl_output
    
    if [ "$VERBOSE" = true ]; then
        # Verbose curl with full output
        curl_output=$(curl -s -w "\nHTTP_CODE:%{http_code}\nTIME:%{time_total}\n" \
            --connect-timeout "$TIMEOUT" \
            --max-time "$TIMEOUT" \
            "$url" 2>&1)
        http_code=$(echo "$curl_output" | grep "HTTP_CODE:" | cut -d: -f2)
        response=$(echo "$curl_output" | grep -v "HTTP_CODE:" | grep -v "TIME:")
    else
        # Silent curl with just status code
        http_code=$(curl -s -o /dev/null -w "%{http_code}" \
            --connect-timeout "$TIMEOUT" \
            --max-time "$TIMEOUT" \
            "$url" 2>/dev/null || echo "000")
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    # Check the response
    if [ "$http_code" = "200" ]; then
        log_success "Health check PASSED"
        log_success "Status: $http_code"
        log_success "Duration: ${duration}s"
        
        if [ "$VERBOSE" = true ]; then
            log_info "Response: $response"
        fi
        
        return 0
    elif [ "$http_code" = "000" ]; then
        log_error "Health check FAILED - Connection timeout or refused"
        log_error "Duration: ${duration}s"
        return 1
    else
        log_error "Health check FAILED - HTTP $http_code"
        log_error "Duration: ${duration}s"
        
        if [ "$VERBOSE" = true ]; then
            log_info "Response: $response"
        fi
        
        return 1
    fi
}

# Main execution
main() {
    log_info "Starting Sorting Office health check smoke test"
    log_info "Target: $HOST"
    
    # Check if curl is available
    if ! command -v curl &> /dev/null; then
        log_error "curl is not installed or not in PATH"
        exit 1
    fi
    
    # Perform the health check
    if perform_health_check; then
        log_success "Health check completed successfully"
        exit 0
    else
        log_error "Health check failed"
        exit 1
    fi
}

# Run main function
main "$@" 
