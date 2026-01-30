# Kraken Regime Library - Usage Guide

**Status:** ✅ Production-Ready Library  
**Version:** 0.1.0  
**Type:** Rust Library (not a standalone application)

---

## 📚 Documentation Navigation

| Document | Purpose |
|----------|---------|
| [DOCUMENTATION.md](DOCUMENTATION.md) | Complete documentation index |
| [README.md](../README.md) | Project overview and concepts |
| [QUICK_REFERENCE.md](QUICK_REFERENCE.md) | Command reference and cheat sheet |
| [SETUP_GUIDE.md](SETUP_GUIDE.md) | Detailed setup and integration |
| **USAGE.md** | **→ You are here: API reference** |

---

## 🎯 What This Library Does

This is a **Rust library** that provides regime-aware trading intelligence for cryptocurrency markets. It detects market conditions (trending, ranging, volatile) and recommends appropriate trading strategies.

**Key Features:**
- 🔍 Multi-method regime detection (Indicators, HMM, Ensemble)
- 🎯 Automatic strategy selection based on market conditions
- 📊 Real-time signal generation
- 🔧 Kraken exchange integration ready
- ⚡ High-performance Rust implementation

---

## 📦 Installation

### Add to Your Cargo.toml

```toml
[dependencies]
kraken_regime = { path = "../kraken_regime" }

# Or if published to crates.io:
# kraken_regime = "0.1.0"
```

### Required Dependencies

The library already includes:
- `tokio` - Async runtime
- `serde` - Serialization
- `reqwest` - HTTP client
- `chrono` - Time handling

---

## 🚀 Quick Start

### 1. Basic Regime Detection

```rust
use kraken_regime::regime::RegimeDetector;

fn main() {
    // Create detector with crypto-optimized settings
    let mut detector = RegimeDetector::crypto_optimized();
    
    // Feed it market data (high, low, close)
    let result = detector.update(50000.0, 49500.0, 49800.0);
    
    println!("Regime: {:?}", result.regime);
    println!("Confidence: {:.1}%", result.confidence * 100.0);
}
```

### 2. Strategy Router (Auto-Switching)

```rust
use kraken_regime::strategy::router::StrategyRouter;

fn main() {
    let mut router = StrategyRouter::default();
    
    // Register your trading pair
    router.register_asset("BTC/USD");
    
    // Update with market data
    if let Some(signal) = router.update("BTC/USD", 50000.0, 49500.0, 49800.0) {
        println!("Strategy: {:?}", signal.strategy);
        println!("Signal: {:?}", signal.signal);
        println!("Reason: {}", signal.reason);
    }
}
```

### 3. Full Trading Integration

```rust
use kraken_regime::prelude::*;

#[tokio::main]
async fn main() {
    // Configure the trader
    let config = KrakenIntegrationConfig {
        pairs: vec!["BTC/USD".into(), "ETH/USD".into()],
        timeframe_minutes: 15,
        live_trading: false,  // Signal-only mode
        ..Default::default()
    };
    
    let mut trader = KrakenRegimeTrader::new(config);
    
    // Process candles from your data source
    let candle = Candle {
        timestamp: 1234567890,
        open: 49900.0,
        high: 50100.0,
        low: 49700.0,
        close: 50000.0,
        volume: 123.45,
    };
    
    if let Some(action) = trader.process_candle("BTC/USD", &candle) {
        println!("Trade Action: {:?}", action.action);
        println!("Size Factor: {:.2}", action.size_factor);
        println!("Regime: {}", action.regime);
        println!("Reason: {}", action.reason);
    }
}
```

---

## 🧠 Detection Methods

### 1. Technical Indicators (Fast)

Best for: Real-time trading, quick decisions

```rust
use kraken_regime::regime::RegimeDetector;

let mut detector = RegimeDetector::crypto_optimized();
let result = detector.update(high, low, close);
```

