//! Enhanced Strategy Router with Multiple Detection Methods
//! 
//! Extends the base StrategyRouter to support:
//! 1. Technical Indicators (default)
//! 2. Hidden Markov Model
//! 3. Ensemble (both combined)
//!
//! Usage:
//! ```rust
//! let router = EnhancedRouter::with_hmm();  // Use HMM detection
//! let router = EnhancedRouter::with_ensemble();  // Use Ensemble (recommended)
//! ```

use crate::regime::{
    RegimeDetector, RegimeConfig, MarketRegime, RegimeConfidence, TrendDirection,
    HMMRegimeDetector, HMMConfig,
    EnsembleRegimeDetector, EnsembleConfig, EnsembleResult,
};
use crate::strategy::mean_reversion::{MeanReversionStrategy, MeanReversionConfig, Signal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Which detection method to use
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionMethod {
    /// Technical indicators (ADX, BB, ATR) - fast, rule-based
    Indicators,
    /// Hidden Markov Model - statistical, learns from returns
    HMM,
    /// Ensemble - combines both for robustness (recommended)
    Ensemble,
}

impl Default for DetectionMethod {
    fn default() -> Self {
        DetectionMethod::Ensemble  // Recommended default
    }
}

/// Configuration for enhanced router
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedRouterConfig {
    /// Which detection method to use
    pub detection_method: DetectionMethod,
    
    /// Indicator-based config
    pub indicator_config: RegimeConfig,
    
    /// HMM config
    #[serde(skip)]
    pub hmm_config: Option<HMMConfig>,
    
    /// Ensemble config  
    #[serde(skip)]
    pub ensemble_config: Option<EnsembleConfig>,
    
    /// Mean reversion strategy config
    pub mean_reversion_config: MeanReversionConfig,
    
    /// Position size in volatile markets
    pub volatile_position_factor: f64,
    
    /// Minimum confidence to trade
    pub min_confidence: f64,
    
    /// Log regime changes
    pub log_changes: bool,
}

impl Default for EnhancedRouterConfig {
    fn default() -> Self {
        Self {
            detection_method: DetectionMethod::Ensemble,
            indicator_config: RegimeConfig::crypto_optimized(),
            hmm_config: Some(HMMConfig::crypto_optimized()),
            ensemble_config: Some(EnsembleConfig::default()),
            mean_reversion_config: MeanReversionConfig::default(),
            volatile_position_factor: 0.5,
            min_confidence: 0.5,
            log_changes: true,
        }
    }
}

/// Active strategy being used
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveStrategy {
    TrendFollowing,
    MeanReversion,
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

/// Enhanced routed signal with extra info
#[derive(Debug, Clone)]
pub struct EnhancedSignal {
    pub signal: Signal,
    pub strategy: ActiveStrategy,
    pub regime: MarketRegime,
    pub confidence: f64,
    pub position_factor: f64,
    pub reason: String,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    
    /// Which detection method produced this
    pub detection_method: DetectionMethod,
    
    /// Did methods agree? (only for Ensemble)
    pub methods_agree: Option<bool>,
    
    /// HMM state probabilities (if using HMM/Ensemble)
    pub state_probabilities: Option<Vec<f64>>,
    
    /// Expected regime duration in bars (from HMM)
    pub expected_duration: Option<f64>,
}

/// Wrapper for different detector types
enum Detector {
    Indicator(RegimeDetector),
    HMM(HMMRegimeDetector),
    Ensemble(EnsembleRegimeDetector),
}

/// Per-asset state
struct AssetState {
    detector: Detector,
    mean_reversion: MeanReversionStrategy,
    current_strategy: ActiveStrategy,
    last_regime: MarketRegime,
    regime_change_count: u32,
}

/// Enhanced Strategy Router
pub struct EnhancedRouter {
    config: EnhancedRouterConfig,
    assets: HashMap<String, AssetState>,
}

impl EnhancedRouter {
    /// Create with specific config
    pub fn new(config: EnhancedRouterConfig) -> Self {
        Self {
            config,
            assets: HashMap::new(),
        }
    }
    
    /// Create with indicator-based detection
    pub fn with_indicators() -> Self {
        Self::new(EnhancedRouterConfig {
            detection_method: DetectionMethod::Indicators,
            ..Default::default()
        })
    }
    
    /// Create with HMM-based detection
    pub fn with_hmm() -> Self {
        Self::new(EnhancedRouterConfig {
            detection_method: DetectionMethod::HMM,
            hmm_config: Some(HMMConfig::crypto_optimized()),
            ..Default::default()
        })
    }
    
    /// Create with Ensemble detection (recommended)
    pub fn with_ensemble() -> Self {
        Self::new(EnhancedRouterConfig {
            detection_method: DetectionMethod::Ensemble,
            ensemble_config: Some(EnsembleConfig::default()),
            ..Default::default()
        })
    }
    
    /// Register an asset
    pub fn register_asset(&mut self, symbol: &str) {
        if self.assets.contains_key(symbol) {
            return;
        }
        
        let detector = match self.config.detection_method {
            DetectionMethod::Indicators => {
                Detector::Indicator(RegimeDetector::new(self.config.indicator_config.clone()))
            }
            DetectionMethod::HMM => {
                let hmm_config = self.config.hmm_config.clone().unwrap_or_default();
                Detector::HMM(HMMRegimeDetector::new(hmm_config))
            }
            DetectionMethod::Ensemble => {
                let ens_config = self.config.ensemble_config.clone().unwrap_or_default();
                Detector::Ensemble(EnsembleRegimeDetector::new(
                    ens_config,
                    self.config.indicator_config.clone(),
                ))
            }
        };
        
        self.assets.insert(symbol.to_string(), AssetState {
            detector,
            mean_reversion: MeanReversionStrategy::new(self.config.mean_reversion_config.clone()),
            current_strategy: ActiveStrategy::NoTrade,
            last_regime: MarketRegime::Uncertain,
            regime_change_count: 0,
        });
    }
    
    /// Update with new OHLC data
    pub fn update(&mut self, symbol: &str, high: f64, low: f64, close: f64) -> Option<EnhancedSignal> {
        if !self.assets.contains_key(symbol) {
            self.register_asset(symbol);
        }
        
        let state = self.assets.get_mut(symbol)?;
        
        // Get regime from appropriate detector
        let (regime_result, methods_agree, state_probs, expected_duration) = match &mut state.detector {
            Detector::Indicator(det) => {
                let result = det.update(high, low, close);
                (result, None, None, None)
            }
            Detector::HMM(det) => {
                let result = det.update_ohlc(high, low, close);
                let probs = det.state_probabilities().to_vec();
                let duration = det.expected_regime_duration(det.current_state_index());
                (result, None, Some(probs), Some(duration))
            }
            Detector::Ensemble(det) => {
                let ens_result = det.update(high, low, close);
                let probs = det.hmm_state_probabilities().to_vec();
                let duration = det.expected_regime_duration();
                (ens_result.to_regime_confidence(), Some(ens_result.methods_agree), Some(probs), Some(duration))
            }
        };
        
        // Check for regime change
        if regime_result.regime != state.last_regime {
            state.regime_change_count += 1;
            if self.config.log_changes {
                println!(
                    "[{}] Regime change #{} ({:?}): {} → {} (conf: {:.2})",
                    symbol,
                    state.regime_change_count,
                    self.config.detection_method,
                    state.last_regime,
                    regime_result.regime,
                    regime_result.confidence
                );
            }
            state.last_regime = regime_result.regime;
        }
        
        // Select strategy based on regime
        let (strategy, position_factor) = self.select_strategy(&regime_result);
        state.current_strategy = strategy;
        
        // Generate signal
        let (signal, reason, stop_loss, take_profit) = match strategy {
            ActiveStrategy::TrendFollowing => {
                // Simplified trend signal - integrate with your existing strategies
                self.trend_signal(&regime_result, close)
            }
            ActiveStrategy::MeanReversion => {
                let mr_signal = state.mean_reversion.update(high, low, close);
                let bb = state.mean_reversion.last_bb_values();
                let rsi = state.mean_reversion.last_rsi();
                (
                    mr_signal,
                    format!("MeanRev: %B={:.2} RSI={:.0}", 
                            bb.map(|b| b.percent_b).unwrap_or(0.5),
                            rsi.unwrap_or(50.0)),
                    state.mean_reversion.stop_loss(),
                    state.mean_reversion.take_profit(),
                )
            }
            ActiveStrategy::NoTrade => {
                (Signal::Hold, "Uncertain - staying out".into(), None, None)
            }
        };
        
        Some(EnhancedSignal {
            signal,
            strategy,
            regime: regime_result.regime,
            confidence: regime_result.confidence,
            position_factor,
            reason,
            stop_loss,
            take_profit,
            detection_method: self.config.detection_method,
            methods_agree,
            state_probabilities: state_probs,
            expected_duration,
        })
    }
    
    /// Select strategy based on regime
    fn select_strategy(&self, regime: &RegimeConfidence) -> (ActiveStrategy, f64) {
        if regime.confidence < self.config.min_confidence {
            return (ActiveStrategy::NoTrade, 0.0);
        }
        
        match regime.regime {
            MarketRegime::Trending(_) => (ActiveStrategy::TrendFollowing, 1.0),
            MarketRegime::MeanReverting => (ActiveStrategy::MeanReversion, 1.0),
            MarketRegime::Volatile => (ActiveStrategy::MeanReversion, self.config.volatile_position_factor),
            MarketRegime::Uncertain => (ActiveStrategy::NoTrade, 0.0),
        }
    }
    
    /// Generate trend following signal
    fn trend_signal(
        &self,
        regime: &RegimeConfidence,
        close: f64,
    ) -> (Signal, String, Option<f64>, Option<f64>) {
        // Simplified - integrate with your existing Golden Cross/EMA Pullback
        match regime.regime {
            MarketRegime::Trending(TrendDirection::Bullish) if regime.confidence > 0.6 => {
                let atr_estimate = close * 0.02;
                (
                    Signal::Buy,
                    format!("Bullish Trend (conf: {:.0}%)", regime.confidence * 100.0),
                    Some(close - atr_estimate * 2.0),
                    Some(close + atr_estimate * 3.0),
                )
            }
            MarketRegime::Trending(TrendDirection::Bearish) if regime.confidence > 0.6 => {
                (
                    Signal::Sell,
                    format!("Bearish Trend (conf: {:.0}%)", regime.confidence * 100.0),
                    None,
                    None,
                )
            }
            _ => (Signal::Hold, "Trend unclear".into(), None, None),
        }
    }
    
    /// Get current regime for an asset
    pub fn get_regime(&self, symbol: &str) -> Option<MarketRegime> {
        self.assets.get(symbol).map(|s| s.last_regime)
    }
    
    /// Get current strategy for an asset
    pub fn get_strategy(&self, symbol: &str) -> Option<ActiveStrategy> {
        self.assets.get(symbol).map(|s| s.current_strategy)
    }
    
    /// Check if detector is warmed up
    pub fn is_ready(&self, symbol: &str) -> bool {
        self.assets.get(symbol).map(|s| {
            match &s.detector {
                Detector::Indicator(d) => d.is_ready(),
                Detector::HMM(d) => d.is_ready(),
                Detector::Ensemble(d) => d.is_ready(),
            }
        }).unwrap_or(false)
    }
    
    /// Get detection method being used
    pub fn detection_method(&self) -> DetectionMethod {
        self.config.detection_method
    }
    
    /// Get regime change count for an asset
    pub fn regime_changes(&self, symbol: &str) -> u32 {
        self.assets.get(symbol).map(|s| s.regime_change_count).unwrap_or(0)
    }
}

impl std::fmt::Display for EnhancedSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} | Regime: {} | Strategy: {} | Conf: {:.0}% | Size: {:.0}%",
            self.signal,
            self.regime,
            self.strategy,
            self.confidence * 100.0,
            self.position_factor * 100.0
        )?;
        
        if let Some(agree) = self.methods_agree {
            write!(f, " | Agree: {}", if agree { "✓" } else { "✗" })?;
        }
        
        if let Some(dur) = self.expected_duration {
            write!(f, " | ExpDur: {:.0} bars", dur)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_enhanced_router_creation() {
        let router = EnhancedRouter::with_ensemble();
        assert_eq!(router.detection_method(), DetectionMethod::Ensemble);
    }
    
    #[test]
    fn test_method_switching() {
        let indicator_router = EnhancedRouter::with_indicators();
        let hmm_router = EnhancedRouter::with_hmm();
        let ensemble_router = EnhancedRouter::with_ensemble();
        
        assert_eq!(indicator_router.detection_method(), DetectionMethod::Indicators);
        assert_eq!(hmm_router.detection_method(), DetectionMethod::HMM);
        assert_eq!(ensemble_router.detection_method(), DetectionMethod::Ensemble);
    }
    
    #[test]
    fn test_asset_registration() {
        let mut router = EnhancedRouter::with_ensemble();
        router.register_asset("BTC/USD");
        router.register_asset("ETH/USD");
        
        assert!(router.get_regime("BTC/USD").is_some());
        assert!(router.get_regime("ETH/USD").is_some());
        assert!(router.get_regime("SOL/USD").is_none());
    }
}
