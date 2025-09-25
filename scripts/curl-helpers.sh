#!/bin/bash

# Sorting Office Curl Helpers
# A collection of curl-based utilities for testing and interacting with the Sorting Office API

set -e

# Configuration
DEFAULT_HOST="localhost:3000"
DEFAULT_USER="admin"
DEFAULT_PASSWORD="admin123"
COOKIE_FILE="/tmp/sortingoffice_cookies.txt"

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
Sorting Office Curl Helpers

Usage: $0 <command> [options]

Commands:
    login [user] [password] [host]     - Login and save session cookies
    logout                             - Clear session cookies
    status                             - Check if logged in
    backup-list                        - List database backups
    backup-create <database_id>        - Create a new backup
    backup-download <filename>         - Download a backup file
    backup-delete <filename>           - Delete a backup file
    database-switch <database_id>      - Switch to a different database
    aliases-list                       - List aliases
    domains-list                       - List domains
    domains-create <domain> [transport] - Create a new domain
    users-list                         - List users
    stats                              - Get system statistics
    health                             - Check application health
    config                             - Get configuration page
    help                               - Show this help message

Examples:
    $0 login                           # Login with default credentials
    $0 login admin mypassword          # Login with custom password
    $0 login admin mypassword 192.168.1.100:3000  # Login to remote host
    $0 backup-list                     # List backups (requires login)
    $0 backup-create primary           # Create backup of primary database
    $0 aliases-list                    # List aliases

Environment Variables:
    SORTINGOFFICE_HOST                 - Default host (default: $DEFAULT_HOST)
    SORTINGOFFICE_USER                 - Default username (default: $DEFAULT_USER)
    SORTINGOFFICE_PASSWORD             - Default password (default: $DEFAULT_PASSWORD)

EOF
}

# Check if logged in by testing a protected endpoint
check_login() {
    if [ ! -f "$COOKIE_FILE" ]; then
        return 1
    fi
    
    # Load environment from cookie file if it exists
    if [ -f "$COOKIE_FILE.env" ]; then
        source "$COOKIE_FILE.env"
    fi
    
    # Use default host if HOST is not set
    local host="${HOST:-http://$DEFAULT_HOST}"
    
    local response=$(curl -s -o /dev/null -w "%{http_code}" -b "$COOKIE_FILE" "$host/database_backup/list")
    if [ "$response" = "200" ]; then
        return 0
    else
        return 1
    fi
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
        echo "HOST=$HOST" > "$COOKIE_FILE.env"
        echo "USER=$user" >> "$COOKIE_FILE.env"
    else
        log_error "Login failed with status code: $status_code"
        log_error "Response: $body"
        rm -f "$COOKIE_FILE"
        exit 1
    fi
}

# Logout function
logout() {
    if [ -f "$COOKIE_FILE" ]; then
        log_info "Logging out..."
        curl -s -b "$COOKIE_FILE" "$HOST/logout" > /dev/null || true
        rm -f "$COOKIE_FILE" "$COOKIE_FILE.env"
        log_success "Logged out and cleared cookies"
    else
        log_warning "No active session found"
    fi
}

# Check login status
status() {
    if check_login; then
        log_success "Logged in"
        if [ -f "$COOKIE_FILE.env" ]; then
            source "$COOKIE_FILE.env"
            echo "Host: $HOST"
            echo "User: $USER"
        fi
    else
        log_warning "Not logged in"
    fi
}

# Require login for protected operations
require_login() {
    if ! check_login; then
        log_error "Not logged in. Please run: $0 login"
        exit 1
    fi
    
    # Load environment from cookie file
    if [ -f "$COOKIE_FILE.env" ]; then
        source "$COOKIE_FILE.env"
    fi
}

# Backup operations
backup_list() {
    require_login
    log_info "Fetching backup list..."
    curl -s -b "$COOKIE_FILE" "$HOST/database_backup/list" | jq '.' 2>/dev/null || curl -s -b "$COOKIE_FILE" "$HOST/database_backup/list"
}

backup_create() {
    require_login
    local database_id="$1"
    if [ -z "$database_id" ]; then
        log_error "Database ID required"
        echo "Usage: $0 backup-create <database_id>"
        exit 1
    fi
    
    log_info "Creating backup for database: $database_id"
    curl -s -b "$COOKIE_FILE" \
        -X POST "$HOST/database_backup/create-htmx" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "database_id=$database_id"
}

backup_download() {
    require_login
    local filename="$1"
    if [ -z "$filename" ]; then
        log_error "Filename required"
        echo "Usage: $0 backup-download <filename>"
        exit 1
    fi
    
    log_info "Downloading backup: $filename"
    curl -s -b "$COOKIE_FILE" -O "$HOST/database_backup/download/$filename"
    log_success "Downloaded: $filename"
}

