//! Kraken Regime-Aware Trading Bot
//!
//! A trading system that automatically detects market regimes and switches
//! between appropriate strategies:
//!
//! - **Trending markets** → Use trend-following (Golden Cross, EMA Pullback)
//! - **Ranging markets** → Use mean reversion (Bollinger Bands)
//! - **Volatile markets** → Reduce position size or stay in cash
//!
//! ## Key Features
//!
//! - Multi-asset support (BTC, ETH, SOL)
//! - Real-time regime detection using ADX, Bollinger Bands, ATR
//! - Automatic strategy switching
//! - Integration with Kraken WebSocket and REST APIs
//! - Risk management with dynamic position sizing
//!
//! ## Usage
//!
//! ```rust,no_run
//! use kraken_regime::{
//!     integration::KrakenRegimeTrader,
//!     integration::KrakenIntegrationConfig,
//! };
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create trader with default config
//!     let config = KrakenIntegrationConfig::default();
//!     let mut trader = KrakenRegimeTrader::new(config);
//!
//!     // Warmup with historical data
//!     // trader.warmup_with_history("BTC/USD", &historical_candles);
//!
//!     // Process real-time data
//!     // let action = trader.process_candle("BTC/USD", &candle);
//! }
//! ```
//!
//! ## Research Background
//!
//! Regime-aware trading is based on the principle that different market
//! conditions favor different strategies:
//!
//! - Markets alternate between trending, mean-reverting, and volatile states
//! - Trend-following strategies profit in trends but lose in ranges
//! - Mean reversion strategies profit in ranges but lose in trends
//! - Detecting the regime and selecting the right strategy improves returns
//!   by 20-40% according to research, primarily by avoiding large drawdowns

pub mod regime;
pub mod strategy;
pub mod integration;

// Re-exports for convenience
pub use regime::{
    MarketRegime,
    TrendDirection,
    RegimeConfig,
    RegimeConfidence,
    RegimeDetector,
    RecommendedStrategy,
};

pub use strategy::{
    mean_reversion::{MeanReversionStrategy, MeanReversionConfig, Signal},
    router::{StrategyRouter, StrategyRouterConfig, RoutedSignal, ActiveStrategy},
};

pub use integration::{
    KrakenRegimeTrader,
    KrakenIntegrationConfig,
    Candle,
    TradeAction,
    TradeType,
};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude for common imports
pub mod prelude {
    pub use crate::{
        MarketRegime,
        RegimeDetector,
        StrategyRouter,
        KrakenRegimeTrader,
        KrakenIntegrationConfig,
        Candle,
        TradeAction,
        Signal,
    };
}