**Uses:**
- ADX (trend strength)
- Bollinger Bands (volatility)
- ATR (volatility expansion)
- EMA alignment (trend direction)

### 2. Hidden Markov Model (Statistical)

Best for: Statistical analysis, learning from data

```rust
use kraken_regime::regime::{HMMRegimeDetector, HMMConfig};

let mut hmm = HMMRegimeDetector::new(HMMConfig::crypto_optimized());
let result = hmm.update_ohlc(high, low, close);

// Extra insights
println!("State probabilities: {:?}", hmm.state_probabilities());
println!("Expected duration: {:.0} bars", 
    hmm.expected_regime_duration(hmm.current_state_index()));
```

### 3. Ensemble (Recommended)

Best for: Production use, robust decisions

```rust
use kraken_regime::regime::EnsembleRegimeDetector;

let mut ensemble = EnsembleRegimeDetector::default_config();
let result = ensemble.update(high, low, close);

println!("Methods agree: {}", result.methods_agree);
println!("Confidence: {:.1}%", result.confidence * 100.0);
```

---

## 📊 Working with Signals

### Signal Types

```rust
pub enum Signal {
    Buy,    // Enter long position
    Sell,   // Exit position or enter short
    Hold,   // Do nothing
}
```

### Trade Actions

```rust
pub struct TradeAction {
    pub symbol: String,
    pub action: TradeType,
    pub price: f64,
    pub size_factor: f64,      // 0.0-1.0 (reduces in volatile markets)
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub source_strategy: String,
    pub regime: String,
    pub confidence: f64,
    pub reason: String,
}
```

### Example: Acting on Signals

```rust
match action.action {
    TradeType::Buy => {
        let position_size = max_position * action.size_factor;
        println!("BUY {} at {} (size: {})", 
            action.symbol, action.price, position_size);
        
        if let Some(sl) = action.stop_loss {
            println!("Stop Loss: {}", sl);
        }
    }
    TradeType::Sell => {
        println!("SELL {} at {}", action.symbol, action.price);
    }
    TradeType::Hold => {
        // Do nothing
    }
}
```

---

## 🔧 Integration Examples

### With Your Existing Kraken Bot

```rust
// In your WebSocket handler
use kraken_regime::prelude::*;

pub struct MyTradingBot {
    // Your existing fields...
    regime_trader: KrakenRegimeTrader,
}

impl MyTradingBot {
    pub fn new() -> Self {
        let config = KrakenIntegrationConfig::default();
        
        Self {
            // Your existing initialization...
            regime_trader: KrakenRegimeTrader::new(config),
        }
    }
    
    pub fn handle_candle(&mut self, pair: &str, ohlc: &OhlcData) {
        let candle = Candle {
            timestamp: ohlc.timestamp,
            open: ohlc.open,
            high: ohlc.high,
            low: ohlc.low,
            close: ohlc.close,
            volume: ohlc.volume,
        };
        
        if let Some(action) = self.regime_trader.process_candle(pair, &candle) {
            // Use your existing trading logic
            self.execute_trade(action);
        }
    }
}
```

### With Discord Notifications

```rust
use kraken_regime::prelude::*;
use reqwest::Client;

async fn send_discord_alert(action: &TradeAction, webhook_url: &str) {
    let client = Client::new();
    
    let message = format!(
        "🔔 **Trading Signal**\n\
         Symbol: {}\n\
         Action: {:?}\n\
         Price: ${:.2}\n\
         Regime: {}\n\
         Confidence: {:.1}%\n\
         Reason: {}",
        action.symbol,
        action.action,
        action.price,
        action.regime,
        action.confidence * 100.0,
        action.reason
    );
    
    let payload = serde_json::json!({
        "content": message
    });
    
    let _ = client.post(webhook_url)
        .json(&payload)
        .send()
        .await;
}
```

### With Redis State Persistence

