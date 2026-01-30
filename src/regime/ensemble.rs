//! Ensemble Regime Detector
//!
//! Combines multiple regime detection methods:
//! 1. Technical Indicators (ADX, Bollinger Bands, ATR) - Rule-based
//! 2. Hidden Markov Model - Statistical/probabilistic
//!
//! The ensemble approach provides more robust regime detection by:
//! - Reducing false positives when methods disagree
//! - Increasing confidence when methods agree
//! - Leveraging different strengths of each approach

use super::{
    detector::RegimeDetector,
    hmm::HMMRegimeDetector,
    types::{MarketRegime, RegimeConfidence, RegimeConfig},
};

#[cfg(test)]
use super::types::TrendDirection;

/// Configuration for ensemble detector
#[derive(Debug, Clone)]
pub struct EnsembleConfig {
    /// Weight for technical indicator detector (0.0 - 1.0)
    pub indicator_weight: f64,
    /// Weight for HMM detector (0.0 - 1.0)
    pub hmm_weight: f64,
    /// Minimum agreement threshold to declare a regime
    pub agreement_threshold: f64,
    /// Use HMM only after warmup (more conservative)
    pub require_hmm_warmup: bool,
    /// Boost confidence when both methods agree
    pub agreement_confidence_boost: f64,
    /// Reduce confidence when methods disagree
    pub disagreement_confidence_penalty: f64,
}

impl Default for EnsembleConfig {
    fn default() -> Self {
        Self {
            indicator_weight: 0.6, // Slightly favor indicators (faster response)
            hmm_weight: 0.4,
            agreement_threshold: 0.5,
            require_hmm_warmup: true,
            agreement_confidence_boost: 0.15,
            disagreement_confidence_penalty: 0.2,
        }
    }
}

impl EnsembleConfig {
    /// Equal weighting between methods
    pub fn balanced() -> Self {
        Self {
            indicator_weight: 0.5,
            hmm_weight: 0.5,
            ..Default::default()
        }
    }

    /// Favor HMM (more statistical)
    pub fn hmm_focused() -> Self {
        Self {
            indicator_weight: 0.3,
            hmm_weight: 0.7,
            agreement_threshold: 0.6,
            ..Default::default()
        }
    }

    /// Favor indicators (faster response)
    pub fn indicator_focused() -> Self {
        Self {
            indicator_weight: 0.7,
            hmm_weight: 0.3,
            agreement_threshold: 0.4,
            ..Default::default()
        }
    }
}

/// Result from ensemble detection
#[derive(Debug, Clone)]
pub struct EnsembleResult {
    /// Final regime determination
    pub regime: MarketRegime,
    /// Combined confidence
    pub confidence: f64,
    /// Whether methods agree
    pub methods_agree: bool,
    /// Indicator-based result
    pub indicator_result: RegimeConfidence,
    /// HMM-based result
    pub hmm_result: RegimeConfidence,
    /// Individual method regimes for debugging
    pub indicator_regime: MarketRegime,
    pub hmm_regime: MarketRegime,
}

impl EnsembleResult {
    /// Convert to standard RegimeConfidence
    pub fn to_regime_confidence(&self) -> RegimeConfidence {
        RegimeConfidence::new(self.regime, self.confidence)
    }
}

/// Ensemble regime detector combining multiple methods
#[derive(Debug)]
pub struct EnsembleRegimeDetector {
    config: EnsembleConfig,

    /// Technical indicator-based detector
    indicator_detector: RegimeDetector,

    /// Hidden Markov Model detector
    hmm_detector: HMMRegimeDetector,

    /// Current ensemble regime
    current_regime: MarketRegime,

    /// Track agreement history
    agreement_history: Vec<bool>,
}

impl EnsembleRegimeDetector {
    pub fn new(ensemble_config: EnsembleConfig, indicator_config: RegimeConfig) -> Self {
        Self {
            config: ensemble_config,
            indicator_detector: RegimeDetector::new(indicator_config),
            hmm_detector: HMMRegimeDetector::crypto_optimized(),
            current_regime: MarketRegime::Uncertain,
            agreement_history: Vec::with_capacity(100),
        }
    }

