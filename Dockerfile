# Build stage
FROM rust:1.92 as builder

WORKDIR /app

# Install PostgreSQL client and build tools
RUN apt-get update && apt-get install -y \
    postgresql-client \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire workspace
COPY . .

# Build the application in release mode with sqlx offline mode
ENV SQLX_OFFLINE=true
RUN cargo build --release 

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    postgresql-client \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from builder
COPY --from=builder /app/target/release/server /app/server

# Copy migrations
COPY migrations /app/migrations

# Expose the default port (can be overridden with PORT env var)
EXPOSE 2607

# Set environment variables
ENV PORT=8080
ENV BIND_ADDR=0.0.0.0

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/health || exit 1

# Create entrypoint script
RUN echo '#!/bin/bash\n\
    set -e\n\
    \n\
    echo "Waiting for database..."\n\
    while ! pg_isready -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME"; do\n\
    sleep 1\n\
    done\n\
    \n\
    echo "Applying SQL migrations..."\n\
    for f in /app/migrations/*.sql; do\n\
    echo "Running $f"\n\
    psql -v ON_ERROR_STOP=1 "$DATABASE_URL" -f "$f"\n\
    done\n\
    \n\
    echo "Starting server..."\n\
    exec /app/server' > /app/entrypoint.sh && chmod +x /app/entrypoint.sh

# (sqlx-cli no longer needed; migrations applied via psql)

# Run the entrypoint script
ENTRYPOINT ["/app/entrypoint.sh"]
