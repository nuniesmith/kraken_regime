# syntax=docker/dockerfile:1.7
# =============================================================================
# Kraken Regime Trading Bot - Minimal Dockerfile
# =============================================================================
# This is a minimal Dockerfile for the library-only project
# For full application deployment, additional binaries need to be implemented
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

# Copy source code
COPY src ./src
COPY examples ./examples

# Build the library and tests (no binaries yet)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release --lib

# Create a simple health check binary for Docker
RUN echo 'fn main() { println!("Kraken Regime Library - No binary target configured"); std::process::exit(0); }' > /tmp/health.rs && \
    rustc /tmp/health.rs -o /tmp/health-check

# =============================================================================
# Runtime stage - minimal image
# =============================================================================
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    tzdata \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Set default timezone
ENV TZ=America/New_York
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

# Create app user
RUN useradd -m -u 1000 -s /bin/bash appuser

# Set working directory
WORKDIR /app

# Copy library from builder
COPY --from=builder /app/target/release/libkraken_regime.rlib /app/lib/
COPY --from=builder /tmp/health-check /app/health-check

# Create data directories
RUN mkdir -p /app/data /app/logs && \
    chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Set environment variables
ENV RUST_LOG=info \
    RUST_BACKTRACE=1

# Default command - show library info
CMD ["/app/health-check"]
