#!/bin/bash
#
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Environment file (simplified - single .env)
ENV_FILE=".env"

# Compose file selection (dev vs prod)
# Use prod if PROD=1 or if docker-compose.prod.yml is explicitly requested
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.yml}"
if [ "${PROD:-0}" = "1" ] || [ -n "${USE_PROD_COMPOSE:-}" ]; then
    COMPOSE_FILE="docker-compose.prod.yml"
fi

# Check which docker compose command to use
if command -v docker-compose &> /dev/null; then
    COMPOSE_CMD="docker-compose -f $COMPOSE_FILE"
else
    COMPOSE_CMD="docker compose -f $COMPOSE_FILE"
fi

# Banner
show_banner() {
    echo -e "${BLUE}"
    echo "=================================================="
    echo "  🦑 Kraken Trading Bot - Simulation Mode"
    echo "  Generates signals for manual prop trading"
    echo "=================================================="
    echo -e "${NC}"
    if [ "$COMPOSE_FILE" = "docker-compose.prod.yml" ]; then
        echo -e "${MAGENTA}  Using: Production Compose (docker-compose.prod.yml)${NC}"
    else
        echo -e "${CYAN}  Using: Development Compose (docker-compose.yml)${NC}"
    fi
    echo ""
}

# Help message
show_help() {
    echo -e "${CYAN}Usage:${NC} ./run.sh <command> [options]"
    echo ""
    echo -e "${CYAN}Commands:${NC}"
    echo -e "  ${GREEN}start${NC}           Build and start all services (runs tests first by default)"
    echo -e "  ${GREEN}stop${NC}            Stop all services"
    echo -e "  ${GREEN}restart${NC}         Restart all services"
    echo -e "  ${GREEN}status${NC}          Show service status"
    echo -e "  ${GREEN}logs${NC} [service]  View logs (optional: kraken-bot, redis)"
    echo -e "  ${GREEN}build${NC}           Build/rebuild Docker images"
    echo -e "  ${GREEN}rebuild${NC}         Force rebuild Docker images (no cache)"
    echo -e "  ${GREEN}shell${NC} <service> Open shell in container"
    echo -e "  ${GREEN}redis-cli${NC}       Open Redis CLI"
    echo -e "  ${GREEN}clean${NC}           Stop and remove containers, networks"
    echo -e "  ${GREEN}purge${NC}           Clean + remove volumes (WARNING: deletes data)"
    echo -e "  ${GREEN}config${NC}          Validate and show docker-compose config"
    echo -e "  ${GREEN}health${NC}          Check health of all services"
    echo -e "  ${GREEN}env${NC}             Generate/regenerate .env file"
    echo -e "  ${GREEN}env-show${NC}        Show current .env configuration"
    echo -e "  ${GREEN}pull${NC}            Pull latest Docker images"
    echo ""
    echo -e "${CYAN}Test Commands:${NC}"
    echo -e "  ${GREEN}test${NC}            Run all Rust tests"
    echo -e "  ${GREEN}test-quick${NC}      Run tests without verbose output"
    echo -e "  ${GREEN}test-full${NC}       Run format check + clippy + build + tests"
    echo -e "  ${GREEN}test-watch${NC}      Run tests in watch mode (requires cargo-watch)"
    echo -e "  ${GREEN}fmt${NC}             Auto-format Rust code"
    echo -e "  ${GREEN}clippy${NC}          Run Clippy linter"
    echo ""
    echo -e "${CYAN}Environment Variables:${NC}"
    echo "  PROD=1                  Use production compose file"
    echo "  COMPOSE_FILE=<file>     Specify custom compose file"
    echo "  RUN_TESTS=0             Skip tests during start (default: 1)"
    echo "  TEST_VERBOSE=1          Show verbose test output (default: 0)"
    echo ""
    echo -e "${CYAN}Examples:${NC}"
    echo "  ./run.sh start          # Run tests, then build and start everything"
    echo "  RUN_TESTS=0 ./run.sh start  # Skip tests, just start"
    echo "  ./run.sh test           # Run all tests"
    echo "  ./run.sh test-full      # Full CI-style check"
    echo "  PROD=1 ./run.sh start   # Start using production compose"
    echo "  ./run.sh logs           # View all logs"
    echo "  ./run.sh env            # Generate .env file interactively"
    echo "  ./run.sh status         # Check service status"
    echo "  ./run.sh pull           # Pull latest images from Docker Hub"
    echo ""
}

