# Kraken Regime-Aware Trading Bot

A Rust-based algorithmic trading system that automatically detects market regimes and switches between appropriate strategies.

## 📚 Documentation

This project has comprehensive documentation to help you get started and integrate the system:

| Document | Purpose |
|----------|---------|
| **[DOCUMENTATION.md](docs/DOCUMENTATION.md)** | 📑 **Start here** - Complete documentation index and navigation guide |
| **[QUICK_REFERENCE.md](docs/QUICK_REFERENCE.md)** | ⚡ Quick command reference and cheat sheet |
| **[SETUP_GUIDE.md](docs/SETUP_GUIDE.md)** | 🛠️ Detailed setup, integration, and configuration guide |
| **[USAGE.md](docs/USAGE.md)** | 💻 API reference and code examples for library integration |
| **[DOCKER_GUIDE.md](docs/DOCKER_GUIDE.md)** | 🐳 Docker setup and container deployment |
| **[RASPBERRY_PI_SETUP.md](docs/RASPBERRY_PI_SETUP.md)** | 🥧 Raspberry Pi compatibility and optimization guide |
| **[PROJECT_REVIEW.md](docs/PROJECT_REVIEW.md)** | 🏗️ Technical architecture and codebase review |

**Quick Start (Native):**
```bash
# 1. Fetch data
cargo run --bin backtest_cli -- fetch --pair BTC/USD --days 90

# 2. Run tests
cargo test

# 3. Run backtest
cargo run --bin backtest_cli -- backtest --pair BTC/USD
```

**Quick Start (Docker):**
```bash
# 1. Create .env file with your config
cp .env.example .env

# 2. Start services
docker compose up -d

# 3. View logs
docker compose logs -f
```

For detailed instructions, see [SETUP_GUIDE.md](docs/SETUP_GUIDE.md).

## 🎯 Overview

Based on the research showing that **regime-aware strategies outperform static ones by 20-40%**, this system:

1. **Detects Market Regime** using multiple methods (Indicators, HMM, or Ensemble)
2. **Selects Strategy** based on current conditions
3. **Manages Risk** with dynamic position sizing

| Regime | Detection | Strategy | Position Size |
|--------|-----------|----------|---------------|
| **Trending** | ADX > 25, EMA alignment | Trend Following (Golden Cross) | 100% |
| **Mean-Reverting** | ADX < 20, price in BB range | Mean Reversion (Bollinger Bands) | 100% |
| **Volatile** | High ATR, wide BB | Mean Reversion | 50% |
| **Uncertain** | Low confidence | No Trade | 0% |

## 🔬 Detection Methods

### 1. Technical Indicators (Default)
Fast, rule-based detection using:
- **ADX** - Trend strength (>25 = trending, <20 = ranging)
- **Bollinger Bands** - Volatility and price extremes
- **ATR** - Volatility expansion
- **EMA Alignment** - Trend direction

```rust
use kraken_regime::strategy::EnhancedRouter;
let router = EnhancedRouter::with_indicators();
```

### 2. Hidden Markov Model (HMM)
Statistical approach that learns regime distributions from returns:
- No assumptions about indicator values
- Learns mean/variance for each regime
- Provides transition probabilities
- Predicts expected regime duration

```rust
let router = EnhancedRouter::with_hmm();
```

**Based on**: Hamilton (1989) "A New Approach to Economic Analysis of Nonstationary Time Series"

### 3. Ensemble (Recommended)
Combines both methods for robustness:
- **Higher confidence** when methods agree
- **Lower confidence** when they disagree
- Best of both: fast response + statistical validation

```rust
let router = EnhancedRouter::with_ensemble();  // Recommended!
```

**Based on**: Horvath et al. (2021) "Clustering Market Regimes Using Wasserstein Distance"

## 📁 Project Structure

```
kraken_regime/
├── src/
│   ├── lib.rs                    # Main library entry
│   ├── regime/                   # Regime detection module
│   │   ├── mod.rs
│   │   ├── types.rs              # MarketRegime, RegimeConfig
│   │   ├── indicators.rs         # ADX, ATR, Bollinger Bands, EMA
│   │   ├── detector.rs           # Indicator-based detector
│   │   ├── hmm.rs                # Hidden Markov Model detector
│   │   └── ensemble.rs           # Ensemble (combines both)
│   ├── strategy/                 # Trading strategies
│   │   ├── mod.rs
│   │   ├── mean_reversion.rs     # Bollinger Bands strategy
│   │   ├── router.rs             # Original strategy router
│   │   └── enhanced_router.rs    # Router with HMM/Ensemble support
│   └── integration/              # Kraken exchange integration
│       ├── mod.rs
│       └── kraken.rs             # WebSocket & REST API
├── examples/
│   ├── live_trading.rs           # Live trading example
│   ├── backtest.rs               # Backtesting example
│   └── compare_methods.rs        # Compare Indicators vs HMM vs Ensemble
└── Cargo.toml
```

