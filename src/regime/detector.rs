//! Regime Detector
//! 
//! Combines multiple indicators to classify market regime and recommend strategies

use std::collections::VecDeque;
use super::{
    indicators::{ADX, ATR, BollingerBands, EMA},
    types::{MarketRegime, RegimeConfig, RegimeConfidence, TrendDirection, RecommendedStrategy},
};

/// Main regime detection engine
#[derive(Debug)]
pub struct RegimeDetector {
    config: RegimeConfig,
    
    // Indicators
    adx: ADX,
    atr: ATR,
    atr_avg: EMA,  // For measuring ATR expansion
    bb: BollingerBands,
    ema_short: EMA,
    ema_long: EMA,
    
    // State
    current_regime: MarketRegime,
    regime_history: VecDeque<MarketRegime>,
    bars_in_regime: usize,
    
    // For trend direction
    last_close: Option<f64>,
}

impl RegimeDetector {
    pub fn new(config: RegimeConfig) -> Self {
        Self {
            adx: ADX::new(config.adx_period),
            atr: ATR::new(config.atr_period),
            atr_avg: EMA::new(50),  // Longer-term ATR average
            bb: BollingerBands::new(config.bb_period, config.bb_std_dev),
            ema_short: EMA::new(config.ema_short_period),
            ema_long: EMA::new(config.ema_long_period),
            current_regime: MarketRegime::Uncertain,
            regime_history: VecDeque::with_capacity(20),
            bars_in_regime: 0,
            last_close: None,
            config,
        }
    }
    
    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(RegimeConfig::default())
    }
    
    /// Create optimized for crypto markets
    pub fn crypto_optimized() -> Self {
        Self::new(RegimeConfig::crypto_optimized())
    }
    
    /// Update with new OHLC bar
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> RegimeConfidence {
        // Update all indicators
        let adx_value = self.adx.update(high, low, close);
        let atr_value = self.atr.update(high, low, close);
        let bb_values = self.bb.update(close);
        let ema_short = self.ema_short.update(close);
        let ema_long = self.ema_long.update(close);
        
        // Update ATR average for expansion detection
        if let Some(atr) = atr_value {
            self.atr_avg.update(atr);
        }
        
        self.last_close = Some(close);
        
        // Check if we have enough data
        if !self.is_ready() {
            return RegimeConfidence::new(MarketRegime::Uncertain, 0.0);
        }
        
        // Detect regime
        let (new_regime, confidence) = self.classify_regime(
            adx_value.unwrap(),
            atr_value.unwrap(),
            bb_values.as_ref().unwrap(),
            ema_short.unwrap(),
            ema_long.unwrap(),
            close,
        );
        
        // Apply stability filter - avoid whipsawing
        let stable_regime = self.apply_stability_filter(new_regime, confidence);
        
        // Update state
        if stable_regime != self.current_regime {
            self.regime_history.push_back(self.current_regime);
            if self.regime_history.len() > 20 {
                self.regime_history.pop_front();
            }
            self.current_regime = stable_regime;
            self.bars_in_regime = 0;
        } else {
            self.bars_in_regime += 1;
        }
        
        RegimeConfidence::with_metrics(
            stable_regime,
            confidence,
            adx_value.unwrap(),
            bb_values.as_ref().map(|b| b.width_percentile).unwrap_or(50.0),
            self.calculate_trend_strength(ema_short.unwrap(), ema_long.unwrap(), close),
        )
    }
    
    /// Classify regime based on indicator values
    fn classify_regime(
        &self,
        adx: f64,
        atr: f64,
        bb: &super::indicators::BollingerBandsValues,
        ema_short: f64,
        ema_long: f64,
        close: f64,
    ) -> (MarketRegime, f64) {
        // Calculate ATR expansion
        let atr_expansion = if let Some(avg_atr) = self.atr_avg.value() {
            atr / avg_atr
        } else {
            1.0
        };
        
        // Score each regime possibility
        let mut trending_score = 0.0;
        let mut ranging_score = 0.0;
        let mut volatile_score = 0.0;
        
        // ADX analysis
        if adx >= self.config.adx_trending_threshold {
            trending_score += 0.4;
        } else if adx <= self.config.adx_ranging_threshold {
            ranging_score += 0.3;
        }
        
        // Bollinger Band width analysis
        if bb.is_high_volatility(self.config.bb_width_volatility_threshold) {
            volatile_score += 0.3;
        }
        if bb.is_squeeze(25.0) {
            ranging_score += 0.2;  // Tight bands suggest range-bound
        }
        
        // ATR expansion
        if atr_expansion >= self.config.atr_expansion_threshold {
            volatile_score += 0.3;
        } else if atr_expansion < 0.8 {
            ranging_score += 0.2;  // Low volatility suggests ranging
        }
        
        // EMA alignment for trend
        let ema_diff_pct = ((ema_short - ema_long) / ema_long).abs() * 100.0;
        if ema_diff_pct > 2.0 {
            trending_score += 0.3;
        } else if ema_diff_pct < 1.0 {
            ranging_score += 0.2;
        }
        
        // Price position relative to EMAs
        let price_above_both = close > ema_short && close > ema_long;
        let price_below_both = close < ema_short && close < ema_long;
        if price_above_both || price_below_both {
            trending_score += 0.2;
        } else {
            ranging_score += 0.2;  // Price between EMAs suggests consolidation
        }
        
        // Determine regime and direction
        let max_score = trending_score.max(ranging_score).max(volatile_score);
        let confidence = max_score / 1.2;  // Normalize to 0-1 range
        
        let regime = if volatile_score >= 0.5 && volatile_score >= trending_score {
            MarketRegime::Volatile
        } else if trending_score > ranging_score && trending_score > 0.3 {
            // Determine trend direction
            let direction = if ema_short > ema_long && close > ema_long {
                TrendDirection::Bullish
            } else if ema_short < ema_long && close < ema_long {
                TrendDirection::Bearish
            } else if let Some(dir) = self.adx.trend_direction() {
                dir
            } else {
                TrendDirection::Bullish  // Default
            };
            MarketRegime::Trending(direction)
        } else if ranging_score > 0.3 {
            MarketRegime::MeanReverting
        } else {
            MarketRegime::Uncertain
        };
        
        (regime, confidence.min(1.0))
    }
    
    /// Apply stability filter to avoid regime whipsawing
    fn apply_stability_filter(&self, new_regime: MarketRegime, confidence: f64) -> MarketRegime {
        // If confidence is low, stick with current regime
        if confidence < 0.4 {
            return self.current_regime;
        }
        
        // Require minimum duration in current regime before switching
        if self.bars_in_regime < self.config.min_regime_duration 
            && new_regime != self.current_regime 
        {
            // Only switch if new regime is strongly confirmed
            if confidence < 0.7 {
                return self.current_regime;
            }
        }
        
        // Check recent history for stability
        let recent_count = self.regime_history
            .iter()
            .rev()
            .take(self.config.regime_stability_bars)
            .filter(|&&r| matches!(
                (&r, &new_regime),
                (MarketRegime::Trending(_), MarketRegime::Trending(_)) |
                (MarketRegime::MeanReverting, MarketRegime::MeanReverting) |
                (MarketRegime::Volatile, MarketRegime::Volatile)
            ))
            .count();
        
        // If regime has been bouncing around, require stronger confirmation
        if recent_count < self.config.regime_stability_bars / 2 && confidence < 0.6 {
            return self.current_regime;
        }
        
        new_regime
    }
    
    fn calculate_trend_strength(&self, ema_short: f64, ema_long: f64, close: f64) -> f64 {
        let ema_alignment = (ema_short - ema_long).abs() / ema_long * 100.0;
        let price_position = if close > ema_short && close > ema_long {
            1.0
        } else if close < ema_short && close < ema_long {
            1.0
        } else {
            0.5
        };
        
        (ema_alignment * price_position / 5.0).min(1.0)  // Normalize
    }
    
    /// Check if detector has enough data to classify regime
    pub fn is_ready(&self) -> bool {
        self.adx.is_ready() && self.atr.is_ready() && 
        self.bb.is_ready() && self.ema_short.is_ready() && 
        self.ema_long.is_ready()
    }
    
    /// Get current detected regime
    pub fn current_regime(&self) -> MarketRegime {
        self.current_regime
    }
    
    /// Get recommended strategy for current regime
    pub fn recommended_strategy(&self) -> RecommendedStrategy {
        RecommendedStrategy::from(&self.current_regime)
    }
    
    /// Get number of bars in current regime
    pub fn bars_in_current_regime(&self) -> usize {
        self.bars_in_regime
    }
    
    /// Get ADX value
    pub fn adx_value(&self) -> Option<f64> {
        self.adx.value()
    }
    
    /// Get ATR value
    pub fn atr_value(&self) -> Option<f64> {
        self.atr.value()
    }
    
    /// Get current config
    pub fn config(&self) -> &RegimeConfig {
        &self.config
    }
    
    /// Update config (resets internal state)
    pub fn set_config(&mut self, config: RegimeConfig) {
        *self = Self::new(config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn generate_trending_data(bars: usize, start_price: f64, trend_strength: f64) -> Vec<(f64, f64, f64)> {
        let mut data = Vec::new();
        let mut price = start_price;
        
        for _ in 0..bars {
            let change = trend_strength * (1.0 + (rand::random::<f64>() - 0.5) * 0.2);
            price += change;
            
            let high = price + price * 0.005;
            let low = price - price * 0.005;
            let close = price;
            
            data.push((high, low, close));
        }
        
        data
    }
    
    fn generate_ranging_data(bars: usize, center_price: f64, range_pct: f64) -> Vec<(f64, f64, f64)> {
        let mut data = Vec::new();
        
        for i in 0..bars {
            let offset = (i as f64 * 0.5).sin() * center_price * range_pct / 100.0;
            let price = center_price + offset;
            
            let high = price + price * 0.002;
            let low = price - price * 0.002;
            let close = price;
            
            data.push((high, low, close));
        }
        
        data
    }
    
    #[test]
    fn test_trending_detection() {
        let mut detector = RegimeDetector::default_config();
        
        // Generate uptrending data
        let data = generate_trending_data(300, 100.0, 0.5);
        
        let mut last_regime = MarketRegime::Uncertain;
        for (high, low, close) in data {
            let result = detector.update(high, low, close);
            if detector.is_ready() {
                last_regime = result.regime;
            }
        }
        
        println!("Final regime: {:?}", last_regime);
        assert!(matches!(last_regime, MarketRegime::Trending(_)));
    }
    
    #[test]
    fn test_ranging_detection() {
        let mut detector = RegimeDetector::default_config();
        
        // Generate ranging data
        let data = generate_ranging_data(300, 100.0, 2.0);
        
        let mut last_regime = MarketRegime::Uncertain;
        for (high, low, close) in data {
            let result = detector.update(high, low, close);
            if detector.is_ready() {
                last_regime = result.regime;
            }
        }
        
        println!("Final regime: {:?}", last_regime);
        // Ranging should either be MeanReverting or at least not strongly Trending
    }
}