# Generate .env file
generate_env_file() {
    local force="${1:-false}"

    if [ -f "$ENV_FILE" ] && [ "$force" != "true" ]; then
        echo -e "${GREEN}✓ .env file already exists${NC}"
        return 0
    fi

    echo -e "${CYAN}Generating .env file for simulation mode...${NC}"
    echo ""

    # Check for environment variables (from CI/CD or manual)
    local api_key="${KRAKEN_API_KEY:-}"
    local api_secret="${KRAKEN_API_SECRET:-}"
    local discord_webhook="${DISCORD_WEBHOOK_URL:-${DISCORD_WEBHOOK_KRAKEN:-}}"

    # If not in environment, prompt user
    if [ -z "$api_key" ]; then
        echo -e "${YELLOW}Enter your Kraken API Key (for market data access):${NC}"
        read -r api_key
    fi

    if [ -z "$api_secret" ]; then
        echo -e "${YELLOW}Enter your Kraken API Secret:${NC}"
        read -rs api_secret
        echo ""
    fi

    if [ -z "$discord_webhook" ]; then
        echo -e "${YELLOW}Enter Discord Webhook URL for trading signals (optional, press Enter to skip):${NC}"
        read -r discord_webhook
    fi

    # Generate the .env file
    cat > "$ENV_FILE" << ENVEOF
# =============================================================================
# Kraken Trading Bot - Simulation Mode Configuration
# =============================================================================
# This bot runs in SIMULATION MODE - no real trades are executed
# Generates trading signals for manual prop trading via Discord alerts
# =============================================================================

# API Credentials (for market data access only in sim mode)
KRAKEN_API_KEY=${api_key}
KRAKEN_API_SECRET=${api_secret}

# Trading Configuration
TRADING_PAIRS=BTC/USD,ETH/USD,SOL/USD
EMA_SHORT_PERIOD=50
EMA_LONG_PERIOD=200

# Risk Management (Simulation - \$5K virtual balance)
RISK_PER_TRADE_PERCENTAGE=1.0
MAX_POSITION_SIZE_USD=250.0
MIN_POSITION_SIZE_USD=10.0
MIN_ACCOUNT_BALANCE_USD=100.0
STOP_LOSS_PERCENTAGE=2.0
TAKE_PROFIT_PERCENTAGE=5.0
MAX_DAILY_TRADES=10
TRADE_COOLDOWN_SECONDS=180

# Portfolio Management
TARGET_BTC_ALLOCATION_PCT=40.0
TARGET_ETH_ALLOCATION_PCT=30.0
TARGET_SOL_ALLOCATION_PCT=20.0
TARGET_USD_ALLOCATION_PCT=10.0
PROFIT_HOLD_PERCENTAGE=10.0
REBALANCE_THRESHOLD_PCT=10.0
ENABLE_REBALANCING=false

# ===========================================
# SIMULATION MODE - No real trades executed
# ===========================================
ENABLE_DRY_RUN=true
SIGNAL_ONLY_MODE=true
SIMULATED_BALANCE_USD=5000.0

# System Configuration
LOG_LEVEL=info
HEALTH_CHECK_PORT=8080

# WebSocket Configuration
WS_RECONNECT_DELAY_SECS=5
WS_PING_INTERVAL_SECS=30

# Redis State Persistence
REDIS_URL=redis://redis:6379
REDIS_INSTANCE_ID=kraken-sim

# Discord Notifications (Trading Signals)
DISCORD_WEBHOOK_URL=${discord_webhook}
ENVEOF

    chmod 600 "$ENV_FILE"
    echo -e "${GREEN}✓ Created .env file (Simulation Mode)${NC}"
    echo ""
}

