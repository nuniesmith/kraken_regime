-- =============================================================================
-- Kraken Bot - Redis Instance Registry Initialization Script
-- =============================================================================
-- This Lua script initializes the instance registry structure in Redis.
-- It creates the necessary keys and data structures for tracking bot instances.
--
-- Usage: redis-cli --eval init-instance-registry.lua
--
-- Redis Key Structure:
--   kraken:instances:registry       - Hash of all registered instances
--   kraken:instances:counter        - Auto-incrementing instance counter
--   kraken:instances:ports          - Hash tracking allocated ports
--   kraken:instances:{id}:config    - Hash with instance configuration
--   kraken:instances:{id}:status    - String with current status
--   kraken:instances:{id}:created   - Timestamp of creation
--   kraken:instances:{id}:updated   - Timestamp of last update
-- =============================================================================

-- Keys
local registry_key = "kraken:instances:registry"
local counter_key = "kraken:instances:counter"
local ports_key = "kraken:instances:ports"
local primary_instance_key = "kraken:instances:primary:config"

-- Initialize counter if not exists
if redis.call("EXISTS", counter_key) == 0 then
    redis.call("SET", counter_key, 0)
end

-- Initialize registry hash if not exists
if redis.call("EXISTS", registry_key) == 0 then
    redis.call("HSET", registry_key, "_initialized", os.time())
    redis.call("HSET", registry_key, "_version", "1.0.0")
end

-- Initialize ports allocation hash
if redis.call("EXISTS", ports_key) == 0 then
    -- Reserve primary instance ports
    redis.call("HSET", ports_key, "health:8080", "primary")
    redis.call("HSET", ports_key, "metrics:9091", "primary")
    redis.call("HSET", ports_key, "_next_health_port", 8081)
    redis.call("HSET", ports_key, "_next_metrics_port", 9101)
end

-- Register primary instance if not exists
if redis.call("EXISTS", primary_instance_key) == 0 then
    redis.call("HSET", primary_instance_key,
        "instance_id", "primary",
        "name", "Kraken Bot (Primary)",
        "container_name", "kraken-bot",
        "health_port", 8080,
        "metrics_port", 9091,
        "trading_pairs", "BTC/USD,ETH/USD,SOL/USD",
        "strategy_type", "pullback",
        "enable_dry_run", "true",
        "signal_only_mode", "true",
        "created_at", os.time(),
        "created_by", "system",
        "is_primary", "true"
    )

    -- Add to registry
    redis.call("HSET", registry_key, "primary", "active")

    -- Set status
    redis.call("SET", "kraken:instances:primary:status", "registered")
end

return "OK: Instance registry initialized"
