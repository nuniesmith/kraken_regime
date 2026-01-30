//! Strategy Router
//!
//! Automatically switches between trading strategies based on detected market regime.
//! This is the core of regime-aware trading - using the right tool for current conditions.
//!
//! Research shows regime-aware strategies outperform static ones by 20-40% by:
//! - Using trend-following in trending markets
//! - Using mean reversion in ranging markets
//! - Reducing exposure in volatile/choppy markets

use crate::regime::{MarketRegime, RegimeConfidence, RegimeConfig, RegimeDetector, TrendDirection};
use crate::strategy::mean_reversion::{MeanReversionConfig, MeanReversionStrategy, Signal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the strategy router
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRouterConfig {
    /// Regime detection config
    pub regime_config: RegimeConfig,

    /// Mean reversion strategy config
    pub mean_reversion_config: MeanReversionConfig,

    /// Position size reduction factor when in volatile regime
    pub volatile_position_size_factor: f64,

    /// Minimum confidence to act on regime
    pub min_regime_confidence: f64,

    /// Enable logging of regime changes
    pub log_regime_changes: bool,

    /// EMA periods for trend following (your existing Golden Cross)
    pub trend_ema_short: usize,
    pub trend_ema_long: usize,
}

impl Default for StrategyRouterConfig {
    fn default() -> Self {
        Self {
            regime_config: RegimeConfig::crypto_optimized(),
            mean_reversion_config: MeanReversionConfig::default(),
            volatile_position_size_factor: 0.5, // Half position in volatile markets
            min_regime_confidence: 0.5,
            log_regime_changes: true,
            trend_ema_short: 50,
            trend_ema_long: 200,
        }
    }
}

/// Trade signal with routing information
#[derive(Debug, Clone)]
pub struct RoutedSignal {
    /// The trading signal
    pub signal: Signal,

    /// Which strategy generated the signal
    pub source_strategy: ActiveStrategy,

    /// Current detected regime
    pub regime: MarketRegime,

    /// Regime confidence
    pub confidence: f64,

    /// Position size multiplier (reduced in volatile markets)
    pub position_size_factor: f64,

    /// Reasoning for the signal
    pub reason: String,

    /// Stop loss level if applicable
    pub stop_loss: Option<f64>,

    /// Take profit level if applicable
    pub take_profit: Option<f64>,
}

/// Currently active strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveStrategy {
    /// EMA Golden Cross / Pullback - trend following
    TrendFollowing,
    /// Bollinger Bands mean reversion
    MeanReversion,
    /// Staying out of market
    NoTrade,
}

impl std::fmt::Display for ActiveStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActiveStrategy::TrendFollowing => write!(f, "Trend Following"),
            ActiveStrategy::MeanReversion => write!(f, "Mean Reversion"),
            ActiveStrategy::NoTrade => write!(f, "No Trade"),
        }
    }
}

/// Per-asset strategy state
#[derive(Debug)]
struct AssetState {
    regime_detector: RegimeDetector,
    mean_reversion: MeanReversionStrategy,
    current_strategy: ActiveStrategy,
    last_regime: MarketRegime,
    regime_change_count: u32,
}

impl AssetState {
    fn new(config: &StrategyRouterConfig) -> Self {
        Self {
            regime_detector: RegimeDetector::new(config.regime_config.clone()),
            mean_reversion: MeanReversionStrategy::new(config.mean_reversion_config.clone()),
            current_strategy: ActiveStrategy::NoTrade,
            last_regime: MarketRegime::Uncertain,
            regime_change_count: 0,
        }
    }
}

/// Main Strategy Router
///
/// Manages multiple assets, each with their own regime detection and strategy selection.
#[derive(Debug)]
pub struct StrategyRouter {
    config: StrategyRouterConfig,
    assets: HashMap<String, AssetState>,
}

impl StrategyRouter {
    pub fn new(config: StrategyRouterConfig) -> Self {
        Self {
            config,
            assets: HashMap::new(),
        }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(StrategyRouterConfig::default())
    }

    /// Register an asset (e.g., "BTC/USD", "ETH/USD", "SOL/USD")
    pub fn register_asset(&mut self, symbol: &str) {
        if !self.assets.contains_key(symbol) {
            self.assets
                .insert(symbol.to_string(), AssetState::new(&self.config));
        }
    }