# Show current .env configuration
show_env_config() {
    if [ ! -f "$ENV_FILE" ]; then
        echo -e "${RED}No .env file found. Run './run.sh env' to generate one.${NC}"
        return 1
    fi

    echo -e "${CYAN}Current .env Configuration:${NC}"
    echo ""

    # Show non-sensitive values
    echo -e "${BLUE}Trading Pairs:${NC} $(grep '^TRADING_PAIRS=' "$ENV_FILE" | cut -d'=' -f2)"
    echo -e "${BLUE}Simulation Mode:${NC} $(grep '^ENABLE_DRY_RUN=' "$ENV_FILE" | cut -d'=' -f2)"
    echo -e "${BLUE}Signal Only:${NC} $(grep '^SIGNAL_ONLY_MODE=' "$ENV_FILE" | cut -d'=' -f2)"
    echo -e "${BLUE}Simulated Balance:${NC} \$$(grep '^SIMULATED_BALANCE_USD=' "$ENV_FILE" | cut -d'=' -f2)"
    echo ""

    # Check if API keys are configured
    if grep -q "^KRAKEN_API_KEY=.\+" "$ENV_FILE"; then
        echo -e "${GREEN}✓ Kraken API Key configured${NC}"
    else
        echo -e "${RED}✗ Kraken API Key not configured${NC}"
    fi

    if grep -q "^KRAKEN_API_SECRET=.\+" "$ENV_FILE"; then
        echo -e "${GREEN}✓ Kraken API Secret configured${NC}"
    else
        echo -e "${RED}✗ Kraken API Secret not configured${NC}"
    fi

    # Check Discord webhook
    local webhook=$(grep '^DISCORD_WEBHOOK_URL=' "$ENV_FILE" | cut -d'=' -f2)
    if [ -n "$webhook" ] && [ "$webhook" != "" ]; then
        echo -e "${GREEN}✓ Discord Webhook configured${NC}"
    else
        echo -e "${YELLOW}⚠ Discord Webhook not configured (no alerts)${NC}"
    fi

    echo ""
}

# Ensure Redis config exists in .env
ensure_redis_config() {
    if [ ! -f "$ENV_FILE" ]; then
        return
    fi

    if ! grep -q "^REDIS_URL=" "$ENV_FILE" 2>/dev/null; then
        echo "" >> "$ENV_FILE"
        echo "# Redis State Persistence" >> "$ENV_FILE"
        echo "REDIS_URL=redis://redis:6379" >> "$ENV_FILE"
        echo "REDIS_INSTANCE_ID=kraken-sim" >> "$ENV_FILE"
        echo -e "${GREEN}✓ Added Redis config to .env${NC}"
    fi
}

# Check environment configuration
check_environment() {
    echo -e "${CYAN}Checking environment configuration...${NC}"
    echo ""

    # Generate .env if it doesn't exist
    if [ ! -f "$ENV_FILE" ]; then
        echo -e "${YELLOW}No .env file found${NC}"
        generate_env_file
    fi

    # Ensure Redis config
    ensure_redis_config

    # Validate API credentials are present
    if ! grep -q "^KRAKEN_API_KEY=.\+" "$ENV_FILE" || ! grep -q "^KRAKEN_API_SECRET=.\+" "$ENV_FILE"; then
        echo -e "${RED}⚠️  API credentials not configured in .env${NC}"
        echo ""
        echo "Do you want to regenerate .env now? (y/n)"
        read -r response
        if [[ "$response" =~ ^[Yy]$ ]]; then
            generate_env_file true
        else
            echo -e "${RED}Exiting. Please configure .env before running.${NC}"
            exit 1
        fi
    fi

    echo -e "${GREEN}✓ Configuration valid${NC}"
    echo ""

    # Show mode
    if grep -q "^ENABLE_DRY_RUN=true" "$ENV_FILE" && grep -q "^SIGNAL_ONLY_MODE=true" "$ENV_FILE"; then
        echo -e "${GREEN}Mode: SIMULATION (Signal Generation Only)${NC}"
        echo -e "${BLUE}  No real trades will be executed${NC}"
        echo -e "${BLUE}  Trading signals sent to Discord${NC}"
    else
        echo -e "${RED}⚠️  WARNING: Not in simulation mode!${NC}"
        echo ""
        echo "Are you sure you want to continue? (yes/no)"
        read -r response
        if [[ ! "$response" =~ ^[Yy][Ee][Ss]$ ]]; then
            echo -e "${BLUE}Exiting. Set ENABLE_DRY_RUN=true and SIGNAL_ONLY_MODE=true in .env${NC}"
            exit 1
        fi
    fi

    echo ""
}

# ============================================================================
# Test Commands
# ============================================================================

