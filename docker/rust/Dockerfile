# syntax=docker/dockerfile:1.7
# =============================================================================
# Kraken Trading Bot - Rust Dockerfile
# =============================================================================
# Multi-stage build for optimized production image
# =============================================================================

# Build stage
FROM rust:1.92-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock* ./
COPY askama.toml ./

# Create dummy source to cache dependencies
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    mkdir -p src/bin && \
    echo "fn main() {println!(\"Placeholder for dependency caching\");}" > src/main.rs && \
    touch src/lib.rs && \
    echo "fn main() {}" > src/bin/discord_webhook.rs && \
    echo "fn main() {}" > src/bin/dashboard.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src
RUN ln -s /app/src/dashboard/templates /app/templates

# Build the application (touch to invalidate cache for main.rs)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    touch src/main.rs src/lib.rs src/bin/discord_webhook.rs src/bin/dashboard.rs && \
    cargo build --release && \
    strip target/release/kraken-trading-bot target/release/dashboard target/release/discord-webhook-adapter

# =============================================================================
# Runtime stage - minimal image
# =============================================================================
FROM debian:bookworm-slim

# Install runtime dependencies and timezone data
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    tzdata \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Set default timezone (can be overridden by TZ env var)
ENV TZ=America/New_York
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

# Create app user for security
RUN useradd -m -u 1000 -s /bin/bash appuser

# Set working directory
WORKDIR /app

# Copy binaries from builder
COPY --from=builder /app/target/release/kraken-trading-bot /app/kraken-trading-bot
COPY --from=builder /app/target/release/dashboard /app/dashboard
COPY --from=builder /app/target/release/discord-webhook-adapter /app/discord-webhook-adapter

# Create data directories
RUN mkdir -p /app/data /app/logs /data/ohlc /data/optimized_params && \
    chown -R appuser:appuser /app /data

# Switch to non-root user
USER appuser

# Health check using the HTTP health endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=15s --retries=3 \
    CMD curl -sf http://localhost:8080/health || exit 1

# Set environment variables
ENV RUST_LOG=info \
    RUST_BACKTRACE=1 \
    OHLC_DATA_DIR=/data/ohlc \
    OPTIMIZED_PARAMS_DIR=/data/optimized_params

# Run the application
ENTRYPOINT ["/app/kraken-trading-bot"]