    /// Update with new OHLC data for an asset and get routed signal
    pub fn update(
        &mut self,
        symbol: &str,
        high: f64,
        low: f64,
        close: f64,
    ) -> Option<RoutedSignal> {
        // Register if not seen before
        if !self.assets.contains_key(symbol) {
            self.register_asset(symbol);
        }

        let config = &self.config;
        let state = self.assets.get_mut(symbol)?;

        // Update regime detection
        let regime_result = state.regime_detector.update(high, low, close);

        // Check for regime change
        if regime_result.regime != state.last_regime {
            state.regime_change_count += 1;
            if config.log_regime_changes {
                println!(
                    "[{}] Regime change #{}: {} → {} (confidence: {:.2})",
                    symbol,
                    state.regime_change_count,
                    state.last_regime,
                    regime_result.regime,
                    regime_result.confidence
                );
            }
            state.last_regime = regime_result.regime;
        }

        // Determine active strategy based on regime
        let (active_strategy, position_factor) = Self::select_strategy(
            &regime_result,
            config.min_regime_confidence,
            config.volatile_position_size_factor,
        );

        state.current_strategy = active_strategy;

        // Generate signal based on active strategy
        let (signal, reason, stop_loss, take_profit) = match active_strategy {
            ActiveStrategy::TrendFollowing => {
                // Your existing Golden Cross / EMA Pullback logic would go here
                // For now, returning Hold - integrate with your existing strategies
                Self::trend_following_signal(&state.regime_detector, close)
            }
            ActiveStrategy::MeanReversion => {
                let mr_signal = state.mean_reversion.update(high, low, close);
                let reason = format!(
                    "Mean Reversion: %B={:.2}, RSI={:.1}",
                    state
                        .mean_reversion
                        .last_bb_values()
                        .map(|b| b.percent_b)
                        .unwrap_or(0.5),
                    state.mean_reversion.last_rsi().unwrap_or(50.0)
                );
                (
                    mr_signal,
                    reason,
                    state.mean_reversion.stop_loss(),
                    state.mean_reversion.take_profit(),
                )
            }
            ActiveStrategy::NoTrade => (
                Signal::Hold,
                "Volatile/Uncertain - staying out".to_string(),
                None,
                None,
            ),
        };

        Some(RoutedSignal {
            signal,
            source_strategy: active_strategy,
            regime: regime_result.regime,
            confidence: regime_result.confidence,
            position_size_factor: position_factor,
            reason,
            stop_loss,
            take_profit,
        })
    }

    /// Select strategy based on regime
    fn select_strategy(
        regime: &RegimeConfidence,
        min_confidence: f64,
        volatile_factor: f64,
    ) -> (ActiveStrategy, f64) {
        // If confidence too low, stay out
        if regime.confidence < min_confidence {
            return (ActiveStrategy::NoTrade, 0.0);
        }

        match regime.regime {
            MarketRegime::Trending(_) => (ActiveStrategy::TrendFollowing, 1.0),
            MarketRegime::MeanReverting => (ActiveStrategy::MeanReversion, 1.0),
            MarketRegime::Volatile => {
                // Still trade but with reduced size
                // Use mean reversion with tight stops in volatile markets
                (ActiveStrategy::MeanReversion, volatile_factor)
            }
            MarketRegime::Uncertain => (ActiveStrategy::NoTrade, 0.0),
        }
    }

    /// Simple trend following signal based on EMA alignment
    /// (Placeholder - integrate with your existing Golden Cross strategy)
    fn trend_following_signal(
        detector: &RegimeDetector,
        close: f64,
    ) -> (Signal, String, Option<f64>, Option<f64>) {
        let adx = detector.adx_value().unwrap_or(0.0);
        let atr = detector.atr_value().unwrap_or(close * 0.02);

        // This is a simplified version - integrate with your existing EMA strategies
        let regime = detector.current_regime();

        match regime {
            MarketRegime::Trending(TrendDirection::Bullish) if adx > 25.0 => {
                let stop_loss = close - (atr * 2.0);
                let take_profit = close + (atr * 3.0); // 1.5 R:R
                (
                    Signal::Buy,
                    format!("Trend Buy: Bullish trend, ADX={:.1}", adx),
                    Some(stop_loss),
                    Some(take_profit),
                )
            }
            MarketRegime::Trending(TrendDirection::Bearish) if adx > 25.0 => {
                // In spot trading, we'd sell/exit here, not short
                let stop_loss = close + (atr * 2.0);
                let take_profit = close - (atr * 3.0);
                (
                    Signal::Sell,
                    format!("Trend Sell: Bearish trend, ADX={:.1}", adx),
                    Some(stop_loss),
                    Some(take_profit),
                )
            }
            _ => (
                Signal::Hold,
                "Trend: Waiting for stronger signal".to_string(),
                None,
                None,
            ),
        }
    }