# Run all tests
cmd_test() {
    echo -e "${BLUE}🧪 Running Rust Tests...${NC}"
    echo ""

    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        echo -e "${YELLOW}⚠ Cargo not found. Please install Rust toolchain.${NC}"
        echo -e "${BLUE}Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
        return 1
    fi

    local test_args="--workspace"

    # Add verbose flag if requested
    if [ "${TEST_VERBOSE:-0}" = "1" ]; then
        test_args="$test_args --verbose"
    fi

    # Run tests
    echo -e "${CYAN}Running: cargo test $test_args${NC}"
    echo ""

    if cargo test $test_args; then
        echo ""
        echo -e "${GREEN}✓ All tests passed!${NC}"
        return 0
    else
        echo ""
        echo -e "${RED}✗ Some tests failed${NC}"
        return 1
    fi
}

# Run tests quietly (for integration into start)
cmd_test_quick() {
    echo -e "${BLUE}🧪 Running Quick Tests...${NC}"

    if ! command -v cargo &> /dev/null; then
        echo -e "${YELLOW}⚠ Cargo not found, skipping tests${NC}"
        return 0
    fi

    if cargo test --workspace --quiet 2>&1; then
        echo -e "${GREEN}✓ All tests passed${NC}"
        return 0
    else
        echo -e "${RED}✗ Tests failed${NC}"
        return 1
    fi
}

# Run full test suite (CI-style)
cmd_test_full() {
    echo -e "${BLUE}🔧 Running Full Test Suite (CI-style)...${NC}"
    echo ""

    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}✗ Cargo not found. Please install Rust toolchain.${NC}"
        return 1
    fi

    local failed=0
    local total_start=$(date +%s)

    # 1. Auto-fix formatting first
    echo -e "${CYAN}Step 1/4: Auto-fixing formatting...${NC}"
    cargo fmt --all 2>/dev/null || true
    echo -e "${GREEN}✓ Formatting applied${NC}"
    echo ""

    # 2. Check formatting
    echo -e "${CYAN}Step 2/4: Checking formatting...${NC}"
    if cargo fmt --all -- --check; then
        echo -e "${GREEN}✓ Formatting check passed${NC}"
    else
        echo -e "${RED}✗ Formatting issues found (run: cargo fmt)${NC}"
        ((failed++))
    fi
    echo ""

    # 3. Run Clippy
    echo -e "${CYAN}Step 3/4: Running Clippy linter...${NC}"
    if cargo clippy --all-targets --all-features -- -D warnings 2>&1; then
        echo -e "${GREEN}✓ Clippy passed${NC}"
    else
        echo -e "${RED}✗ Clippy warnings/errors found${NC}"
        echo -e "${YELLOW}💡 Tip: Some issues can be auto-fixed with 'cargo clippy --fix'${NC}"
        ((failed++))
    fi
    echo ""

    # 4. Run tests
    echo -e "${CYAN}Step 4/4: Running tests...${NC}"
    if cargo test --workspace; then
        echo -e "${GREEN}✓ All tests passed${NC}"
    else
        echo -e "${RED}✗ Some tests failed${NC}"
        ((failed++))
    fi
    echo ""

    # Summary
    local total_end=$(date +%s)
    local duration=$((total_end - total_start))

    echo "=================================================="
    if [ $failed -eq 0 ]; then
        echo -e "${GREEN}✓ All checks passed! (${duration}s)${NC}"
        echo "=================================================="
        return 0
    else
        echo -e "${RED}✗ $failed check(s) failed (${duration}s)${NC}"
        echo "=================================================="
        return 1
    fi
}

# Run tests in watch mode
cmd_test_watch() {
    echo -e "${BLUE}🔄 Running Tests in Watch Mode...${NC}"
    echo ""

    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}✗ Cargo not found${NC}"
        return 1
    fi

    if ! command -v cargo-watch &> /dev/null; then
        echo -e "${YELLOW}cargo-watch not installed. Installing...${NC}"
        cargo install cargo-watch
    fi

    echo -e "${CYAN}Watching for changes... (Ctrl+C to stop)${NC}"
    echo ""
    cargo watch -x "test --workspace"
}

# Auto-format code
cmd_fmt() {
    echo -e "${BLUE}🔧 Formatting Rust code...${NC}"

    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}✗ Cargo not found${NC}"
        return 1
    fi

    if cargo fmt --all; then
        echo -e "${GREEN}✓ Code formatted${NC}"
        return 0
    else
        echo -e "${RED}✗ Formatting failed${NC}"
        return 1
    fi
}

