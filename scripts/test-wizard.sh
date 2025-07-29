#!/bin/bash

# Wizard Test Script for Sorting Office
# Tests the wizard form submission functionality

set -e

# Configuration
DEFAULT_HOST="localhost:3000"
DEFAULT_USER="admin"
DEFAULT_PASSWORD="admin123"
COOKIE_FILE="/tmp/sortingoffice_wizard_cookies.txt"

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
Wizard Test Script

Usage: $0 <command> [options]

Commands:
    test-domain-config [host] [user] [password]  - Test domain configuration step
    test-alias-config [host] [user] [password]   - Test alias configuration step
    test-full-wizard [host] [user] [password]    - Test complete wizard flow
    help                                          - Show this help message

Examples:
    $0 test-domain-config                        # Test domain config with defaults
    $0 test-alias-config localhost:3000 admin admin123  # Test alias config
    $0 test-full-wizard                          # Test complete wizard flow

Environment Variables:
    SORTINGOFFICE_HOST                 - Default host (default: $DEFAULT_HOST)
    SORTINGOFFICE_USER                 - Default username (default: $DEFAULT_USER)
    SORTINGOFFICE_PASSWORD             - Default password (default: $DEFAULT_PASSWORD)

EOF
}

# Login function
login() {
    local user="${1:-$DEFAULT_USER}"
    local password="${2:-$DEFAULT_PASSWORD}"
    local host="${3:-$DEFAULT_HOST}"
    
    HOST="http://$host"
    
    log_info "Logging in as $user to $HOST..."
    
    # Clear any existing cookies
    rm -f "$COOKIE_FILE"
    
    # Perform login
    local response=$(curl -s -w "%{http_code}" -c "$COOKIE_FILE" \
        -X POST "$HOST/login" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "id=$user&password=$password")
    
    local status_code="${response: -3}"
    local body="${response%???}"
    
    if [ "$status_code" = "302" ] || [ "$status_code" = "200" ]; then
        log_success "Login successful for user: $user"
    else
        log_error "Login failed with status code: $status_code"
        log_error "Response: $body"
        rm -f "$COOKIE_FILE"
        exit 1
    fi
}

# Test domain configuration step
test_domain_config() {
    local host="${1:-$DEFAULT_HOST}"
    local user="${2:-$DEFAULT_USER}"
    local password="${3:-$DEFAULT_PASSWORD}"
    
    HOST="http://$host"
    
    log_info "Testing domain configuration step..."
    
    # Login first
    login "$user" "$password" "$host"
    
    # Get the domain config page to extract CSRF token if needed
    log_info "Fetching domain configuration page..."
    local page_response=$(curl -s -b "$COOKIE_FILE" "$HOST/wizard/domain-config")
    
    # Check if page loads successfully
    if echo "$page_response" | grep -q "Configure Domains"; then
        log_success "Domain configuration page loaded successfully"
    else
        log_error "Failed to load domain configuration page"
        return 1
    fi
    
    # Test form submission with sample domains
    log_info "Testing domain configuration form submission..."
    local form_data="domains=test1.example.com%2C+test2.example.org&transport=virtual&enabled=on"
    
    local response=$(curl -s -w "%{http_code}" -b "$COOKIE_FILE" \
        -X POST "$HOST/wizard/domain-config" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "$form_data")
    
    local status_code="${response: -3}"
    local body="${response%???}"
    
    log_info "Form submission status code: $status_code"
    
    if [ "$status_code" = "200" ] || [ "$status_code" = "302" ]; then
        log_success "Domain configuration form submission successful"
        if echo "$body" | grep -q "Configure Aliases"; then
            log_success "Redirected to alias configuration step"
        else
            log_warning "Response doesn't contain expected alias configuration content"
        fi
    else
        log_error "Domain configuration form submission failed"
        log_error "Response: $body"
        return 1
    fi
}