backup_delete() {
    require_login
    local filename="$1"
    if [ -z "$filename" ]; then
        log_error "Filename required"
        echo "Usage: $0 backup-delete <filename>"
        exit 1
    fi
    
    log_info "Deleting backup: $filename"
    curl -s -b "$COOKIE_FILE" \
        -X DELETE "$HOST/database_backup/delete/$filename" \
        -H "Accept: application/json"
}

database_switch() {
    require_login
    local database_id="$1"
    if [ -z "$database_id" ]; then
        log_error "Database ID required"
        echo "Usage: $0 database-switch <database_id>"
        exit 1
    fi
    
    log_info "Switching to database: $database_id"
    local response=$(curl -s -w "%{http_code}" -b "$COOKIE_FILE" -c "$COOKIE_FILE" \
        -X POST "$HOST/database/select" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "database_id=$database_id")
    
    local status_code="${response: -3}"
    local body="${response%???}"
    
    if [ "$status_code" = "302" ] || [ "$status_code" = "200" ]; then
        log_success "Switched to database: $database_id"
    else
        log_error "Database switch failed with status code: $status_code"
        log_error "Response: $body"
        exit 1
    fi
}

# Resource listing operations
aliases_list() {
    require_login
    log_info "Fetching aliases list..."
    curl -s -b "$COOKIE_FILE" "$HOST/aliases" | grep -E "(table|tr|td)" | head -20 || curl -s -b "$COOKIE_FILE" "$HOST/aliases"
}

domains_list() {
    require_login
    log_info "Fetching domains list..."
    curl -s -b "$COOKIE_FILE" "$HOST/domains" | grep -E "(table|tr|td)" | head -20 || curl -s -b "$COOKIE_FILE" "$HOST/domains"
}

domains_create() {
    require_login
    local domain="$1"
    local transport="${2:-virtual}"
    
    if [ -z "$domain" ]; then
        log_error "Domain name required"
        echo "Usage: $0 domains-create <domain> [transport]"
        exit 1
    fi
    
    log_info "Creating domain: $domain with transport: $transport"
    local response=$(curl -s -w "%{http_code}" -b "$COOKIE_FILE" \
        -X POST "$HOST/domains" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "domain=$domain&transport=$transport&enabled=on")
    
    local status_code="${response: -3}"
    local body="${response%???}"
    
    if [ "$status_code" = "200" ]; then
        log_success "Domain created successfully: $domain"
        echo "$body"
    else
        log_error "Domain creation failed with status code: $status_code"
        log_error "Response: $body"
        exit 1
    fi
}

users_list() {
    require_login
    log_info "Fetching users list..."
    curl -s -b "$COOKIE_FILE" "$HOST/users" | grep -E "(table|tr|td)" | head -20 || curl -s -b "$COOKIE_FILE" "$HOST/users"
}

# System operations
stats() {
    require_login
    log_info "Fetching system statistics..."
    curl -s -b "$COOKIE_FILE" "$HOST/stats" | jq '.' 2>/dev/null || curl -s -b "$COOKIE_FILE" "$HOST/stats"
}

health() {
    log_info "Checking application health..."
    curl -s "$HOST/health" | jq '.' 2>/dev/null || curl -s "$HOST/health"
}

config() {
    require_login
    log_info "Fetching configuration page..."
    curl -s -b "$COOKIE_FILE" "$HOST/config" | grep -E "(form|input|select)" | head -20 || curl -s -b "$COOKIE_FILE" "$HOST/config"
}

# Main command dispatcher
main() {
    local command="$1"
    
    case "$command" in
        login)
            login "$2" "$3" "$4"
            ;;
        logout)
            logout
            ;;
        status)
            status
            ;;
        backup-list)
            backup_list
            ;;
        backup-create)
            backup_create "$2"
            ;;
        backup-download)
            backup_download "$2"
            ;;
        backup-delete)
            backup_delete "$2"
            ;;
        database-switch)
            database_switch "$2"
            ;;
        aliases-list)
            aliases_list
            ;;
        domains-list)
            domains_list
            ;;
        domains-create)
            domains_create "$2" "$3"
            ;;
        users-list)
            users_list
            ;;
        stats)
            stats
            ;;
        health)
            health
            ;;
        config)
            config
            ;;
        help|--help|-h)
            show_usage
            ;;
        "")
            show_usage
            exit 1
            ;;
        *)
            log_error "Unknown command: $command"
            show_usage
            exit 1
            ;;
    esac
}

# Load environment variables
if [ -n "$SORTINGOFFICE_HOST" ]; then
    DEFAULT_HOST="$SORTINGOFFICE_HOST"
fi

if [ -n "$SORTINGOFFICE_USER" ]; then
    DEFAULT_USER="$SORTINGOFFICE_USER"
fi

if [ -n "$SORTINGOFFICE_PASSWORD" ]; then
    DEFAULT_PASSWORD="$SORTINGOFFICE_PASSWORD"
fi

# Run main function
main "$@" 
