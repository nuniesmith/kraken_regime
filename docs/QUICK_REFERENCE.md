# Quick Reference Card

## 📚 Documentation Navigation

| Document | Purpose |
|----------|---------|
| [DOCUMENTATION.md](DOCUMENTATION.md) | Complete documentation index |
| [README.md](../README.md) | Project overview and concepts |
| **QUICK_REFERENCE.md** | **→ You are here: Command reference** |
| [SETUP_GUIDE.md](SETUP_GUIDE.md) | Detailed setup and integration |
| [USAGE.md](USAGE.md) | API reference and code examples |

---

## 🚀 Getting Started

```bash
# 1. Fetch data (do this first!)
cargo run --bin backtest_cli -- fetch --pair BTC/USD --days 90

# 2. Run tests
cargo test

# 3. Run backtest
cargo run --bin backtest_cli -- backtest --pair BTC/USD

# 4. Start paper trading
cargo run --bin paper_trade
```

## 📊 Data Commands

```bash
# Fetch data for multiple pairs
cargo run --bin backtest_cli -- fetch --pair BTC/USD --days 90 --timeframe 15
cargo run --bin backtest_cli -- fetch --pair ETH/USD --days 90
cargo run --bin backtest_cli -- fetch --pair SOL/USD --days 60

# List available data
cargo run --bin backtest_cli -- list

# Custom data directory
cargo run --bin backtest_cli -- fetch --pair BTC/USD --days 30 --data-dir ./my_data
```

## 🧪 Testing Commands

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific tests
cargo test regime          # Regime detection tests
cargo test indicator       # Indicator tests
cargo test backtest        # Backtest engine tests
cargo test cost            # Cost model tests

# Run only unit tests
cargo test --lib

# Run benchmarks
cargo bench
```

## 📈 Backtest Commands

```bash
# Basic backtest
cargo run --bin backtest_cli -- backtest --pair BTC/USD

# Full configuration
cargo run --bin backtest_cli -- backtest \
    --pair BTC/USD \
    --timeframe 15 \
    --capital 10000 \
    --risk 1.0 \
    --fees standard \
    --method ensemble \
    --export-trades

# Compare detection methods
cargo run --bin backtest_cli -- compare --pair BTC/USD
```

## 🔄 Walk-Forward Analysis

```bash
# Default settings
cargo run --bin backtest_cli -- walk-forward --pair BTC/USD

# Custom settings
cargo run --bin backtest_cli -- walk-forward \
    --pair BTC/USD \
    --train-days 30 \
    --test-days 7 \
    --trials 50 \
    --target sharpe
```

## 📄 Paper Trading

```bash
# Default (BTC/USD, ETH/USD)
cargo run --bin paper_trade

# With environment variables
TRADING_PAIRS=BTC/USD,ETH/USD,SOL/USD \
INITIAL_CAPITAL=10000 \
RISK_PER_TRADE=0.01 \
UPDATE_INTERVAL_SECS=60 \
RUST_LOG=info \
cargo run --bin paper_trade

# Debug mode
RUST_LOG=debug cargo run --bin paper_trade
```

## ⚙️ Configuration Options

### Fee Tiers
| Tier | Maker | Taker |
|------|-------|-------|
| standard | 0.16% | 0.26% |
| intermediate | 0.14% | 0.24% |
| pro | 0.10% | 0.20% |
| conservative | 0.20% | 0.30% |
| zero | 0% | 0% |

### Detection Methods
| Method | Flag | Description |
|--------|------|-------------|
| Indicators | `--method indicators` | Fast, rule-based |
| HMM | `--method hmm` | Statistical learning |
| Ensemble | `--method ensemble` | Combined (recommended) |

### Optimization Targets
| Target | Flag |
|--------|------|
| Sharpe Ratio | `--target sharpe` |
| Sortino Ratio | `--target sortino` |
| Calmar Ratio | `--target calmar` |
| Total Return | `--target return` |
| Profit Factor | `--target profit_factor` |
| Win Rate | `--target win_rate` |

## 📁 File Locations

```
your_project/
├── data/ohlc/               # Historical data
│   ├── BTC_USD_15m.csv
│   └── ETH_USD_15m.csv
├── paper_trades_*.csv       # Paper trading logs
└── trades_*.csv             # Backtest export
```

## 🔍 Interpreting Results

### Good Backtest Results
- Sharpe Ratio > 1.0
- Max Drawdown < 20%
- Win Rate > 50% with Profit Factor > 1.5
- Costs < 30% of gross profit

### Good Walk-Forward Results
- Efficiency Ratio > 0.5
- Consistency > 70%
- OOS Sharpe > 0.5

## ⚠️ Warnings

1. Always run walk-forward analysis before paper trading
2. Paper trade for 2-4 weeks before considering live
3. Never risk more than you can afford to lose
4. Past performance ≠ future results
