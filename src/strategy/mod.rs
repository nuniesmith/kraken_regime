//! Trading Strategies Module
//!
//! Contains various trading strategies that can be selected based on market regime.
//! 
//! ## Router Options
//! 
//! - `StrategyRouter` - Original router using indicator-based detection
//! - `EnhancedRouter` - Supports Indicators, HMM, or Ensemble detection

pub mod mean_reversion;
pub mod router;
pub mod enhanced_router;

// Re-export main types
pub use mean_reversion::{MeanReversionStrategy, MeanReversionConfig, Signal, StrategyResult};
pub use router::{StrategyRouter, StrategyRouterConfig, RoutedSignal, ActiveStrategy, RouterStats};
pub use enhanced_router::{EnhancedRouter, EnhancedRouterConfig, EnhancedSignal, DetectionMethod};