```rust
use redis::{Client, Commands};
use kraken_regime::prelude::*;

fn save_regime_state(redis: &Client, pair: &str, regime: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = redis.get_connection()?;
    
    let key = format!("regime:{}", pair);
    conn.set(&key, regime)?;
    conn.expire(&key, 300)?; // 5 minutes
    
    Ok(())
}

fn get_regime_state(redis: &Client, pair: &str) -> Option<String> {
    let mut conn = redis.get_connection().ok()?;
    conn.get(format!("regime:{}", pair)).ok()
}
```

---

## ⚙️ Configuration

### Regime Detection Config

```rust
use kraken_regime::regime::RegimeConfig;

let config = RegimeConfig {
    // ADX settings
    adx_period: 14,
    adx_trending_threshold: 25.0,
    adx_ranging_threshold: 20.0,
    
    // Bollinger Bands
    bb_period: 20,
    bb_std_dev: 2.0,
    bb_width_volatility_threshold: 75.0,
    
    // ATR
    atr_period: 14,
    atr_expansion_threshold: 1.5,
    
    // Regime stability
    min_regime_duration: 5,
    regime_stability_bars: 3,
    
    ..Default::default()
};
```

### Strategy Router Config

```rust
use kraken_regime::strategy::router::StrategyRouterConfig;

let config = StrategyRouterConfig {
    regime_config: RegimeConfig::crypto_optimized(),
    
    // Position sizing
    volatile_position_size_factor: 0.5,  // 50% in volatile markets
    min_regime_confidence: 0.5,          // Need 50% confidence
    
    ..Default::default()
};
```

---

## 📈 Market Regimes

### Regime Types

```rust
pub enum MarketRegime {
    Trending(TrendDirection),  // Strong directional move
    MeanReverting,             // Oscillating around mean
    Volatile,                  // Large unpredictable swings
    Uncertain,                 // Low confidence
}

pub enum TrendDirection {
    Bullish,   // Uptrend
    Bearish,   // Downtrend
}
```

### Typical Characteristics

| Regime | ADX | BB Width | Best Strategy |
|--------|-----|----------|---------------|
| Trending | >25 | Medium | Trend Following |
| Mean-Reverting | <20 | Low | Bollinger Bands |
| Volatile | Any | High | Reduced Size or Cash |
| Uncertain | 20-25 | Medium | No Trade |

---

## 🧪 Testing

### Run Tests

```bash
# All tests
cargo test --lib

# Specific module
cargo test --lib regime::

# With output
cargo test --lib -- --nocapture
```

### Example Test

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bull_market_detection() {
        let mut detector = RegimeDetector::default();
        
        // Feed uptrending data
        let mut price = 100.0;
        for _ in 0..50 {
            price *= 1.01;  // 1% gain each bar
            detector.update(price * 1.005, price * 0.995, price);
        }
        
        let result = detector.update(price * 1.005, price * 0.995, price);
        
        match result.regime {
            MarketRegime::Trending(TrendDirection::Bullish) => {
                assert!(result.confidence > 0.5);
            }
            _ => panic!("Expected bullish trend"),
        }
    }
}
```

---

## 📚 API Reference

### Main Modules

```rust
// Regime detection
use kraken_regime::regime::{
    RegimeDetector,
    HMMRegimeDetector,
    EnsembleRegimeDetector,
    MarketRegime,
    RegimeConfig,
};

// Strategy routing
use kraken_regime::strategy::{
    router::StrategyRouter,
    mean_reversion::MeanReversionStrategy,
};

// Kraken integration
use kraken_regime::integration::{
    KrakenRegimeTrader,
    KrakenIntegrationConfig,
    Candle,
    TradeAction,
};

// Convenience re-exports
use kraken_regime::prelude::*;
```

---

## 🎓 Best Practices

### 1. Warmup Period

Always allow for warmup (50-100 bars) before trusting signals:

```rust
let mut detector = RegimeDetector::default();
let warmup_bars = 50;

