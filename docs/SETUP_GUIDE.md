# Kraken Regime-Aware Trading System - Complete Setup Guide

## 📚 Documentation Navigation

| Document | Purpose |
|----------|---------|
| [DOCUMENTATION.md](DOCUMENTATION.md) | Complete documentation index |
| [README.md](../README.md) | Project overview and concepts |
| [QUICK_REFERENCE.md](QUICK_REFERENCE.md) | Command reference and cheat sheet |
| **SETUP_GUIDE.md** | **→ You are here: Detailed setup guide** |
| [USAGE.md](USAGE.md) | API reference and code examples |

---

## Table of Contents
1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Project Structure](#project-structure)
4. [Adding to Your Project](#adding-to-your-project)
5. [Fetching Historical Data](#fetching-historical-data)
6. [Running Tests](#running-tests)
7. [Running Backtests](#running-backtests)
8. [Walk-Forward Analysis](#walk-forward-analysis)
9. [Paper Trading Setup](#paper-trading-setup)
10. [Configuration Reference](#configuration-reference)
11. [Troubleshooting](#troubleshooting)

---

## Overview

This system provides:
- **Regime Detection**: Identifies market conditions (Trending, Mean-Reverting, Volatile)
- **Strategy Routing**: Automatically switches strategies based on detected regime
- **Data Management**: Fetch and store historical OHLCV data from Kraken
- **Backtesting**: Realistic simulation with fees and slippage
- **Walk-Forward Testing**: Out-of-sample validation to prevent overfitting
- **Paper Trading**: Test with real market data without risking capital

### Detection Methods

| Method | Description | Best For |
|--------|-------------|----------|
| **Indicators** | ADX, BB, ATR, EMA rules | Fast response, interpretable |
| **HMM** | Hidden Markov Model learns from returns | Statistical validation |
| **Ensemble** | Combines both methods | Production (recommended) |

### Strategies by Regime

| Regime | Strategy | Position Size |
|--------|----------|---------------|
| Trending (Bull/Bear) | Trend Following | 100% |
| Mean-Reverting | Bollinger Bands | 100% |
| Volatile | Conservative Mean Reversion | 50% |
| Uncertain | No Trade | 0% |

---

## Quick Start

### Step 1: Copy Files to Your Project

```bash
# From the kraken_regime directory, copy to your project
cp -r src/data /path/to/your/kraken/src/
cp -r src/backtest /path/to/your/kraken/src/
cp -r src/regime /path/to/your/kraken/src/
cp -r src/strategy /path/to/your/kraken/src/
cp -r src/integration /path/to/your/kraken/src/
cp src/tests.rs /path/to/your/kraken/src/
cp examples/* /path/to/your/kraken/examples/
```

### Step 2: Update Your Cargo.toml

Add these dependencies:

```toml
[dependencies]
# Required dependencies
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"
reqwest = { version = "0.11", features = ["json"] }
tokio-tungstenite = { version = "0.21", features = ["native-tls"] }
futures-util = "0.3"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
anyhow = "1.0"
clap = { version = "4.4", features = ["derive"] }
rand = "0.8"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.9"
criterion = { version = "0.5", features = ["html_reports"] }
```

### Step 3: Update Your lib.rs

```rust
// Add these module declarations
pub mod data;
pub mod backtest;
pub mod regime;
pub mod strategy;
pub mod integration;

#[cfg(test)]
mod tests;

// Re-export for convenience
pub use data::{KrakenDataFetcher, DataStorage, Timeframe, TradingPair};
pub use backtest::{Backtester, BacktestConfig, TradingCosts, WalkForwardAnalysis};
pub use regime::{RegimeDetector, HMMRegimeDetector, EnsembleRegimeDetector, MarketRegime};
pub use strategy::{StrategyRouter, EnhancedRouter, MeanReversionStrategy};
pub use integration::{KrakenRegimeTrader, Candle, TradeAction};
```

### Step 4: Fetch Data and Test

```bash
# Fetch 30 days of BTC/USD data
cargo run --example fetch_data

# Run backtest
cargo run --example backtest

# Run tests
cargo test
```

---

## Project Structure

```
your_project/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── data/                    # Data fetching and storage
│   │   ├── mod.rs
│   │   ├── fetcher.rs          # Kraken API data fetcher
│   │   ├── storage.rs          # CSV/binary storage
│   │   └── types.rs            # Candle, TradingPair, Timeframe
│   │
│   ├── backtest/               # Backtesting engine
│   │   ├── mod.rs
│   │   ├── engine.rs           # Main backtest runner
│   │   ├── costs.rs            # Fees and slippage models
│   │   ├── metrics.rs          # Performance calculations
│   │   └── walk_forward.rs     # Walk-forward analysis
│   │
│   ├── regime/                 # Regime detection
│   │   ├── mod.rs
│   │   ├── detector.rs         # Indicator-based detection
│   │   ├── hmm.rs              # Hidden Markov Model
│   │   ├── ensemble.rs         # Combined detection
│   │   ├── indicators.rs       # ADX, ATR, BB, EMA
│   │   └── types.rs            # MarketRegime enum
│   │
│   ├── strategy/               # Trading strategies
│   │   ├── mod.rs
│   │   ├── mean_reversion.rs   # Bollinger Bands strategy
│   │   ├── router.rs           # Original router
│   │   └── enhanced_router.rs  # Multi-method router
│   │
│   ├── integration/            # Kraken integration
│   │   ├── mod.rs
│   │   └── kraken.rs           # WebSocket/REST integration
│   │
│   ├── bin/
│   │   └── backtest_cli.rs     # CLI tool
│   │
│   └── tests.rs                # Test suite
│
├── examples/
│   ├── fetch_data.rs           # Data fetching example
│   ├── backtest.rs             # Backtesting example
│   ├── walk_forward.rs         # Walk-forward example
│   └── live_trading.rs         # Live trading example
│
└── data/
    └── ohlc/                   # Stored OHLCV data
        ├── BTC_USD_15m.csv
        ├── ETH_USD_15m.csv
        └── ...
```

---

## Fetching Historical Data

### Using the CLI

```bash
# Fetch 90 days of BTC/USD 15-minute data
cargo run --bin backtest_cli -- fetch --pair BTC/USD --days 90 --timeframe 15

# Fetch multiple pairs
cargo run --bin backtest_cli -- fetch --pair ETH/USD --days 60
cargo run --bin backtest_cli -- fetch --pair SOL/USD --days 60

# List available data
cargo run --bin backtest_cli -- list
```

### Programmatically

```rust
use kraken_regime::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fetcher = KrakenDataFetcher::new();
    let storage = DataStorage::with_data_dir("./data/ohlc");
    
    let pair = TradingPair::new("BTC", "USD");
    let timeframe = Timeframe::M15;
    
    // Fetch 30 days of data
    let candles = fetcher.fetch_all(&pair, timeframe, 30).await?;
    
    // Save to CSV
    storage.save_csv(&pair, timeframe, &candles)?;
    
    println!("Fetched {} candles", candles.len());
    Ok(())
}
```

### Supported Timeframes

| Code | Minutes | Description |
|------|---------|-------------|
| M1 | 1 | 1 minute |
| M5 | 5 | 5 minutes |
| M15 | 15 | 15 minutes (recommended) |
| M30 | 30 | 30 minutes |
| H1 | 60 | 1 hour |
| H4 | 240 | 4 hours |
| D1 | 1440 | 1 day |

---

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Specific Test Categories

```bash
# Unit tests only
cargo test --lib

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_regime_detector

# Run tests matching pattern
cargo test regime
cargo test backtest
cargo test indicator
```

### Test Categories

The test suite includes:

1. **Indicator Tests** - EMA, ATR, ADX, Bollinger Bands
2. **Regime Detection Tests** - Trending, ranging, volatile detection
3. **Strategy Tests** - Mean reversion, strategy routing
4. **Cost Model Tests** - Fee calculation, slippage simulation
5. **Integration Tests** - Full pipeline testing
6. **Property Tests** - Edge cases and extreme values

---

## Running Backtests

### Using the CLI

```bash
# Basic backtest
cargo run --bin backtest_cli -- backtest --pair BTC/USD

# With custom settings
cargo run --bin backtest_cli -- backtest \
    --pair BTC/USD \
    --timeframe 15 \
    --capital 10000 \
    --risk 1.0 \
    --fees standard \
    --method ensemble

# Export trades to CSV
cargo run --bin backtest_cli -- backtest --pair BTC/USD --export-trades
```

### Fee Tiers

| Tier | Maker Fee | Taker Fee | Description |
|------|-----------|-----------|-------------|
| standard | 0.16% | 0.26% | Default Kraken fees |
| intermediate | 0.14% | 0.24% | Higher volume |
| pro | 0.10% | 0.20% | Professional traders |
| conservative | 0.20% | 0.30% | Pessimistic estimate |
| zero | 0% | 0% | For testing only |

### Programmatically

```rust
use kraken_regime::prelude::*;

fn main() {
    // Load data
    let storage = DataStorage::with_data_dir("./data/ohlc");
    let pair = TradingPair::new("BTC", "USD");
    let candles = storage.load(&pair, Timeframe::M15).unwrap();
    
    // Configure backtest with realistic costs
    let config = BacktestConfig {
        initial_capital: 10000.0,
        costs: TradingCosts::kraken_standard(),
        risk_per_trade: 0.01,  // 1%
        max_position_size: 2500.0,
        min_position_size: 10.0,
        use_stops: true,
        ..Default::default()
    };
    
    // Run backtest
    let mut backtester = Backtester::new(config);
    let result = backtester.run("BTC/USD", &candles);
    
    // Print results
    result.metrics.print_summary();
    
    // Access individual metrics
    println!("Sharpe Ratio: {:.2}", result.metrics.sharpe_ratio);
    println!("Max Drawdown: {:.2}%", result.metrics.max_drawdown_pct);
    println!("Win Rate: {:.1}%", result.metrics.win_rate);
}
```

### Understanding Backtest Results

```
╔══════════════════════════════════════════════════════════════╗
║                  PERFORMANCE METRICS                         ║
╠══════════════════════════════════════════════════════════════╣
║ RETURNS                                                      ║
║   Total Return:         15.23% ($1,523.00)                  ║
║   CAGR:                 45.67%                               ║
╠══════════════════════════════════════════════════════════════╣
║ RISK-ADJUSTED                                                ║
║   Sharpe Ratio:          1.85    (> 1.0 is good)            ║
║   Sortino Ratio:         2.34    (> 2.0 is excellent)       ║
║   Calmar Ratio:          3.12                                ║
╠══════════════════════════════════════════════════════════════╣
║ DRAWDOWN                                                     ║
║   Max Drawdown:          8.45%   (< 20% is acceptable)      ║
║   Max DD Duration:       5.2 days                           ║
╠══════════════════════════════════════════════════════════════╣
║ TRADES                                                       ║
║   Total Trades:          47                                  ║
║   Win Rate:              57.4%   (> 50% with good R:R)      ║
║   Profit Factor:         1.89    (> 1.5 is good)            ║
╠══════════════════════════════════════════════════════════════╣
║ COSTS                                                        ║
║   Total Fees:            $234.56                             ║
║   Total Slippage:        $89.12                              ║
║   Costs % of Profit:     21.3%                               ║
╚══════════════════════════════════════════════════════════════╝
```

---

## Walk-Forward Analysis

Walk-forward analysis is **critical** for validating your strategy won't overfit.

### Using the CLI

```bash
cargo run --bin backtest_cli -- walk-forward \
    --pair BTC/USD \
    --train-days 30 \
    --test-days 7 \
    --trials 50 \
    --target sharpe
```

### Programmatically

```rust
use kraken_regime::prelude::*;
use kraken_regime::backtest::walk_forward::OptimizationTarget;

fn main() {
    let storage = DataStorage::with_data_dir("./data/ohlc");
    let pair = TradingPair::new("BTC", "USD");
    let candles = storage.load(&pair, Timeframe::M15).unwrap();
    
    // Configure walk-forward
    let wf_config = WalkForwardConfig {
        train_size: 30 * 96,    // 30 days at 15-min bars
        test_size: 7 * 96,      // 7 days
        step_size: 7 * 96,      // Roll forward 7 days
        warmup_bars: 200,
        optimization_trials: 50,
        optimization_target: OptimizationTarget::SharpeRatio,
    };
    
    let bt_config = BacktestConfig::default();
    let wf = WalkForwardAnalysis::new(wf_config, bt_config);
    
    let result = wf.run("BTC/USD", &candles);
    result.print_summary();
}
```

### Interpreting Results

| Metric | Good | OK | Poor |
|--------|------|-----|------|
| Efficiency Ratio | > 0.5 | 0.3-0.5 | < 0.3 |
| Consistency Score | > 70% | 50-70% | < 50% |

- **Efficiency Ratio** = OOS Return / IS Return
  - High ratio means strategy generalizes well
  - Low ratio indicates overfitting

- **Consistency Score** = % of test windows that were profitable
  - High = reliable across different periods
  - Low = strategy is unpredictable

---

## Paper Trading Setup

### Step 1: Configure Environment

Create `.env` file:

```bash
# Kraken API (read-only for paper trading)
KRAKEN_API_KEY=your_api_key
KRAKEN_API_SECRET=your_api_secret

# Trading Configuration
TRADING_PAIRS=BTC/USD,ETH/USD,SOL/USD
TIMEFRAME_MINUTES=15
INITIAL_CAPITAL=10000

# Paper trading mode (IMPORTANT!)
ENABLE_DRY_RUN=true
SIGNAL_ONLY_MODE=true

# Risk Management
RISK_PER_TRADE=0.01
MAX_POSITION_SIZE=2500
STOP_LOSS_PCT=0.02
TAKE_PROFIT_PCT=0.05

# Logging
RUST_LOG=info
```

### Step 2: Create Paper Trading Script

```rust
// src/bin/paper_trade.rs
use kraken_regime::prelude::*;
use std::time::Duration;
use tokio::time::interval;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    
    // Configuration
    let pairs = vec!["BTC/USD", "ETH/USD", "SOL/USD"];
    let timeframe = Timeframe::M15;
    let initial_capital = 10000.0;
    
    // Initialize components
    let fetcher = KrakenDataFetcher::new();
    let storage = DataStorage::with_data_dir("./data/ohlc");
    
    // Create router with ensemble detection (recommended)
    let mut router = EnhancedRouter::with_ensemble();
    
    // Paper trading state
    let mut paper_capital = initial_capital;
    let mut paper_position: Option<PaperPosition> = None;
    
    // Warmup with historical data
    println!("📊 Warming up with historical data...\n");
    for pair in &pairs {
        let trading_pair = TradingPair::parse(pair).unwrap();
        
        // Fetch recent data for warmup
        match fetcher.fetch_all(&trading_pair, timeframe, 7).await {
            Ok(candles) => {
                router.register_asset(pair);
                for candle in &candles {
                    router.update(pair, candle.high, candle.low, candle.close);
                }
                println!("  ✓ {} warmed up with {} candles", pair, candles.len());
            }
            Err(e) => println!("  ✗ {} failed: {}", pair, e),
        }
    }
    
    println!("\n🚀 Starting paper trading...\n");
    println!("Initial capital: ${:.2}", paper_capital);
    println!("Pairs: {:?}", pairs);
    println!("Press Ctrl+C to stop\n");
    
    // Main trading loop
    let mut ticker = interval(Duration::from_secs(60));  // Check every minute
    
    loop {
        ticker.tick().await;
        
        for pair in &pairs {
            let trading_pair = TradingPair::parse(pair).unwrap();
            
            // Get current price
            match fetcher.get_ticker(&trading_pair).await {
                Ok(price) => {
                    // Simulate candle (use price for H/L/C)
                    let high = price * 1.001;
                    let low = price * 0.999;
                    
                    if let Some(signal) = router.update(pair, high, low, price) {
                        handle_signal(
                            pair,
                            price,
                            &signal,
                            &mut paper_capital,
                            &mut paper_position,
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to get {}: {}", pair, e);
                }
            }
        }
    }
}

#[derive(Debug)]
struct PaperPosition {
    pair: String,
    entry_price: f64,
    size_usd: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

fn handle_signal(
    pair: &str,
    price: f64,
    signal: &EnhancedSignal,
    capital: &mut f64,
    position: &mut Option<PaperPosition>,
) {
    use kraken_regime::strategy::mean_reversion::Signal;
    
    match signal.signal {
        Signal::Buy if position.is_none() => {
            let size = *capital * 0.01 * signal.position_factor;
            
            *position = Some(PaperPosition {
                pair: pair.to_string(),
                entry_price: price,
                size_usd: size,
                stop_loss: signal.stop_loss,
                take_profit: signal.take_profit,
            });
            
            println!(
                "📈 BUY {} @ ${:.2} | Size: ${:.2} | Regime: {:?} | Method: {:?}",
                pair, price, size, signal.regime, signal.detection_method
            );
        }
        Signal::Sell if position.is_some() => {
            if let Some(pos) = position.take() {
                let pnl = (price - pos.entry_price) / pos.entry_price * pos.size_usd;
                *capital += pnl;
                
                let emoji = if pnl > 0.0 { "💰" } else { "📉" };
                println!(
                    "{} SELL {} @ ${:.2} | P&L: ${:.2} | Capital: ${:.2}",
                    emoji, pair, price, pnl, capital
                );
            }
        }
        _ => {}
    }
    
    // Check stops on existing position
    if let Some(ref pos) = position {
        if let Some(stop) = pos.stop_loss {
            if price <= stop {
                let pnl = (stop - pos.entry_price) / pos.entry_price * pos.size_usd;
                *capital += pnl;
                println!("🛑 STOP HIT {} @ ${:.2} | P&L: ${:.2}", pair, stop, pnl);
                *position = None;
            }
        }
        if let Some(tp) = pos.take_profit {
            if price >= tp {
                let pnl = (tp - pos.entry_price) / pos.entry_price * pos.size_usd;
                *capital += pnl;
                println!("🎯 TARGET HIT {} @ ${:.2} | P&L: ${:.2}", pair, tp, pnl);
                *position = None;
            }
        }
    }
}
```

### Step 3: Add to Cargo.toml

```toml
[[bin]]
name = "paper_trade"
path = "src/bin/paper_trade.rs"
```

### Step 4: Run Paper Trading

```bash
# Start paper trading
cargo run --bin paper_trade

# With debug logging
RUST_LOG=debug cargo run --bin paper_trade
```

### Paper Trading Output

```
📊 Warming up with historical data...
  ✓ BTC/USD warmed up with 672 candles
  ✓ ETH/USD warmed up with 672 candles
  ✓ SOL/USD warmed up with 672 candles

🚀 Starting paper trading...
Initial capital: $10000.00
Pairs: ["BTC/USD", "ETH/USD", "SOL/USD"]
Press Ctrl+C to stop

📈 BUY BTC/USD @ $97234.50 | Size: $100.00 | Regime: Trending(Bullish) | Method: Ensemble
💰 SELL BTC/USD @ $98012.30 | P&L: $0.80 | Capital: $10000.80
📈 BUY ETH/USD @ $3456.78 | Size: $100.00 | Regime: MeanReverting | Method: Ensemble
🛑 STOP HIT ETH/USD @ $3387.65 | P&L: -$2.00 | Capital: $9998.80
```

---

## Configuration Reference

### BacktestConfig

```rust
BacktestConfig {
    initial_capital: 10000.0,     // Starting capital
    costs: TradingCosts::kraken_standard(),  // Fee structure
    risk_per_trade: 0.01,         // Risk 1% per trade
    max_position_size: 2500.0,    // Max $2500 per position
    min_position_size: 10.0,      // Min $10 per position
    use_stops: true,              // Use stop losses
    log_trades: false,            // Log individual trades
    router_config: Default,       // Strategy router config
}
```

### TradingCosts

```rust
TradingCosts {
    maker_fee: 0.0016,            // 0.16%
    taker_fee: 0.0026,            // 0.26%
    fixed_fee_usd: 0.0,           // Per-trade fixed fee
    slippage: SlippageModel::kraken_default(),
}
```

### SlippageModel

```rust
SlippageModel {
    base_spread: 0.0002,          // 0.02% base spread
    market_impact_coefficient: 0.00005,
    market_impact_exponent: 0.5,  // Square root model
    min_slippage_pct: 0.0001,
    max_slippage_pct: 0.005,      // Max 0.5%
    randomness: 0.2,              // 20% random variation
}
```

### RegimeConfig

```rust
RegimeConfig {
    adx_period: 14,
    adx_trending_threshold: 25.0,  // ADX > 25 = trending
    adx_ranging_threshold: 20.0,   // ADX < 20 = ranging
    atr_period: 14,
    atr_volatile_multiplier: 1.5,
    bb_period: 20,
    bb_std_dev: 2.0,
    bb_squeeze_threshold: 50.0,
    ema_short_period: 50,
    ema_long_period: 200,
    min_confidence: 0.6,
}
```

---

## Troubleshooting

### Common Issues

**1. "No data found" error**
```bash
# Fetch data first
cargo run --bin backtest_cli -- fetch --pair BTC/USD --days 30
```

**2. "Not enough data for walk-forward"**
- Need at least train_size + test_size + warmup_bars
- Fetch more historical data or reduce window sizes

**3. High costs eating profits**
- Check fee tier matches your Kraken account level
- Consider using limit orders (maker fees) in live trading

**4. Strategy not generating signals**
- Ensure warmup period has completed (200+ bars)
- Check if market regime is "Uncertain" (no signals generated)

**5. Backtest results don't match expectations**
- Verify slippage model is appropriate for your trade sizes
- Check that stops/TPs are being used correctly

### Performance Benchmarks

Expected execution times on modern hardware:

| Operation | 1K candles | 10K candles | 100K candles |
|-----------|------------|-------------|--------------|
| Backtest | <1s | ~2s | ~20s |
| Walk-forward (50 trials) | ~5s | ~30s | ~5min |
| Data fetch | ~5s | ~30s | ~5min |

### Getting Help

1. Check the test suite for examples: `cargo test -- --nocapture`
2. Enable debug logging: `RUST_LOG=debug cargo run ...`
3. Review the example files in `examples/`

---

## Next Steps

1. **Fetch Data**: Get 90+ days of historical data
2. **Run Backtest**: Verify positive expectancy
3. **Walk-Forward Test**: Confirm out-of-sample performance
4. **Paper Trade**: Test with real market data for 2-4 weeks
5. **Live Trade**: Start with minimal capital (optional, at your own risk)

**Remember**: Past performance does not guarantee future results. Always use proper risk management and never risk more than you can afford to lose.
