# =============================================================================
# Kraken Trading Bot - Python Optimizer Service
# =============================================================================
# Multi-stage build for optimized image size
# =============================================================================

# Build stage for compiling dependencies
FROM python:3.11-slim AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    g++ \
    libffi-dev \
    && rm -rf /var/lib/apt/lists/*

# Create virtual environment
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Install Python dependencies
COPY docker/python/requirements.txt .
RUN pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir -r requirements.txt

# =============================================================================
# Runtime stage
# =============================================================================
FROM python:3.11-slim AS runtime

# Install runtime dependencies and timezone data
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    tzdata \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Set default timezone (can be overridden by TZ env var)
ENV TZ=America/New_York
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

# Copy virtual environment from builder
COPY --from=builder /opt/venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash optimizer

# Set working directory
WORKDIR /app

# Create data directories (will be overwritten by volume mounts)
# - /data/ohlc: CSV exports (legacy)
# - /data/db: SQLite database for OHLC storage
# - /data/optimized_params: Output optimized parameters
# - /data/backtest_results: Backtest result files
# - /data/logs: Log files
RUN mkdir -p /data/ohlc /data/db /data/optimized_params /data/backtest_results /data/logs && \
    chown -R optimizer:optimizer /app /data

# Copy application code (both main.py and data_collector.py)
COPY --chown=optimizer:optimizer src/optimizer/*.py ./

# Switch to non-root user
USER optimizer

# Environment variables with defaults
ENV DATA_DIR=/data \
    OPTIMIZE_ASSETS=BTC,ETH,SOL,FARTCOIN,DOGE,PEPE,OP,AVAX,XRP,ADA,LINK,DOT,WIF \
    OPTUNA_TRIALS=100 \
    OPTIMIZATION_INTERVAL=6h \
    MIN_DATA_DAYS=7 \
    PREFERRED_INTERVAL_MINUTES=60 \
    RUN_ON_START=true \
    DATA_COLLECTION_ENABLED=true \
    DATA_COLLECTION_INTERVAL_MINUTES=5 \
    PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1

# Health check - verify optimizer process is running and database exists
HEALTHCHECK --interval=5m --timeout=30s --start-period=120s --retries=3 \
    CMD python -c "import os, sys; db_exists = os.path.exists('/data/db/ohlc.db'); log_exists = os.path.exists('/data/logs/optimizer.log') or os.path.exists('/data/optimizer.log'); sys.exit(0 if (db_exists or log_exists) else 1)" || exit 1

# Run the optimizer service
ENTRYPOINT ["python", "main.py"]