## 🧠 HMM Details

The Hidden Markov Model learns:
- **State distributions**: Mean and variance of returns in each regime
- **Transition matrix**: Probability of moving between regimes
- **Expected duration**: How long each regime typically lasts

```rust
// HMM provides extra insights
let signal = router.update("BTC/USD", high, low, close)?;

// Probability distribution over states
if let Some(probs) = signal.state_probabilities {
    println!("State probs: Bull={:.1}%, Bear={:.1}%, Volatile={:.1}%",
             probs[0] * 100.0, probs[1] * 100.0, probs[2] * 100.0);
}

// Expected regime duration
if let Some(duration) = signal.expected_duration {
    println!("Expected to stay in this regime for {:.0} more bars", duration);
}
```

## 🔗 Integration with Your Existing Codebase

### Step 1: Add to Your Project

Add this as a module in your existing kraken project:

```bash
# Copy the src files to your project
cp -r src/regime /path/to/your/kraken/src/
cp -r src/strategy /path/to/your/kraken/src/
cp -r src/integration /path/to/your/kraken/src/
```

Or add as a dependency:

```toml
# In your Cargo.toml
[dependencies]
kraken_regime = { path = "../kraken_regime" }
```

### Step 2: Integrate with Your WebSocket Handler

In your existing `src/websocket.rs`, add regime detection:

```rust
use kraken_regime::{KrakenRegimeTrader, KrakenIntegrationConfig, Candle};

// In your WebSocket handler struct
pub struct WebSocketHandler {
    // ... your existing fields ...
    regime_trader: KrakenRegimeTrader,
}

impl WebSocketHandler {
    pub fn new() -> Self {
        let config = KrakenIntegrationConfig {
            pairs: vec!["BTC/USD".into(), "ETH/USD".into(), "SOL/USD".into()],
            timeframe_minutes: 15,
            live_trading: false,  // Start with signal-only
            ..Default::default()
        };
        
        Self {
            // ... your existing initialization ...
            regime_trader: KrakenRegimeTrader::new(config),
        }
    }
    
    // In your OHLC message handler
    fn handle_ohlc(&mut self, pair: &str, ohlc: &OhlcData) {
        let candle = Candle {
            timestamp: ohlc.time,
            open: ohlc.open,
            high: ohlc.high,
            low: ohlc.low,
            close: ohlc.close,
            volume: ohlc.volume,
        };
        
        if let Some(action) = self.regime_trader.process_candle(pair, &candle) {
            match action.action {
                TradeType::Buy => {
                    // Use your existing buy logic
                    // action.size_factor adjusts position size based on regime
                    self.execute_buy(pair, action.size_factor);
                }
                TradeType::Sell => {
                    self.execute_sell(pair);
                }
                TradeType::Hold => {}
            }
        }
    }
}
```

### Step 3: Integrate with Your Strategy Module

Update your `src/strategy/mod.rs`:

```rust
// Add regime-aware routing
pub mod regime_router;

// Your existing strategies
pub mod golden_cross;
pub mod ema_pullback;
pub mod mean_reversion;  // New!

use kraken_regime::MarketRegime;

pub fn select_strategy(regime: MarketRegime) -> Box<dyn Strategy> {
    match regime {
        MarketRegime::Trending(_) => Box::new(golden_cross::GoldenCross::new()),
        MarketRegime::MeanReverting => Box::new(mean_reversion::MeanReversion::new()),
        MarketRegime::Volatile => Box::new(mean_reversion::MeanReversion::conservative()),
        MarketRegime::Uncertain => Box::new(NoTrade),
    }
}
```

### Step 4: Update Your Main Loop

In `src/main.rs`:

```rust
use kraken_regime::prelude::*;

#[tokio::main]
async fn main() {
    // Your existing initialization...
    
    // Add regime trader
    let config = KrakenIntegrationConfig::default();
    let mut regime_trader = KrakenRegimeTrader::new(config);
    
    // Warmup with historical data
    for pair in ["BTC/USD", "ETH/USD", "SOL/USD"] {
        let candles = fetch_historical_candles(pair, 500).await;
        regime_trader.warmup_with_history(pair, &candles);
    }
    
    // Your existing WebSocket loop with regime integration...
}
```

## 📊 Understanding the Regime Detector

### Indicators Used

1. **ADX (Average Directional Index)** - Measures trend strength
   - ADX > 25: Strong trend
   - ADX < 20: Weak/no trend (ranging)

