# Docker Guide

Complete guide for running Kraken Regime in Docker containers.

---

## 📋 **Overview**

This project includes a simplified Docker Compose setup that makes it easy to run the trading bot with all required services.

**What's Included:**
- ✅ **Redis** - Data caching and state management
- ✅ **Kraken Bot** - Main trading application
- ✅ **Persistent volumes** - Data survives container restarts
- ✅ **Health checks** - Automatic service monitoring
- ✅ **ARM64 support** - Works on Raspberry Pi

---

## 🚀 **Quick Start**

### Prerequisites

1. **Docker** installed (20.10+)
2. **Docker Compose** installed (2.0+)
3. **Git** (to clone the repository)

```bash
# Verify Docker installation
docker --version
docker compose version

# Should show:
# Docker version 20.10.x or higher
# Docker Compose version 2.x.x or higher
```

### Step 1: Clone Repository

```bash
cd ~
git clone https://github.com/yourusername/kraken_regime.git
cd kraken_regime
```

### Step 2: Create Environment File

```bash
cp .env.example .env
nano .env
```

Edit `.env` with your settings:

```bash
# Kraken API Credentials
KRAKEN_API_KEY=your_api_key_here
KRAKEN_API_SECRET=your_api_secret_here

# Trading Configuration
TRADING_PAIRS=BTC/USD,ETH/USD
INITIAL_CAPITAL=10000
RISK_PER_TRADE=0.01
UPDATE_INTERVAL_SECS=300

# Safety Settings (IMPORTANT!)
ENABLE_DRY_RUN=true
SIGNAL_ONLY_MODE=true

# Logging
RUST_LOG=info

# Timezone
TZ=America/New_York

# Ports (optional, defaults shown)
REDIS_PORT=6379
```

### Step 3: Build and Start

```bash
# Build images
docker compose build

# Start services
docker compose up -d

# View logs
docker compose logs -f
```

---

## 📁 **Project Structure**

```
kraken_regime/
├── docker-compose.yml          # Main Docker Compose file
├── docker/
│   └── rust/
│       └── Dockerfile          # Rust application container
├── .env                        # Your configuration (create this)
├── .env.example                # Example configuration
└── data/                       # Persistent data (auto-created)
    ├── ohlc/                   # Historical price data
    ├── params/                 # Optimized parameters
    └── logs/                   # Application logs
```

---

## 🔧 **Docker Compose Commands**

### Starting Services

```bash
# Start all services (detached)
docker compose up -d

# Start and view logs
docker compose up

# Start specific service
docker compose up -d redis
docker compose up -d kraken-bot
```

### Viewing Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f kraken-bot
docker compose logs -f redis

# Last 100 lines
docker compose logs --tail=100 kraken-bot

# Since timestamp
docker compose logs --since 2024-01-29T10:00:00 kraken-bot
```

### Stopping Services

```bash
# Stop all services
docker compose stop

# Stop specific service
docker compose stop kraken-bot

# Stop and remove containers
docker compose down

# Stop and remove everything (including volumes!)
docker compose down -v
```

### Restarting Services

```bash
# Restart all
docker compose restart

# Restart specific service
docker compose restart kraken-bot

# Restart after code changes
docker compose down
docker compose build
docker compose up -d
```

### Service Status

```bash
# Check status
docker compose ps

# Check resource usage
docker stats

# Check health
docker compose ps --format json | jq
```

---

## 🐳 **Service Details**

### Redis

**Purpose:** Caching and state management  
**Image:** `redis:7-alpine`  
**Port:** 6379  
**Memory:** 256MB max

**Access Redis CLI:**
```bash
docker compose exec redis redis-cli

# Inside redis-cli:
> PING
PONG
> KEYS *
> INFO memory
> EXIT
```

**Flush Redis (clear all data):**
```bash
docker compose exec redis redis-cli FLUSHALL
```

### Kraken Bot

**Purpose:** Main trading application  
**Build:** Custom Rust container  
**Memory:** 512MB max  
**CPU:** 1 core max

**Access shell:**
```bash
docker compose exec kraken-bot /bin/bash
```

**View environment:**
```bash
docker compose exec kraken-bot env
```

**Run commands inside container:**
```bash
# List data directory
docker compose exec kraken-bot ls -lh /app/data/

# Check Rust version
docker compose exec kraken-bot rustc --version

# View processes
docker compose exec kraken-bot ps aux
```

---

## 📊 **Data Persistence**

### Docker Volumes

All data is stored in named volumes that persist across container restarts:

```bash
# List volumes
docker volume ls | grep kraken

# Inspect volume
docker volume inspect kraken-bot-data

# Backup volume
docker run --rm -v kraken-bot-data:/data -v $(pwd):/backup \
  alpine tar czf /backup/kraken-data-backup.tar.gz -C /data .

# Restore volume
docker run --rm -v kraken-bot-data:/data -v $(pwd):/backup \
  alpine tar xzf /backup/kraken-data-backup.tar.gz -C /data
