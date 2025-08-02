#!/bin/bash

# SSH Tunnel Manager for Sorting Office Production Databases
# This script helps manage SSH tunnels for secure database connections

set -e

# Configuration
DEFAULT_SSH_USER="root"
DEFAULT_SSH_PORT="22"
DEFAULT_LOCAL_PORT="3306"
TUNNEL_CONFIG_FILE="scripts/tunnel-config.sh"

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
SSH Tunnel Manager for Sorting Office

Usage: $0 <command> [options]

Commands:
    start <tunnel_name>     Start an SSH tunnel
    stop <tunnel_name>      Stop an SSH tunnel
    status                  Show tunnel status
    list                    List configured tunnels
    create <name>           Create a new tunnel configuration
    help                    Show this help message

Examples:
    $0 start production     # Start production tunnel
    $0 stop production      # Stop production tunnel
    $0 status              # Show all tunnel statuses
    $0 create staging      # Create new staging tunnel config

Tunnel Configuration:
    Create tunnel configurations in $TUNNEL_CONFIG_FILE
    Example configuration:
        declare -A TUNNEL_PRODUCTION=(
            [host]="prod-server.example.com"
            [user]="root"
            [port]="22"
            [local_port]="3306"
            [remote_port]="3306"
            [key_file]="~/.ssh/prod_key"
        )

EOF
}

# Load tunnel configurations
load_tunnel_config() {
    if [ -f "$TUNNEL_CONFIG_FILE" ]; then
        source "$TUNNEL_CONFIG_FILE"
    else
        log_warning "No tunnel configuration file found at $TUNNEL_CONFIG_FILE"
        log_info "Run '$0 create <name>' to create a tunnel configuration"
    fi
}

# Check if tunnel is running
is_tunnel_running() {
    local tunnel_name="$1"
    local local_port="${2:-3306}"
    
    if netstat -tlnp 2>/dev/null | grep -q ":$local_port "; then
        return 0
    else
        return 1
    fi
}

# Start SSH tunnel
start_tunnel() {
    local tunnel_name="$1"
    
    if [ -z "$tunnel_name" ]; then
        log_error "Tunnel name is required"
        show_usage
        exit 1
    fi
    
    # Get tunnel configuration
    local tunnel_var="TUNNEL_${tunnel_name^^}"
    if [ -z "${!tunnel_var}" ]; then
        log_error "Tunnel configuration '$tunnel_name' not found"
        log_info "Available tunnels:"
        list_tunnels
        exit 1
    fi
    
    # Parse tunnel configuration
    local host="${!tunnel_var[host]}"
    local user="${!tunnel_var[user]:-$DEFAULT_SSH_USER}"
    local port="${!tunnel_var[port]:-$DEFAULT_SSH_PORT}"
    local local_port="${!tunnel_var[local_port]:-$DEFAULT_LOCAL_PORT}"
    local remote_port="${!tunnel_var[remote_port]:-3306}"
    local key_file="${!tunnel_var[key_file]}"
    
    if [ -z "$host" ]; then
        log_error "Host not configured for tunnel '$tunnel_name'"
        exit 1
    fi
    
    # Check if tunnel is already running
    if is_tunnel_running "$tunnel_name" "$local_port"; then
        log_warning "Tunnel '$tunnel_name' is already running on port $local_port"
        return 0
    fi
    
    # Build SSH command
    local ssh_cmd="ssh"
    if [ -n "$key_file" ]; then
        ssh_cmd="$ssh_cmd -i $key_file"
    fi
    ssh_cmd="$ssh_cmd -L $local_port:localhost:$remote_port"
    ssh_cmd="$ssh_cmd -N -f"
    ssh_cmd="$ssh_cmd -o ExitOnForwardFailure=yes"
    ssh_cmd="$ssh_cmd -o ServerAliveInterval=60"
    ssh_cmd="$ssh_cmd -o ServerAliveCountMax=3"
    ssh_cmd="$ssh_cmd $user@$host -p $port"
    
    log_info "Starting tunnel '$tunnel_name'..."
    log_info "Command: $ssh_cmd"
    
    if eval "$ssh_cmd"; then
        log_success "Tunnel '$tunnel_name' started successfully"
        log_info "Local port: $local_port -> Remote: $host:$remote_port"
    else
        log_error "Failed to start tunnel '$tunnel_name'"
        exit 1
    fi
}

# Stop SSH tunnel
stop_tunnel() {
    local tunnel_name="$1"
    
    if [ -z "$tunnel_name" ]; then
        log_error "Tunnel name is required"
        show_usage
        exit 1
    fi
    
    # Get tunnel configuration
    local tunnel_var="TUNNEL_${tunnel_name^^}"
    if [ -z "${!tunnel_var}" ]; then
        log_error "Tunnel configuration '$tunnel_name' not found"
        exit 1
    fi
    
    local local_port="${!tunnel_var[local_port]:-$DEFAULT_LOCAL_PORT}"
    
    # Find and kill the tunnel process
    local pid=$(netstat -tlnp 2>/dev/null | grep ":$local_port " | awk '{print $7}' | cut -d'/' -f1)
    
    if [ -n "$pid" ]; then
        log_info "Stopping tunnel '$tunnel_name' (PID: $pid)..."
        if kill "$pid" 2>/dev/null; then
            log_success "Tunnel '$tunnel_name' stopped successfully"
        else
            log_error "Failed to stop tunnel '$tunnel_name'"
            exit 1
        fi
    else
        log_warning "Tunnel '$tunnel_name' is not running"
    fi
}