for (i, candle) in historical_data.iter().enumerate() {
    let result = detector.update(candle.high, candle.low, candle.close);
    
    if i >= warmup_bars {
        // Only act on signals after warmup
        process_signal(result);
    }
}
```

### 2. Confidence Thresholds

Don't trade on low-confidence signals:

```rust
if result.confidence > 0.6 {  // 60% confidence minimum
    execute_trade(signal);
} else {
    log::info!("Low confidence, skipping trade");
}
```

### 3. Position Sizing

Respect the size factor from volatile regimes:

```rust
let base_position = 1000.0;  // $1000
let actual_position = base_position * action.size_factor;

// In volatile markets, size_factor might be 0.5 (50%)
// In stable trends, size_factor is 1.0 (100%)
```

### 4. Multiple Timeframes

Consider running detection on multiple timeframes:

```rust
let mut detector_15m = RegimeDetector::default();
let mut detector_1h = RegimeDetector::default();
let mut detector_4h = RegimeDetector::default();

// Only trade when all timeframes align
if all_timeframes_bullish {
    execute_buy();
}
```

---

## 🐛 Troubleshooting

### Issue: No signals generated

**Solution:** Check warmup period and confidence thresholds

```rust
// Lower confidence threshold for testing
let config = StrategyRouterConfig {
    min_regime_confidence: 0.3,  // Lower from default 0.5
    ..Default::default()
};
```

### Issue: Too many regime changes

**Solution:** Increase stability requirements

```rust
let config = RegimeConfig {
    min_regime_duration: 10,      // Increase from 5
    regime_stability_bars: 5,     // Increase from 3
    ..Default::default()
};
```

### Issue: Signals lag market

**Solution:** Use shorter indicator periods

```rust
let config = RegimeConfig {
    adx_period: 10,    // Decrease from 14
    bb_period: 15,     // Decrease from 20
    ..Default::default()
};
```

---

## 📊 Performance Tips

1. **Use `RegimeDetector` for real-time** - Fastest, rule-based
2. **Use `HMMRegimeDetector` for analysis** - More CPU intensive
3. **Use `EnsembleRegimeDetector` for production** - Best of both
4. **Cache detector instances** - Don't recreate every tick
5. **Use release builds** - Up to 100x faster than debug

```bash
cargo build --release
```

---

## 📝 Example Project Structure

```
my_trading_bot/
├── Cargo.toml
├── src/
│   ├── main.rs           # Your entry point
│   ├── websocket.rs      # Kraken WebSocket handler
│   ├── strategy.rs       # Your trading logic
│   └── lib.rs
└── kraken_regime/        # This library (git submodule or path)
    ├── Cargo.toml
    └── src/
```

**Cargo.toml:**
```toml
[package]
name = "my_trading_bot"
version = "0.1.0"
edition = "2021"

[dependencies]
kraken_regime = { path = "../kraken_regime" }
tokio = { version = "1", features = ["full"] }
```

---

## 🚀 Next Steps

1. **Run the tests** to see how it works:
   ```bash
   cargo test --lib
   ```

2. **Check the examples** (now fixed and working):
   ```bash
   cargo run --example compare_methods
   ```

3. **Integrate into your bot** using the examples above

4. **Paper trade first** - Always test with simulated funds

5. **Monitor and adjust** - Tune parameters for your specific markets

---

## 📞 Support

- **Issues:** Check the 24 passing unit tests for examples
- **Documentation:** Read inline docs with `cargo doc --open`
- **Examples:** See `examples/` directory for complete workflows

---

## ⚠️ Disclaimer

This library provides **signals only**. It does not:
- Execute trades automatically
- Guarantee profits
- Replace proper risk management
- Constitute financial advice

**Always:**
- Paper trade first
- Use proper position sizing
- Set stop losses
- Never risk more than you can afford to lose

---

**Happy Trading! 🦑📈**