```

### Volume Locations

| Volume | Mount Point | Purpose |
|--------|-------------|---------|
| `kraken-bot-data` | `/app/data` | Historical data, configs |
| `kraken-bot-logs` | `/app/logs` | Application logs |
| `kraken-redis-data` | `/data` | Redis persistence |

### Accessing Data from Host

```bash
# Copy file from container
docker compose cp kraken-bot:/app/data/ohlc/BTC_USD_15m.csv ./

# Copy file to container
docker compose cp ./my-config.toml kraken-bot:/app/config/

# Access volume data directly (requires root)
sudo ls -lh /var/lib/docker/volumes/kraken-bot-data/_data/
```

---

## ⚙️ **Configuration**

### Environment Variables

Edit `.env` file or set in `docker-compose.yml`:

```yaml
environment:
  - RUST_LOG=debug           # Logging: trace, debug, info, warn, error
  - TRADING_PAIRS=BTC/USD    # Comma-separated pairs
  - INITIAL_CAPITAL=10000    # Starting capital in USD
  - RISK_PER_TRADE=0.01      # Risk 1% per trade
  - UPDATE_INTERVAL_SECS=300 # Update every 5 minutes
  - ENABLE_DRY_RUN=true      # Paper trading mode
```

### Resource Limits

Adjust in `docker-compose.yml`:

```yaml
deploy:
  resources:
    limits:
      cpus: "1.0"       # Max 1 CPU core
      memory: 512M      # Max 512MB RAM
    reservations:
      cpus: "0.5"       # Reserve 0.5 CPU
      memory: 256M      # Reserve 256MB RAM
```

### Port Mapping

Change exposed ports:

```yaml
ports:
  - "6380:6379"  # Map Redis to host port 6380
```

---

## 🔍 **Monitoring**

### Health Checks

Docker automatically monitors service health:

```bash
# Check health status
docker compose ps

# Should show "healthy" for Redis
# kraken-redis   redis:7-alpine   "docker-entrypoint..."   Up (healthy)
```

### Resource Usage

```bash
# Real-time stats
docker stats

# One-time snapshot
docker stats --no-stream
```

### Logs Analysis

```bash
# Search logs for errors
docker compose logs kraken-bot | grep -i error

# Count log levels
docker compose logs kraken-bot | grep -c INFO
docker compose logs kraken-bot | grep -c ERROR

# Export logs
docker compose logs kraken-bot > bot-logs.txt
```

---

## 🐛 **Troubleshooting**

### Issue: Container Won't Start

**Check logs:**
```bash
docker compose logs kraken-bot
```

**Common causes:**
- Missing `.env` file
- Invalid API keys
- Port already in use
- Insufficient resources

**Solution:**
```bash
# Verify .env exists
ls -la .env

# Check port conflicts
sudo netstat -tlnp | grep 6379

# Check Docker resources
docker info | grep -A 5 "CPUs\|Total Memory"
```

### Issue: Out of Memory

**Symptoms:** Container restarts frequently

**Check:**
```bash
docker stats

# Look for memory near limit
```

**Solution:**
```bash
# Increase memory limit in docker-compose.yml
limits:
  memory: 1024M  # Increase to 1GB

# Or reduce memory usage
environment:
  - TRADING_PAIRS=BTC/USD  # Use fewer pairs
```

### Issue: Build Fails

**Error:** "Cargo build failed" or "Out of space"

**Solution:**
```bash
# Clean Docker cache
docker system prune -a

# Build with more resources
docker compose build --no-cache

# Check disk space
df -h
```

### Issue: Redis Connection Failed

**Check Redis is running:**
```bash
docker compose ps redis

# Should show "Up (healthy)"
```

**Test connection:**
```bash
docker compose exec redis redis-cli PING
# Should return: PONG
```

**Restart Redis:**
```bash
docker compose restart redis
```

### Issue: Permission Denied

**Error:** "Cannot write to /app/data"

**Solution:**
```bash
# Fix volume permissions
docker compose exec kraken-bot chown -R appuser:appuser /app/data
```

---

## 🍓 **Raspberry Pi Specific**

### ARM64 Compatibility

The Docker setup works on Raspberry Pi 4B with ARM64 architecture.

### Installation on Pi

```bash
# Install Docker on Raspberry Pi OS / Ubuntu
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add your user to docker group
sudo usermod -aG docker $USER
newgrp docker

# Install Docker Compose
sudo apt install -y docker-compose-plugin

# Verify
docker --version
docker compose version
```

### Pi Optimization

Edit `docker-compose.yml` for Pi:

```yaml
kraken-bot:
  deploy:
    resources:
      limits:
        cpus: "0.75"      # Use 3 out of 4 cores
        memory: 512M      # Pi 4B has 2-8GB
      reservations:
        cpus: "0.25"
        memory: 256M