    /// Create with default configs
    pub fn default_config() -> Self {
        Self::new(EnsembleConfig::default(), RegimeConfig::crypto_optimized())
    }

    /// Create balanced ensemble
    pub fn balanced() -> Self {
        Self::new(EnsembleConfig::balanced(), RegimeConfig::crypto_optimized())
    }

    /// Update with new OHLC data
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> EnsembleResult {
        // Update both detectors
        let indicator_result = self.indicator_detector.update(high, low, close);
        let hmm_result = self.hmm_detector.update_ohlc(high, low, close);

        // Get individual regimes
        let indicator_regime = indicator_result.regime;
        let hmm_regime = hmm_result.regime;

        // Check if HMM is warmed up
        let hmm_ready = self.hmm_detector.is_ready();

        // Determine if methods agree
        let methods_agree = self.regimes_agree(indicator_regime, hmm_regime);

        // Track agreement
        self.agreement_history.push(methods_agree);
        if self.agreement_history.len() > 100 {
            self.agreement_history.remove(0);
        }

        // Calculate combined regime and confidence
        let (regime, confidence) = if self.config.require_hmm_warmup && !hmm_ready {
            // Use only indicators until HMM is ready
            (indicator_regime, indicator_result.confidence)
        } else {
            self.combine_results(
                indicator_regime,
                indicator_result.confidence,
                hmm_regime,
                hmm_result.confidence,
                methods_agree,
            )
        };

        self.current_regime = regime;

        EnsembleResult {
            regime,
            confidence,
            methods_agree,
            indicator_result,
            hmm_result,
            indicator_regime,
            hmm_regime,
        }
    }

    /// Check if two regimes agree (same category)
    fn regimes_agree(&self, r1: MarketRegime, r2: MarketRegime) -> bool {
        match (r1, r2) {
            (MarketRegime::Trending(_), MarketRegime::Trending(_)) => true,
            (MarketRegime::MeanReverting, MarketRegime::MeanReverting) => true,
            (MarketRegime::Volatile, MarketRegime::Volatile) => true,
            (MarketRegime::Uncertain, MarketRegime::Uncertain) => true,
            _ => false,
        }
    }

    /// Check if regimes agree on direction too
    fn regimes_agree_direction(&self, r1: MarketRegime, r2: MarketRegime) -> bool {
        match (r1, r2) {
            (MarketRegime::Trending(d1), MarketRegime::Trending(d2)) => d1 == d2,
            (MarketRegime::MeanReverting, MarketRegime::MeanReverting) => true,
            (MarketRegime::Volatile, MarketRegime::Volatile) => true,
            (MarketRegime::Uncertain, MarketRegime::Uncertain) => true,
            _ => false,
        }
    }

    /// Combine results from both methods
    fn combine_results(
        &self,
        indicator_regime: MarketRegime,
        indicator_conf: f64,
        hmm_regime: MarketRegime,
        hmm_conf: f64,
        agree: bool,
    ) -> (MarketRegime, f64) {
        let w_ind = self.config.indicator_weight;
        let w_hmm = self.config.hmm_weight;

        // Weighted confidence
        let mut combined_conf = w_ind * indicator_conf + w_hmm * hmm_conf;

        // Adjust confidence based on agreement
        if agree {
            // Boost confidence when methods agree
            combined_conf += self.config.agreement_confidence_boost;

            // Extra boost if they agree on direction too
            if self.regimes_agree_direction(indicator_regime, hmm_regime) {
                combined_conf += 0.05;
            }
        } else {
            // Penalty when methods disagree
            combined_conf -= self.config.disagreement_confidence_penalty;
        }

        combined_conf = combined_conf.clamp(0.0, 1.0);

        // Determine final regime
        let regime = if agree {
            // Use the regime they agree on (prefer indicator's direction if trending)
            match indicator_regime {
                MarketRegime::Trending(_) => indicator_regime,
                _ => indicator_regime,
            }
        } else if combined_conf < self.config.agreement_threshold {
            // Low confidence due to disagreement - be conservative
            MarketRegime::Uncertain
        } else {
            // Use higher-weighted method's regime
            if w_ind >= w_hmm {
                indicator_regime
            } else {
                hmm_regime
            }
        };

        (regime, combined_conf)
    }

