//! Trading Strategies Module
//!
//! Contains various trading strategies that can be selected based on market regime.
//!
//! ## Router Options
//!
//! - `StrategyRouter` - Original router using indicator-based detection
//! - `EnhancedRouter` - Supports Indicators, HMM, or Ensemble detection

pub mod enhanced_router;
pub mod mean_reversion;
pub mod router;

// Re-export main types
pub use enhanced_router::{DetectionMethod, EnhancedRouter, EnhancedRouterConfig, EnhancedSignal};
pub use mean_reversion::{MeanReversionConfig, MeanReversionStrategy, Signal, StrategyResult};
pub use router::{ActiveStrategy, RoutedSignal, RouterStats, StrategyRouter, StrategyRouterConfig};