# Show tunnel status
show_status() {
    log_info "SSH Tunnel Status:"
    echo ""
    
    if [ ! -f "$TUNNEL_CONFIG_FILE" ]; then
        log_warning "No tunnel configuration file found"
        return
    fi
    
    # Get all tunnel variables
    local tunnels=$(grep -E "^declare -A TUNNEL_[A-Z_]+=" "$TUNNEL_CONFIG_FILE" | sed 's/declare -A TUNNEL_\([A-Z_]*\)=.*/\1/' | tr '[:upper:]' '[:lower:]')
    
    for tunnel in $tunnels; do
        local tunnel_var="TUNNEL_${tunnel^^}"
        if [ -n "${!tunnel_var}" ]; then
            local host="${!tunnel_var[host]}"
            local local_port="${!tunnel_var[local_port]:-$DEFAULT_LOCAL_PORT}"
            
            if is_tunnel_running "$tunnel" "$local_port"; then
                echo -e "  ${GREEN}✓${NC} $tunnel (port $local_port) - RUNNING"
            else
                echo -e "  ${RED}✗${NC} $tunnel (port $local_port) - STOPPED"
            fi
        fi
    done
}

# List configured tunnels
list_tunnels() {
    if [ ! -f "$TUNNEL_CONFIG_FILE" ]; then
        log_warning "No tunnel configuration file found"
        return
    fi
    
    log_info "Configured tunnels:"
    echo ""
    
    local tunnels=$(grep -E "^declare -A TUNNEL_[A-Z_]+=" "$TUNNEL_CONFIG_FILE" | sed 's/declare -A TUNNEL_\([A-Z_]*\)=.*/\1/' | tr '[:upper:]' '[:lower:]')
    
    for tunnel in $tunnels; do
        local tunnel_var="TUNNEL_${tunnel^^}"
        if [ -n "${!tunnel_var}" ]; then
            local host="${!tunnel_var[host]}"
            local user="${!tunnel_var[user]:-$DEFAULT_SSH_USER}"
            local local_port="${!tunnel_var[local_port]:-$DEFAULT_LOCAL_PORT}"
            local remote_port="${!tunnel_var[remote_port]:-3306}"
            
            echo "  $tunnel:"
            echo "    Host: $user@$host"
            echo "    Port: $local_port -> $remote_port"
            echo ""
        fi
    done
}

# Create new tunnel configuration
create_tunnel_config() {
    local tunnel_name="$1"
    
    if [ -z "$tunnel_name" ]; then
        log_error "Tunnel name is required"
        show_usage
        exit 1
    fi
    
    # Convert to uppercase for variable name
    local tunnel_var="TUNNEL_${tunnel_name^^}"
    
    log_info "Creating tunnel configuration for '$tunnel_name'..."
    
    # Create config file if it doesn't exist
    if [ ! -f "$TUNNEL_CONFIG_FILE" ]; then
        cat > "$TUNNEL_CONFIG_FILE" << 'EOF'
#!/bin/bash
# SSH Tunnel Configuration for Sorting Office
# This file contains tunnel configurations for production databases

EOF
        chmod +x "$TUNNEL_CONFIG_FILE"
    fi
    
    # Check if tunnel already exists
    if grep -q "TUNNEL_${tunnel_name^^}=" "$TUNNEL_CONFIG_FILE"; then
        log_warning "Tunnel configuration '$tunnel_name' already exists"
        return
    fi
    
    # Add new tunnel configuration
    cat >> "$TUNNEL_CONFIG_FILE" << EOF

# Tunnel configuration for $tunnel_name
declare -A $tunnel_var=(
    [host]="your-server.example.com"
    [user]="root"
    [port]="22"
    [local_port]="3306"
    [remote_port]="3306"
    [key_file]="~/.ssh/your_key"
)
EOF
    
    log_success "Tunnel configuration '$tunnel_name' created in $TUNNEL_CONFIG_FILE"
    log_info "Please edit the configuration with your actual server details"
}

# Main command dispatcher
main() {
    local command="$1"
    
    case "$command" in
        start)
            load_tunnel_config
            start_tunnel "$2"
            ;;
        stop)
            load_tunnel_config
            stop_tunnel "$2"
            ;;
        status)
            load_tunnel_config
            show_status
            ;;
        list)
            load_tunnel_config
            list_tunnels
            ;;
        create)
            create_tunnel_config "$2"
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

# Run main function
main "$@" 
