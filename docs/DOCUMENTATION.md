# Kraken Regime Documentation Index

Welcome to the Kraken Regime-Aware Trading System documentation! This guide will help you find the information you need.

---

## 📚 Documentation Overview

This project includes several documentation files, each serving a specific purpose:

| Document | Purpose | Best For |
|----------|---------|----------|
| [README.md](../README.md) | Project overview, concepts, and research background | Understanding what the system does and how it works |
| [QUICK_REFERENCE.md](QUICK_REFERENCE.md) | Command cheat sheet and quick lookups | Daily usage, CLI commands, and quick answers |
| [SETUP_GUIDE.md](SETUP_GUIDE.md) | Detailed setup and integration instructions | First-time setup, integration, and configuration |
| [USAGE.md](USAGE.md) | API reference and code examples | Library integration and development |
| [PROJECT_REVIEW.md](PROJECT_REVIEW.md) | Technical analysis and architecture review | Understanding the codebase structure |

---

## 🚀 Getting Started

### New Users: Start Here

1. **Read [README.md](../README.md)** - Understand regime detection and why it matters
2. **Follow [SETUP_GUIDE.md](SETUP_GUIDE.md)** - Set up your environment step-by-step
3. **Bookmark [QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - Keep it handy for commands
4. **Run the Quick Start** from SETUP_GUIDE.md:
   ```bash
   # Fetch data
   cargo run --bin backtest_cli -- fetch --pair BTC/USD --days 90
   
   # Run tests
   cargo test
   
   # Run backtest
   cargo run --bin backtest_cli -- backtest --pair BTC/USD
   ```

### Experienced Traders

1. **[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - Commands and configuration
2. **[SETUP_GUIDE.md](SETUP_GUIDE.md#walk-forward-analysis)** - Advanced validation
3. **[SETUP_GUIDE.md](SETUP_GUIDE.md#paper-trading-setup)** - Paper trading setup

### Developers Integrating This Library

1. **[USAGE.md](USAGE.md)** - API reference and integration patterns
2. **[README.md](../README.md#integration-with-your-existing-codebase)** - Integration examples
3. **[PROJECT_REVIEW.md](PROJECT_REVIEW.md)** - Architecture deep dive

---

## 📖 Documentation by Topic

### Installation & Setup
- [Installation](SETUP_GUIDE.md#quick-start) - Add to your project
- [Dependencies](SETUP_GUIDE.md#step-2-update-your-cargotoml) - Required Cargo.toml entries
- [Project Structure](SETUP_GUIDE.md#project-structure) - File organization

### Data Management
- [Fetching Data](SETUP_GUIDE.md#fetching-historical-data) - Download historical OHLCV data
- [CLI Commands](QUICK_REFERENCE.md#-data-commands) - Data fetching commands
- [Programmatic Access](SETUP_GUIDE.md#programmatically) - Fetch data from code

### Regime Detection
- [Overview](../README.md#-detection-methods) - Three detection methods explained
- [Indicators Method](../README.md#1-technical-indicators-default) - Fast, rule-based
- [HMM Method](../README.md#2-hidden-markov-model-hmm) - Statistical learning
- [Ensemble Method](../README.md#3-ensemble-recommended) - Combined approach
- [Configuration](SETUP_GUIDE.md#regimeconfig) - Tuning detection parameters

### Trading Strategies
- [Strategy Routing](../README.md#-overview) - Automatic strategy selection
- [Mean Reversion](USAGE.md#3-full-trading-integration) - Bollinger Bands strategy
- [Position Sizing](QUICK_REFERENCE.md#-configuration-options) - Risk management

### Backtesting
- [Running Backtests](SETUP_GUIDE.md#running-backtests) - Test strategies on historical data
- [CLI Commands](QUICK_REFERENCE.md#-backtest-commands) - Backtest command reference
- [Understanding Results](SETUP_GUIDE.md#understanding-backtest-results) - Interpreting metrics
- [Cost Models](SETUP_GUIDE.md#tradingcosts) - Fees and slippage

### Walk-Forward Analysis
- [Overview](SETUP_GUIDE.md#walk-forward-analysis) - Out-of-sample validation
- [CLI Commands](QUICK_REFERENCE.md#-walk-forward-analysis) - Walk-forward commands
- [Interpreting Results](SETUP_GUIDE.md#interpreting-results) - Efficiency and consistency

### Paper Trading
- [Setup](SETUP_GUIDE.md#paper-trading-setup) - Risk-free testing with real data
- [Configuration](SETUP_GUIDE.md#step-1-configure-environment) - Environment setup
- [Running](QUICK_REFERENCE.md#-paper-trading) - Start paper trading
- [Output](SETUP_GUIDE.md#paper-trading-output) - Understanding results

### API Reference
- [Basic Usage](USAGE.md#-quick-start) - Simple examples
- [Detection Methods](USAGE.md#-detection-methods) - Using detectors
- [Integration](USAGE.md#-integration-examples) - Connect to your bot
- [Configuration](USAGE.md#-configuration) - All config options

### Testing
- [Running Tests](SETUP_GUIDE.md#running-tests) - Test suite
- [Test Categories](QUICK_REFERENCE.md#-testing-commands) - Unit, integration, benchmarks
- [Writing Tests](USAGE.md#example-test) - Add your own tests

### Configuration
- [Quick Reference](QUICK_REFERENCE.md#-configuration-options) - All options at a glance
- [Regime Config](SETUP_GUIDE.md#regimeconfig) - Detection parameters
- [Backtest Config](SETUP_GUIDE.md#backtestconfig) - Simulation settings
- [Trading Costs](SETUP_GUIDE.md#tradingcosts) - Fee structures
- [Slippage Model](SETUP_GUIDE.md#slippagemodel) - Slippage simulation

### Troubleshooting
- [Common Issues](SETUP_GUIDE.md#common-issues) - Frequent problems and solutions
- [Performance](SETUP_GUIDE.md#performance-benchmarks) - Expected execution times
- [Debugging](USAGE.md#-troubleshooting) - Debug techniques

---

## 🎯 Use Case Guides

### "I want to backtest a strategy"

1. **Fetch data**: [Data Commands](QUICK_REFERENCE.md#-data-commands)
2. **Run backtest**: [Backtest Commands](QUICK_REFERENCE.md#-backtest-commands)
3. **Interpret results**: [Understanding Results](SETUP_GUIDE.md#understanding-backtest-results)
4. **Validate**: [Walk-Forward Analysis](SETUP_GUIDE.md#walk-forward-analysis)

### "I want to integrate this into my bot"

1. **Understand the API**: [USAGE.md](USAGE.md)
2. **See integration examples**: [Integration Guide](../README.md#integration-with-your-existing-codebase)
3. **Configure**: [Configuration Reference](SETUP_GUIDE.md#configuration-reference)
4. **Test**: [Testing](USAGE.md#-testing)

### "I want to paper trade"

1. **Setup environment**: [Paper Trading Setup](SETUP_GUIDE.md#paper-trading-setup)
2. **Create script**: [Paper Trading Script](SETUP_GUIDE.md#step-2-create-paper-trading-script)
3. **Run**: [Paper Trading Commands](QUICK_REFERENCE.md#-paper-trading)
4. **Monitor**: [Output Guide](SETUP_GUIDE.md#paper-trading-output)

### "I need a quick command reference"

- **Go to**: [QUICK_REFERENCE.md](QUICK_REFERENCE.md)
- All CLI commands, options, and flags in one place

### "I want to understand the architecture"

1. **Project structure**: [Architecture](PROJECT_REVIEW.md)
2. **Module overview**: [Project Structure](SETUP_GUIDE.md#project-structure)
3. **Detection methods**: [Detection Deep Dive](../README.md#-detection-methods)

---

## 📊 Command Quick Links

### Most Common Commands

```bash
# Fetch data
cargo run --bin backtest_cli -- fetch --pair BTC/USD --days 90

# Run tests
cargo test

# Basic backtest
cargo run --bin backtest_cli -- backtest --pair BTC/USD

# Walk-forward analysis
cargo run --bin backtest_cli -- walk-forward --pair BTC/USD

# Paper trading
cargo run --bin paper_trade
```

**Full command reference**: [QUICK_REFERENCE.md](QUICK_REFERENCE.md)

---

## 🔍 Search by Keyword

### Regime Detection
- [Detection Methods](../README.md#-detection-methods)
- [ADX, Bollinger Bands, ATR](../README.md#indicators-used)
- [HMM Details](../README.md#-hmm-details)
- [Ensemble Method](../README.md#3-ensemble-recommended)

### Trading
- [Strategies](../README.md#-overview)
- [Position Sizing](QUICK_REFERENCE.md#-configuration-options)
- [Risk Management](SETUP_GUIDE.md#step-1-configure-environment)
- [Fee Tiers](QUICK_REFERENCE.md#fee-tiers)

### Configuration
- [All Options](QUICK_REFERENCE.md#-configuration-options)
- [Regime Config](SETUP_GUIDE.md#regimeconfig)
- [Backtest Config](SETUP_GUIDE.md#backtestconfig)

### Performance
- [Metrics](SETUP_GUIDE.md#understanding-backtest-results)
- [Benchmarks](SETUP_GUIDE.md#performance-benchmarks)
- [Optimization](SETUP_GUIDE.md#walk-forward-analysis)

### Integration
- [With Existing Bot](../README.md#integration-with-your-existing-codebase)
- [WebSocket](../README.md#step-2-integrate-with-your-websocket-handler)
- [Discord Alerts](USAGE.md#with-discord-notifications)
- [Redis State](USAGE.md#with-redis-state-persistence)

---

## 📚 Additional Resources

### Research & Theory
- [../README.md - Research References](../README.md#-research-references)
- Turtle Trading methodology
- Bollinger Bands mean reversion
- Regime-switching models

### Code Examples
- [examples/](examples/) directory
- [USAGE.md](USAGE.md) - Code snippets
- [SETUP_GUIDE.md](SETUP_GUIDE.md) - Integration examples

### Testing
- 24+ unit tests covering all components
- [Test Categories](SETUP_GUIDE.md#test-categories)
- [Writing Tests](USAGE.md#example-test)

---

## 🆘 Getting Help

### Quick Answers
- Check [QUICK_REFERENCE.md](QUICK_REFERENCE.md) first
- Look at [Troubleshooting](SETUP_GUIDE.md#troubleshooting)

### Still Stuck?
1. Enable debug logging: `RUST_LOG=debug cargo run ...`
2. Check the test suite: `cargo test -- --nocapture`
3. Review [Common Issues](SETUP_GUIDE.md#common-issues)

### Understanding Concepts
- Read [../README.md](../README.md) for theory
- Review [Detection Methods](../README.md#-detection-methods)
- See [Expected Performance](../README.md#-expected-performance)

---

## ⚠️ Important Warnings

Before going live with any trading system:

1. ✅ **Always backtest first** - [Backtesting Guide](SETUP_GUIDE.md#running-backtests)
2. ✅ **Run walk-forward analysis** - [Walk-Forward Guide](SETUP_GUIDE.md#walk-forward-analysis)
3. ✅ **Paper trade for 2-4 weeks** - [Paper Trading](SETUP_GUIDE.md#paper-trading-setup)
4. ✅ **Start with minimal capital** - Risk management is critical
5. ⚠️ **Past performance ≠ future results** - Markets change

See [Risk Warnings](../README.md#-risk-warnings) for more details.

---

## 📝 Document Versions

- **../README.md** - Project overview (always up to date)
- **QUICK_REFERENCE.md** - CLI reference v1.0
- **SETUP_GUIDE.md** - Setup guide v1.0
- **USAGE.md** - API reference v1.0
- **PROJECT_REVIEW.md** - Architecture review

---

## 🚀 Next Steps

1. **New to the project?** Start with [../README.md](../README.md)
2. **Ready to code?** Follow [SETUP_GUIDE.md](SETUP_GUIDE.md)
3. **Need a command?** Check [QUICK_REFERENCE.md](QUICK_REFERENCE.md)
4. **Building an integration?** See [USAGE.md](USAGE.md)

Happy trading! 🦑📈