# Run Clippy
cmd_clippy() {
    echo -e "${BLUE}📎 Running Clippy...${NC}"

    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}✗ Cargo not found${NC}"
        return 1
    fi

    if cargo clippy --all-targets --all-features -- -D warnings; then
        echo -e "${GREEN}✓ Clippy passed${NC}"
        return 0
    else
        echo -e "${RED}✗ Clippy found issues${NC}"
        echo -e "${YELLOW}💡 Try: cargo clippy --fix${NC}"
        return 1
    fi
}

# ============================================================================
# Docker Commands
# ============================================================================

# Build Docker images
cmd_build() {
    echo -e "${BLUE}Building Docker images...${NC}"
    $COMPOSE_CMD build "$@"
    echo -e "${GREEN}✓ Build complete${NC}"
}

# Force rebuild Docker images
cmd_rebuild() {
    echo -e "${BLUE}Rebuilding Docker images (no cache)...${NC}"
    $COMPOSE_CMD build --no-cache "$@"
    echo -e "${GREEN}✓ Rebuild complete${NC}"
}

# Pull Docker images
cmd_pull() {
    echo -e "${BLUE}Pulling latest Docker images...${NC}"
    $COMPOSE_CMD pull --ignore-pull-failures
    echo -e "${GREEN}✓ Pull complete${NC}"
}

# Start services
cmd_start() {
    show_banner

    # Run tests before start (unless disabled)
    if [ "${RUN_TESTS:-1}" = "1" ]; then
        echo -e "${CYAN}Running tests before start...${NC}"
        echo ""
        if ! cmd_test_quick; then
            echo ""
            echo -e "${RED}⚠️  Tests failed! Start anyway? (y/n)${NC}"
            read -r response
            if [[ ! "$response" =~ ^[Yy]$ ]]; then
                echo -e "${BLUE}Exiting. Fix tests and try again.${NC}"
                exit 1
            fi
        fi
        echo ""
    else
        echo -e "${YELLOW}⚠ Tests skipped (RUN_TESTS=0)${NC}"
        echo ""
    fi

    check_environment

    # Create necessary directories
    mkdir -p data logs

    # For production, pull images; for dev, always build
    if [ "$COMPOSE_FILE" = "docker-compose.prod.yml" ]; then
        echo -e "${BLUE}Pulling latest Docker images...${NC}"
        $COMPOSE_CMD pull --ignore-pull-failures || true
    else
        echo -e "${BLUE}Building Docker images...${NC}"
        $COMPOSE_CMD build
    fi

    # Start the services
    echo -e "${GREEN}Starting services...${NC}"
    $COMPOSE_CMD up -d

    echo ""
    echo -e "${GREEN}✓ Services started successfully!${NC}"
    echo ""

    # Show service status
    cmd_status

    echo ""
    echo -e "${CYAN}Useful commands:${NC}"
    echo -e "  ${BLUE}View logs:${NC}       ./run.sh logs"
    echo -e "  ${BLUE}Stop all:${NC}        ./run.sh stop"
    echo -e "  ${BLUE}Restart:${NC}         ./run.sh restart"
    echo -e "  ${BLUE}Check status:${NC}    ./run.sh status"
    echo -e "  ${BLUE}Check health:${NC}    ./run.sh health"
    echo -e "  ${BLUE}Run tests:${NC}       ./run.sh test"
    echo ""

    echo "View logs now? (y/n)"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        $COMPOSE_CMD logs -f
    fi
}

# Stop services
cmd_stop() {
    echo -e "${BLUE}Stopping all services...${NC}"
    $COMPOSE_CMD down
    echo -e "${GREEN}✓ Services stopped${NC}"
}

# Restart services
cmd_restart() {
    echo -e "${BLUE}Restarting services...${NC}"
    if [ -n "$1" ]; then
        $COMPOSE_CMD restart "$1"
        echo -e "${GREEN}✓ Service '$1' restarted${NC}"
    else
        $COMPOSE_CMD restart
        echo -e "${GREEN}✓ All services restarted${NC}"
    fi
}

# Show status
cmd_status() {
    echo -e "${CYAN}Service Status:${NC}"
    $COMPOSE_CMD ps
}

# View logs
cmd_logs() {
    if [ -n "$1" ]; then
        echo -e "${CYAN}Showing logs for: $1${NC}"
        $COMPOSE_CMD logs -f "$1"
    else
        echo -e "${CYAN}Showing logs for all services (Ctrl+C to exit)${NC}"
        $COMPOSE_CMD logs -f
    fi
}

