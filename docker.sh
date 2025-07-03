#!/bin/bash

# Sorting Office Docker Management Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
show_usage() {
    echo "🚀 Sorting Office Docker Management Script"
    echo "=========================================="
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  build          Build the Docker images"
    echo "  up             Start all services"
    echo "  down           Stop all services"
    echo "  restart        Restart all services"
    echo "  logs           Show logs from all services"
    echo "  logs-app       Show logs from application only"
    echo "  logs-db        Show logs from database only"
    echo "  shell          Open shell in application container"
    echo "  db-shell       Open MySQL shell"
    echo "  clean          Remove all containers and volumes"
    echo "  dev            Start development environment"
    echo "  dev-down       Stop development environment"
    echo "  status         Show status of all services"
    echo "  help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 build       # Build images"
    echo "  $0 up          # Start production environment"
    echo "  $0 dev         # Start development environment"
    echo "  $0 logs        # View all logs"
}

# Function to check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
}

# Function to check if Docker Compose is available
check_docker_compose() {
    if ! command -v docker compose > /dev/null 2>&1; then
        print_error "Docker Compose is not installed. Please install Docker Compose and try again."
        exit 1
    fi
}

# Function to build images
build_images() {
    print_status "Building Docker images..."
    docker compose build
    print_success "Images built successfully!"
}

# Function to start services
start_services() {
    print_status "Starting Sorting Office services..."
    docker compose up -d
    print_success "Services started successfully!"
    print_status "Application will be available at: http://localhost:3000"
    print_status "phpMyAdmin will be available at: http://localhost:8080"
    print_status "Default login: admin/admin"
}

# Function to stop services
stop_services() {
    print_status "Stopping Sorting Office services..."
    docker compose down
    print_success "Services stopped successfully!"
}

# Function to restart services
restart_services() {
    print_status "Restarting Sorting Office services..."
    docker compose restart
    print_success "Services restarted successfully!"
}

# Function to show logs
show_logs() {
    print_status "Showing logs from all services..."
    docker compose logs -f
}

# Function to show app logs
show_app_logs() {
    print_status "Showing application logs..."
    docker compose logs -f app
}

# Function to show database logs
show_db_logs() {
    print_status "Showing database logs..."
    docker compose logs -f db
}

# Function to open shell in app container
open_shell() {
    print_status "Opening shell in application container..."
    docker compose exec app /bin/bash
}

# Function to open database shell
open_db_shell() {
    print_status "Opening MySQL shell..."
    docker compose exec db mysql -u sortingoffice -psortingoffice sortingoffice
}

# Function to clean up
clean_up() {
    print_warning "This will remove all containers and volumes. Are you sure? (y/N)"
    read -r response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
        print_status "Cleaning up Docker resources..."
        docker compose down -v --remove-orphans
        docker system prune -f
        print_success "Cleanup completed!"
    else
        print_status "Cleanup cancelled."
    fi
}

# Function to start development environment
start_dev() {
    print_status "Starting development environment..."
    docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d
    print_success "Development environment started!"
    print_status "Application will be available at: http://localhost:3000"
    print_status "Database will be available at: localhost:3306"
}

# Function to stop development environment
stop_dev() {
    print_status "Stopping development environment..."
    docker compose -f docker-compose.yml -f docker-compose.dev.yml down
    print_success "Development environment stopped!"
}

# Function to show status
show_status() {
    print_status "Service Status:"
    docker compose ps
}

# Main script logic
main() {
    # Check prerequisites
    check_docker
    check_docker_compose

    # Parse command
    case "${1:-help}" in
        build)
            build_images
            ;;
        up)
            start_services
            ;;
        down)
            stop_services
            ;;
        restart)
            restart_services
            ;;
        logs)
            show_logs
            ;;
        logs-app)
            show_app_logs
            ;;
        logs-db)
            show_db_logs
            ;;
        shell)
            open_shell
            ;;
        db-shell)
            open_db_shell
            ;;
        clean)
            clean_up
            ;;
        dev)
            start_dev
            ;;
        dev-down)
            stop_dev
            ;;
        status)
            show_status
            ;;
        help|--help|-h)
            show_usage
            ;;
        *)
            print_error "Unknown command: $1"
            echo ""
            show_usage
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@" 
