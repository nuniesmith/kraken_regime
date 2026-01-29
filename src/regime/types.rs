//! Regime types and classifications

use serde::{Deserialize, Serialize};
use std::fmt;

/// Market regime classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketRegime {
    /// Strong directional movement - use trend-following strategies
    /// Characteristics: High ADX (>25), price above/below MAs, clear momentum
    Trending(TrendDirection),
    
    /// Price oscillating around a mean - use mean reversion strategies  
    /// Characteristics: Low ADX (<20), price within Bollinger Bands, range-bound
    MeanReverting,
    
    /// High volatility, no clear direction - reduce exposure or stay cash
    /// Characteristics: ATR expansion, wide Bollinger Bands, choppy price action
    Volatile,
    
    /// Insufficient data or unclear signals - be cautious
    Uncertain,
}

/// Direction of trend when in Trending regime
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Bullish,
    Bearish,
}

impl fmt::Display for MarketRegime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarketRegime::Trending(TrendDirection::Bullish) => write!(f, "Trending (Bullish)"),
            MarketRegime::Trending(TrendDirection::Bearish) => write!(f, "Trending (Bearish)"),
            MarketRegime::MeanReverting => write!(f, "Mean-Reverting"),
            MarketRegime::Volatile => write!(f, "Volatile/Choppy"),
            MarketRegime::Uncertain => write!(f, "Uncertain"),
        }
    }
}

/// Confidence level in regime classification
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RegimeConfidence {
    pub regime: MarketRegime,
    pub confidence: f64,  // 0.0 to 1.0
    pub adx_value: f64,
    pub bb_width_percentile: f64,
    pub trend_strength: f64,
}

impl RegimeConfidence {
    pub fn new(regime: MarketRegime, confidence: f64) -> Self {
        Self {
            regime,
            confidence,
            adx_value: 0.0,
            bb_width_percentile: 0.0,
            trend_strength: 0.0,
        }
    }
    
    pub fn with_metrics(
        regime: MarketRegime,
        confidence: f64,
        adx: f64,
        bb_width: f64,
        trend_strength: f64,
    ) -> Self {
        Self {
            regime,
            confidence,
            adx_value: adx,
            bb_width_percentile: bb_width,
            trend_strength,
        }
    }
    
    /// Whether confidence is high enough to act on
    pub fn is_actionable(&self) -> bool {
        self.confidence >= 0.6
    }
}

/// Configuration for regime detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeConfig {
    /// ADX period for trend strength
    pub adx_period: usize,
    /// ADX threshold above which market is considered trending
    pub adx_trending_threshold: f64,
    /// ADX threshold below which market is considered mean-reverting
    pub adx_ranging_threshold: f64,
    
    /// Bollinger Bands period
    pub bb_period: usize,
    /// Bollinger Bands standard deviation multiplier
    pub bb_std_dev: f64,
    /// BB width percentile threshold for high volatility
    pub bb_width_volatility_threshold: f64,
    
    /// EMA periods for trend direction
    pub ema_short_period: usize,
    pub ema_long_period: usize,
    
    /// ATR period for volatility measurement
    pub atr_period: usize,
    /// ATR expansion multiplier (current vs average) for volatile regime
    pub atr_expansion_threshold: f64,
    
    /// Lookback period for regime stability (avoid whipsaws)
    pub regime_stability_bars: usize,
    /// Minimum bars in current regime before switching
    pub min_regime_duration: usize,
}

impl Default for RegimeConfig {
    fn default() -> Self {
        Self {
            adx_period: 14,
            adx_trending_threshold: 25.0,
            adx_ranging_threshold: 20.0,
            bb_period: 20,
            bb_std_dev: 2.0,
            bb_width_volatility_threshold: 75.0,  // percentile
            ema_short_period: 50,
            ema_long_period: 200,
            atr_period: 14,
            atr_expansion_threshold: 1.5,
            regime_stability_bars: 3,
            min_regime_duration: 5,
        }
    }
}

impl RegimeConfig {
    /// Configuration optimized for crypto markets (BTC, ETH, SOL)
    pub fn crypto_optimized() -> Self {
        Self {
            adx_period: 14,
            adx_trending_threshold: 20.0,  // Lower threshold - crypto trends hard
            adx_ranging_threshold: 15.0,
            bb_period: 20,
            bb_std_dev: 2.0,
            bb_width_volatility_threshold: 70.0,
            ema_short_period: 21,  // Faster for crypto
            ema_long_period: 50,
            atr_period: 14,
            atr_expansion_threshold: 1.3,  // Crypto is naturally volatile
            regime_stability_bars: 2,
            min_regime_duration: 3,
        }
    }
    
    /// Conservative config - requires stronger signals
    pub fn conservative() -> Self {
        Self {
            adx_period: 14,
            adx_trending_threshold: 30.0,
            adx_ranging_threshold: 18.0,
            bb_period: 20,
            bb_std_dev: 2.0,
            bb_width_volatility_threshold: 80.0,
            ema_short_period: 50,
            ema_long_period: 200,
            atr_period: 14,
            atr_expansion_threshold: 2.0,
            regime_stability_bars: 5,
            min_regime_duration: 10,
        }
    }
}

/// Recommended strategy for current regime
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendedStrategy {
    /// Use trend-following (Golden Cross, EMA Pullback)
    TrendFollowing,
    /// Use mean reversion (Bollinger Bands)
    MeanReversion,
    /// Reduce position size, tight stops
    ReducedExposure,
    /// Stay in cash, wait for clarity
    StayCash,
}

impl From<&MarketRegime> for RecommendedStrategy {
    fn from(regime: &MarketRegime) -> Self {
        match regime {
            MarketRegime::Trending(_) => RecommendedStrategy::TrendFollowing,
            MarketRegime::MeanReverting => RecommendedStrategy::MeanReversion,
            MarketRegime::Volatile => RecommendedStrategy::ReducedExposure,
            MarketRegime::Uncertain => RecommendedStrategy::StayCash,
        }
    }
}