2. **Bollinger Bands** - Measures volatility and price extremes
   - Width percentile > 75%: High volatility
   - Price at bands: Potential reversal

3. **ATR (Average True Range)** - Measures volatility
   - ATR expansion > 1.5x average: Volatile market

4. **EMA Alignment** - Confirms trend direction
   - Price > EMA50 > EMA200: Bullish
   - Price < EMA50 < EMA200: Bearish

### Configuration

```rust
let config = RegimeConfig {
    adx_period: 14,
    adx_trending_threshold: 25.0,    // ADX > this = trending
    adx_ranging_threshold: 20.0,     // ADX < this = ranging
    
    bb_period: 20,
    bb_std_dev: 2.0,
    bb_width_volatility_threshold: 75.0,  // percentile
    
    atr_period: 14,
    atr_expansion_threshold: 1.5,    // vs average
    
    min_regime_duration: 5,          // bars before switching
    regime_stability_bars: 3,        // filter whipsaws
    
    ..Default::default()
};
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific tests
cargo test regime
cargo test backtest
```

For detailed testing guide, see [docs/SETUP_GUIDE.md](docs/SETUP_GUIDE.md#running-tests).

## 📈 Expected Performance

Based on the article's research and typical results:

| Metric | Static Strategy | Regime-Aware |
|--------|-----------------|--------------|
| Win Rate | 40-50% | 50-60% |
| Max Drawdown | -30 to -50% | -15 to -25% |
| Sharpe Ratio | 0.5-1.0 | 1.0-1.5 |

**Key Benefits:**
- Avoids trend-following in ranging markets (reduces whipsaws)
- Avoids mean-reversion in trending markets (avoids "catching falling knives")
- Reduces position size in volatile/uncertain conditions
- Better risk-adjusted returns

## 🔧 Configuration Options

The system is highly configurable. See [docs/QUICK_REFERENCE.md](docs/QUICK_REFERENCE.md#-configuration-options) for all options.

Key configuration areas:
- **Regime Detection** - ADX, Bollinger Bands, ATR thresholds
- **Strategy Routing** - Position sizing, confidence thresholds
- **Trading Costs** - Fee tiers, slippage models
- **Risk Management** - Position limits, stop loss settings

For detailed configuration reference, see [docs/SETUP_GUIDE.md](docs/SETUP_GUIDE.md#configuration-reference).

## ⚠️ Risk Warnings

1. **Backtest ≠ Live Results** - Always paper trade first
2. **Markets Change** - Regime detection isn't perfect
3. **Slippage & Fees** - Account for real trading costs
4. **No Holy Grail** - This improves odds, not guarantees

## 📚 Research References

Based on concepts from:
- "The Most Profitable Algorithmic Trading Strategies" article
- Turtle Trading methodology (trend-following)
- Bollinger Bands mean reversion
- ADX trend strength indicator
- Regime-switching models in quantitative finance

## 🚀 Next Steps

1. **Get Started**: Follow the [SETUP_GUIDE.md](docs/SETUP_GUIDE.md) for detailed setup
2. **Learn Commands**: Bookmark [QUICK_REFERENCE.md](docs/QUICK_REFERENCE.md) for quick lookups
3. **Integrate**: See [USAGE.md](docs/USAGE.md) for API and integration examples
4. **Backtest**: Validate your strategy before paper trading
5. **Paper Trade**: Test with real data for 2-4 weeks before going live

**Complete Documentation**: See [DOCUMENTATION.md](docs/DOCUMENTATION.md) for the full documentation index.

---

## 📚 Complete Documentation

This README provides an overview. For detailed information:

- **[📑 DOCUMENTATION.md](docs/DOCUMENTATION.md)** - Complete documentation index (start here if lost)
- **[⚡ QUICK_REFERENCE.md](docs/QUICK_REFERENCE.md)** - All commands and options at a glance
- **[🛠️ SETUP_GUIDE.md](docs/SETUP_GUIDE.md)** - Step-by-step setup and integration (780 lines)
- **[💻 USAGE.md](docs/USAGE.md)** - API reference and code examples (683 lines)
- **[🐳 DOCKER_GUIDE.md](docs/DOCKER_GUIDE.md)** - Docker setup and deployment guide
- **[🥧 RASPBERRY_PI_SETUP.md](docs/RASPBERRY_PI_SETUP.md)** - Raspberry Pi 4B setup and optimization
- **[🏗️ PROJECT_REVIEW.md](docs/PROJECT_REVIEW.md)** - Technical architecture review

---

Built to integrate with your existing Kraken trading bot. Happy trading! 🦑