# Open shell in container
cmd_shell() {
    local service="$1"
    if [ -z "$service" ]; then
        echo -e "${RED}Error: Please specify a service${NC}"
        echo "Available services: kraken-bot, redis"
        exit 1
    fi

    echo -e "${CYAN}Opening shell in $service container...${NC}"
    docker exec -it "$service" /bin/sh 2>/dev/null || docker exec -it "$service" /bin/bash
}

# Open Redis CLI
cmd_redis_cli() {
    echo -e "${CYAN}Opening Redis CLI...${NC}"
    docker exec -it kraken-redis redis-cli
}

# Clean up containers and networks
cmd_clean() {
    echo -e "${YELLOW}Stopping and removing containers, networks...${NC}"
    $COMPOSE_CMD down --remove-orphans
    echo -e "${GREEN}✓ Cleanup complete${NC}"
}

# Purge everything including volumes
cmd_purge() {
    echo -e "${RED}WARNING: This will delete all data including:${NC}"
    echo "  - Redis data (bot state)"
    echo "  - All logs"
    echo ""
    echo -e "${YELLOW}Are you sure you want to continue? (yes/no)${NC}"
    read -r response
    if [[ "$response" =~ ^[Yy][Ee][Ss]$ ]]; then
        echo -e "${RED}Purging all containers, networks, and volumes...${NC}"
        $COMPOSE_CMD down -v --remove-orphans
        echo -e "${GREEN}✓ Purge complete${NC}"
    else
        echo -e "${BLUE}Purge cancelled${NC}"
    fi
}

# Validate and show config
cmd_config() {
    echo -e "${CYAN}Validating docker-compose configuration...${NC}"
    $COMPOSE_CMD config
}

# Check health of services
cmd_health() {
    echo -e "${CYAN}Checking service health...${NC}"
    echo ""

    # Redis
    echo -e "${BLUE}Redis:${NC}"
    if docker exec kraken-redis redis-cli ping 2>/dev/null | grep -q "PONG"; then
        echo -e "  ${GREEN}✓ Healthy - PONG received${NC}"
    else
        echo -e "  ${RED}✗ Unhealthy or not running${NC}"
    fi
    echo ""

    # Bot
    echo -e "${BLUE}Kraken Bot:${NC}"
    if curl -sf http://localhost:8080/health 2>/dev/null; then
        echo -e "\n  ${GREEN}✓ Healthy${NC}"
    else
        if docker ps | grep -q "kraken"; then
            echo -e "  ${YELLOW}⚠ Running but health endpoint not responding${NC}"
        else
            echo -e "  ${RED}✗ Not running${NC}"
        fi
    fi
    echo ""

    # Show docker compose status
    echo -e "${CYAN}Container Status:${NC}"
    $COMPOSE_CMD ps
}

# ============================================================================
# Main command handler
# ============================================================================

main() {
    local cmd="${1:-}"
    shift 2>/dev/null || true

    case "$cmd" in
        # Service commands
        start)
            cmd_start "$@"
            ;;
        stop)
            cmd_stop "$@"
            ;;
        restart)
            cmd_restart "$@"
            ;;
        status|ps)
            cmd_status "$@"
            ;;
        logs|log)
            cmd_logs "$@"
            ;;
        build)
            cmd_build "$@"
            ;;
        rebuild)
            cmd_rebuild "$@"
            ;;
        shell|exec)
            cmd_shell "$@"
            ;;
        redis-cli|redis)
            cmd_redis_cli "$@"
            ;;
        clean)
            cmd_clean "$@"
            ;;
        purge)
            cmd_purge "$@"
            ;;
        config)
            cmd_config "$@"
            ;;
        health|check)
            cmd_health "$@"
            ;;
        pull)
            cmd_pull "$@"
            ;;
        env)
            generate_env_file true
            ;;
        env-show|show-env)
            show_env_config
            ;;
        # Test commands
        test)
            cmd_test "$@"
            ;;
        test-quick)
            cmd_test_quick "$@"
            ;;
        test-full|ci)
            cmd_test_full "$@"
            ;;
        test-watch|watch)
            cmd_test_watch "$@"
            ;;
        fmt|format)
            cmd_fmt "$@"
            ;;
        clippy|lint)
            cmd_clippy "$@"
            ;;
        # Help
        help|-h|--help)
            show_banner
            show_help
            ;;
        "")
            show_banner
            show_help
            ;;
        *)
            echo -e "${RED}Unknown command: $cmd${NC}"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

# Run main with all arguments
main "$@"