    /// Get current regime
    pub fn current_regime(&self) -> MarketRegime {
        self.current_regime
    }

    /// Get agreement rate over recent history
    pub fn agreement_rate(&self) -> f64 {
        if self.agreement_history.is_empty() {
            return 0.0;
        }
        let agrees = self.agreement_history.iter().filter(|&&a| a).count();
        agrees as f64 / self.agreement_history.len() as f64
    }

    /// Check if both detectors are ready
    pub fn is_ready(&self) -> bool {
        self.indicator_detector.is_ready()
            && (!self.config.require_hmm_warmup || self.hmm_detector.is_ready())
    }

    /// Get HMM state probabilities
    pub fn hmm_state_probabilities(&self) -> &[f64] {
        self.hmm_detector.state_probabilities()
    }

    /// Get HMM expected regime duration
    pub fn expected_regime_duration(&self) -> f64 {
        self.hmm_detector
            .expected_regime_duration(self.hmm_detector.current_state_index())
    }

    /// Get detailed status for monitoring
    pub fn status(&self) -> EnsembleStatus {
        EnsembleStatus {
            current_regime: self.current_regime,
            indicator_ready: self.indicator_detector.is_ready(),
            hmm_ready: self.hmm_detector.is_ready(),
            agreement_rate: self.agreement_rate(),
            hmm_state_probs: self.hmm_detector.state_probabilities().to_vec(),
            expected_duration: self.expected_regime_duration(),
        }
    }
}

/// Status information for monitoring
#[derive(Debug, Clone)]
pub struct EnsembleStatus {
    pub current_regime: MarketRegime,
    pub indicator_ready: bool,
    pub hmm_ready: bool,
    pub agreement_rate: f64,
    pub hmm_state_probs: Vec<f64>,
    pub expected_duration: f64,
}

impl std::fmt::Display for EnsembleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Regime: {} | Agreement: {:.1}% | HMM Ready: {} | Expected Duration: {:.1} bars",
            self.current_regime,
            self.agreement_rate * 100.0,
            self.hmm_ready,
            self.expected_duration
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensemble_creation() {
        let ensemble = EnsembleRegimeDetector::default_config();
        assert!(!ensemble.is_ready());
    }

    #[test]
    fn test_regimes_agree() {
        let ensemble = EnsembleRegimeDetector::default_config();

        // Same category should agree
        assert!(ensemble.regimes_agree(
            MarketRegime::Trending(TrendDirection::Bullish),
            MarketRegime::Trending(TrendDirection::Bearish)
        ));

        // Different categories should not agree
        assert!(!ensemble.regimes_agree(
            MarketRegime::Trending(TrendDirection::Bullish),
            MarketRegime::MeanReverting
        ));
    }

    #[test]
    fn test_agreement_rate() {
        let mut ensemble = EnsembleRegimeDetector::default_config();

        // Simulate some updates
        let mut price = 100.0;
        for i in 0..50 {
            price *= if i % 2 == 0 { 1.01 } else { 0.99 };
            ensemble.update(price * 1.01, price * 0.99, price);
        }

        // Should have some agreement rate
        let rate = ensemble.agreement_rate();
        assert!(rate >= 0.0 && rate <= 1.0);
    }

    #[test]
    fn test_bull_market_agreement() {
        let mut ensemble = EnsembleRegimeDetector::default_config();

        // Strong bull market - both methods should eventually agree
        let mut price = 100.0;
        for _ in 0..200 {
            price *= 1.005; // Consistent upward
            let high = price * 1.002;
            let low = price * 0.998;
            ensemble.update(high, low, price);
        }

        let result = ensemble.update(price * 1.002, price * 0.998, price);
        println!("Bull market result: {:?}", result);

        // In a strong trend, agreement rate should be reasonable
        assert!(ensemble.agreement_rate() > 0.3);
    }
}