```

### Pi Performance

| Task | Time on Pi 4B |
|------|---------------|
| **First build** | 20-40 min |
| **Rebuild** | 3-7 min |
| **Container start** | 5-10 sec |
| **Runtime memory** | 200-400 MB |

**Tips:**
- Use SSD instead of SD card
- Enable swap (2GB+)
- Use active cooling
- Start with 1 trading pair

---

## 🔒 **Security**

### Best Practices

1. **Protect `.env` file:**
```bash
chmod 600 .env
echo ".env" >> .gitignore
```

2. **Use read-only API keys** (for paper trading)

3. **Run containers as non-root:**
```yaml
# Already configured in Dockerfile
USER appuser
```

4. **Limit network exposure:**
```yaml
# Don't expose ports to internet
ports:
  - "127.0.0.1:6379:6379"  # Bind to localhost only
```

5. **Keep images updated:**
```bash
docker compose pull
docker compose up -d
```

### Secrets Management

For production, use Docker secrets:

```yaml
secrets:
  kraken_api_key:
    file: ./secrets/api_key.txt
  kraken_api_secret:
    file: ./secrets/api_secret.txt

services:
  kraken-bot:
    secrets:
      - kraken_api_key
      - kraken_api_secret
```

---

## 🚀 **Advanced Usage**

### Multi-Environment Setup

```bash
# Development
docker compose -f docker-compose.yml -f docker-compose.dev.yml up

# Production (if you create docker-compose.prod.yml)
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

### Scaling Services

```bash
# Run multiple bot instances
docker compose up -d --scale kraken-bot=2

# Note: Requires different configurations per instance
```

### Custom Networks

```yaml
networks:
  kraken-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.28.0.0/16
```

### Build Arguments

```yaml
kraken-bot:
  build:
    context: .
    dockerfile: docker/rust/Dockerfile
    args:
      RUST_VERSION: "1.75"
      BUILD_MODE: "release"
```

---

## 📦 **Backup & Restore**

### Full Backup

```bash
# Create backup script
cat > backup-docker.sh << 'EOF'
#!/bin/bash
BACKUP_DIR=~/docker-backups/$(date +%Y%m%d_%H%M%S)
mkdir -p $BACKUP_DIR

# Backup volumes
docker run --rm \
  -v kraken-bot-data:/data \
  -v $BACKUP_DIR:/backup \
  alpine tar czf /backup/bot-data.tar.gz -C /data .

docker run --rm \
  -v kraken-redis-data:/data \
  -v $BACKUP_DIR:/backup \
  alpine tar czf /backup/redis-data.tar.gz -C /data .

# Backup configs
cp .env $BACKUP_DIR/
cp docker-compose.yml $BACKUP_DIR/

echo "Backup completed: $BACKUP_DIR"
EOF

chmod +x backup-docker.sh
./backup-docker.sh
```

### Restore

```bash
# Restore from backup
BACKUP_DIR=~/docker-backups/20240129_120000

docker compose down

docker run --rm \
  -v kraken-bot-data:/data \
  -v $BACKUP_DIR:/backup \
  alpine tar xzf /backup/bot-data.tar.gz -C /data

docker run --rm \
  -v kraken-redis-data:/data \
  -v $BACKUP_DIR:/backup \
  alpine tar xzf /backup/redis-data.tar.gz -C /data

docker compose up -d
```

---

## 🎯 **Production Checklist**

Before running in production:

- [ ] `.env` file configured correctly
- [ ] API keys tested and working
- [ ] `SIGNAL_ONLY_MODE=true` (start safe)
- [ ] Resource limits set appropriately
- [ ] Health checks configured
- [ ] Log rotation enabled
- [ ] Backup strategy in place
- [ ] Monitoring setup
- [ ] Firewall configured
- [ ] Docker volumes backed up
- [ ] Restart policies set
- [ ] Timezone configured

---

## 📚 **Additional Resources**

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Reference](https://docs.docker.com/compose/compose-file/)
- [Redis Docker Hub](https://hub.docker.com/_/redis)
- [Rust Docker Images](https://hub.docker.com/_/rust)

---

## 🆘 **Getting Help**

### Check Container Health

```bash
# Full health report
docker compose ps
docker compose logs --tail=50 kraken-bot
docker stats --no-stream
docker system df
```

### Debug Mode

```bash
# Run with debug logging
RUST_LOG=debug docker compose up

# Or edit .env:
RUST_LOG=debug

docker compose restart kraken-bot
```

### Clean Start

```bash
# Nuclear option - fresh start
docker compose down -v
docker system prune -a
docker volume prune
docker compose build --no-cache
docker compose up -d
```

---

## ✅ **Summary**

**Basic Commands:**
```bash
# Start
docker compose up -d

# Logs
docker compose logs -f

# Stop
docker compose down

# Restart
docker compose restart
```

**Your Docker setup is now ready!** 🐳

Start with `docker compose up -d` and monitor logs with `docker compose logs -f kraken-bot`.

---

*Compatible with Docker 20.10+, Docker Compose 2.0+, x86_64 and ARM64 (Raspberry Pi)*