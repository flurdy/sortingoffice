# Multi-stage build for Rust application
FROM rust:slim AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libmariadb-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Copy the full source code and other needed files before building
COPY src/ ./src/
COPY templates/ ./templates/
COPY migrations/ ./migrations/
COPY resources/ ./resources/

# Copy minimal Docker/test config as the default config
COPY config/config.docker.toml /app/config/config.toml

# Copy static assets
COPY static /app/static

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libmariadb3 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 sortingoffice

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/sortingoffice /app/sortingoffice

# Copy templates and migrations
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/resources /app/resources
COPY --from=builder /app/static /app/static
COPY --from=builder /app/config /app/config

# Change ownership to non-root user
RUN chown -R sortingoffice:sortingoffice /app

# Switch to non-root user
USER sortingoffice

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/ || exit 1

# Run the application
CMD ["./sortingoffice"] 