# Test alias configuration step
test_alias_config() {
    local host="${1:-$DEFAULT_HOST}"
    local user="${2:-$DEFAULT_USER}"
    local password="${3:-$DEFAULT_PASSWORD}"
    
    HOST="http://$host"
    
    log_info "Testing alias configuration step..."
    
    # Login first
    login "$user" "$password" "$host"
    
    # First, submit domain configuration to get to alias config
    log_info "Setting up domain configuration for alias test..."
    local domain_form_data="domains=test.example.com&transport=virtual&enabled=on"
    
    curl -s -b "$COOKIE_FILE" \
        -X POST "$HOST/wizard/domain-config" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "$domain_form_data" > /dev/null
    
    # Get the alias config page
    log_info "Fetching alias configuration page..."
    local page_response=$(curl -s -b "$COOKIE_FILE" "$HOST/wizard/alias-config")
    
    # Check if page loads successfully
    if echo "$page_response" | grep -q "Configure Aliases"; then
        log_success "Alias configuration page loaded successfully"
    else
        log_error "Failed to load alias configuration page"
        return 1
    fi
    
    # Test form submission with sample aliases
    log_info "Testing alias configuration form submission..."
    local form_data="required_aliases=postmaster&required_aliases=abuse&common_aliases=admin&common_aliases=webmaster&custom_aliases=info&common_destination=admin%40example.com&catchall_enabled=on"
    
    local response=$(curl -s -w "%{http_code}" -b "$COOKIE_FILE" \
        -X POST "$HOST/wizard/alias-config" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "$form_data")
    
    local status_code="${response: -3}"
    local body="${response%???}"
    
    log_info "Form submission status code: $status_code"
    
    if [ "$status_code" = "200" ] || [ "$status_code" = "302" ]; then
        log_success "Alias configuration form submission successful"
        if echo "$body" | grep -q "Review"; then
            log_success "Redirected to review step"
        else
            log_warning "Response doesn't contain expected review content"
        fi
    else
        log_error "Alias configuration form submission failed"
        log_error "Response: $body"
        return 1
    fi
}

# Test complete wizard flow
test_full_wizard() {
    local host="${1:-$DEFAULT_HOST}"
    local user="${2:-$DEFAULT_USER}"
    local password="${3:-$DEFAULT_PASSWORD}"
    
    HOST="http://$host"
    
    log_info "Testing complete wizard flow..."
    
    # Login first
    login "$user" "$password" "$host"
    
    # Test domain configuration
    log_info "Step 1: Testing domain configuration..."
    test_domain_config "$host" "$user" "$password"
    
    # Test alias configuration
    log_info "Step 2: Testing alias configuration..."
    test_alias_config "$host" "$user" "$password"
    
    # Test review step
    log_info "Step 3: Testing review step..."
    local review_response=$(curl -s -b "$COOKIE_FILE" "$HOST/wizard/review")
    
    if echo "$review_response" | grep -q "Review"; then
        log_success "Review step loaded successfully"
    else
        log_error "Failed to load review step"
        return 1
    fi
    
    # Test execution step
    log_info "Step 4: Testing execution step..."
    local execute_form_data="confirmed=on"
    
    local execute_response=$(curl -s -w "%{http_code}" -b "$COOKIE_FILE" \
        -X POST "$HOST/wizard/execute" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "$execute_form_data")
    
    local status_code="${execute_response: -3}"
    local body="${execute_response%???}"
    
    if [ "$status_code" = "200" ] || [ "$status_code" = "302" ]; then
        log_success "Execution step successful"
        if echo "$body" | grep -q "Complete"; then
            log_success "Wizard completed successfully"
        else
            log_warning "Response doesn't contain expected completion content"
        fi
    else
        log_error "Execution step failed"
        log_error "Response: $body"
        return 1
    fi
    
    log_success "Complete wizard flow test passed!"
}

# Main script logic
main() {
    local command="$1"
    
    case "$command" in
        "test-domain-config")
            test_domain_config "$2" "$3" "$4"
            ;;
        "test-alias-config")
            test_alias_config "$2" "$3" "$4"
            ;;
        "test-full-wizard")
            test_full_wizard "$2" "$3" "$4"
            ;;
        "help"|"--help"|"-h"|"")
            show_usage
            ;;
        *)
            log_error "Unknown command: $command"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@" 