    /// Get current regime for an asset
    pub fn get_regime(&self, symbol: &str) -> Option<MarketRegime> {
        self.assets.get(symbol).map(|s| s.last_regime)
    }

    /// Get current active strategy for an asset
    pub fn get_active_strategy(&self, symbol: &str) -> Option<ActiveStrategy> {
        self.assets.get(symbol).map(|s| s.current_strategy)
    }

    /// Get all registered assets
    pub fn assets(&self) -> Vec<&str> {
        self.assets.keys().map(|s| s.as_str()).collect()
    }

    /// Get regime change count for an asset
    pub fn regime_changes(&self, symbol: &str) -> Option<u32> {
        self.assets.get(symbol).map(|s| s.regime_change_count)
    }

    /// Is the router ready for an asset (enough data)?
    pub fn is_ready(&self, symbol: &str) -> bool {
        self.assets
            .get(symbol)
            .map(|s| s.regime_detector.is_ready())
            .unwrap_or(false)
    }

    /// Get config
    pub fn config(&self) -> &StrategyRouterConfig {
        &self.config
    }
}

/// Statistics about router performance
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RouterStats {
    pub total_signals: u64,
    pub trend_following_signals: u64,
    pub mean_reversion_signals: u64,
    pub no_trade_periods: u64,
    pub regime_changes: u64,
}

impl RouterStats {
    pub fn record_signal(&mut self, signal: &RoutedSignal) {
        self.total_signals += 1;
        match signal.source_strategy {
            ActiveStrategy::TrendFollowing => self.trend_following_signals += 1,
            ActiveStrategy::MeanReversion => self.mean_reversion_signals += 1,
            ActiveStrategy::NoTrade => self.no_trade_periods += 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_registration() {
        let mut router = StrategyRouter::default_config();

        router.register_asset("BTC/USD");
        router.register_asset("ETH/USD");
        router.register_asset("SOL/USD");

        assert_eq!(router.assets().len(), 3);
    }

    #[test]
    fn test_router_update() {
        let mut router = StrategyRouter::default_config();

        // Feed data until ready
        for i in 0..250 {
            let price = 50000.0 + (i as f64 * 10.0); // Trending up
            let high = price + 50.0;
            let low = price - 50.0;

            let result = router.update("BTC/USD", high, low, price);

            if router.is_ready("BTC/USD") && result.is_some() {
                let signal = result.unwrap();
                println!(
                    "Bar {}: Regime={}, Strategy={}, Signal={:?}",
                    i, signal.regime, signal.source_strategy, signal.signal
                );
            }
        }

        assert!(router.is_ready("BTC/USD"));
    }

    #[test]
    fn test_regime_based_strategy_selection() {
        let _config = StrategyRouterConfig::default();

        // Trending regime should select TrendFollowing
        let trending = RegimeConfidence::new(MarketRegime::Trending(TrendDirection::Bullish), 0.8);
        let (strategy, _) = StrategyRouter::select_strategy(&trending, 0.5, 0.5);
        assert_eq!(strategy, ActiveStrategy::TrendFollowing);

        // Mean reverting should select MeanReversion
        let ranging = RegimeConfidence::new(MarketRegime::MeanReverting, 0.8);
        let (strategy, _) = StrategyRouter::select_strategy(&ranging, 0.5, 0.5);
        assert_eq!(strategy, ActiveStrategy::MeanReversion);

        // Low confidence should be NoTrade
        let uncertain = RegimeConfidence::new(MarketRegime::Trending(TrendDirection::Bullish), 0.3);
        let (strategy, _) = StrategyRouter::select_strategy(&uncertain, 0.5, 0.5);
        assert_eq!(strategy, ActiveStrategy::NoTrade);
    }
}
