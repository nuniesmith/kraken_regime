//! Hidden Markov Model Regime Detection
//! 
//! Implements HMM-based regime detection as described in:
//! - Hamilton, J.D. (1989) "A New Approach to the Economic Analysis of Nonstationary Time Series"
//! 
//! The HMM approach learns regime distributions directly from returns data,
//! making no assumptions about what indicators define each regime.

use std::collections::VecDeque;
use super::types::{MarketRegime, TrendDirection, RegimeConfidence};

/// Configuration for HMM regime detector
#[derive(Debug, Clone)]
pub struct HMMConfig {
    /// Number of hidden states (regimes)
    pub n_states: usize,
    /// Minimum observations before making predictions
    pub min_observations: usize,
    /// Learning rate for online updates (0 = no online learning)
    pub learning_rate: f64,
    /// Smoothing factor for transition probabilities
    pub transition_smoothing: f64,
    /// Window size for return calculations
    pub lookback_window: usize,
    /// Confidence threshold for regime classification
    pub min_confidence: f64,
}

impl Default for HMMConfig {
    fn default() -> Self {
        Self {
            n_states: 3,  // Bull, Bear, High-Vol
            min_observations: 100,
            learning_rate: 0.01,
            transition_smoothing: 0.1,
            lookback_window: 252,  // ~1 year of daily data
            min_confidence: 0.6,
        }
    }
}

impl HMMConfig {
    /// Config optimized for crypto (faster regime changes)
    pub fn crypto_optimized() -> Self {
        Self {
            n_states: 3,
            min_observations: 50,
            learning_rate: 0.02,  // Faster adaptation
            transition_smoothing: 0.05,
            lookback_window: 100,
            min_confidence: 0.5,
        }
    }
    
    /// Conservative config (more stable regimes)
    pub fn conservative() -> Self {
        Self {
            n_states: 2,  // Just bull/bear
            min_observations: 150,
            learning_rate: 0.005,
            transition_smoothing: 0.15,
            lookback_window: 500,
            min_confidence: 0.7,
        }
    }
}

/// Gaussian parameters for a single state
#[derive(Debug, Clone)]
struct GaussianState {
    mean: f64,
    variance: f64,
    /// Running statistics for online updates
    sum: f64,
    sum_sq: f64,
    count: usize,
}

impl GaussianState {
    fn new(mean: f64, variance: f64) -> Self {
        Self {
            mean,
            variance,
            sum: 0.0,
            sum_sq: 0.0,
            count: 0,
        }
    }
    
    /// Probability density function
    fn pdf(&self, x: f64) -> f64 {
        let diff = x - self.mean;
        let exponent = -0.5 * diff * diff / self.variance;
        let normalizer = (2.0 * std::f64::consts::PI * self.variance).sqrt();
        exponent.exp() / normalizer
    }
    
    /// Update statistics with new observation
    fn update(&mut self, x: f64, weight: f64, learning_rate: f64) {
        if learning_rate > 0.0 {
            // Online update using exponential moving average
            self.mean = (1.0 - learning_rate * weight) * self.mean + learning_rate * weight * x;
            let new_var = (x - self.mean).powi(2);
            self.variance = (1.0 - learning_rate * weight) * self.variance + learning_rate * weight * new_var;
            self.variance = self.variance.max(1e-8);  // Prevent zero variance
        }
        
        // Also track running stats
        self.sum += x * weight;
        self.sum_sq += x * x * weight;
        self.count += 1;
    }
}

/// Hidden Markov Model for regime detection
#[derive(Debug)]
pub struct HMMRegimeDetector {
    config: HMMConfig,
    
    /// Gaussian emission distributions for each state
    states: Vec<GaussianState>,
    
    /// Transition probability matrix A[i][j] = P(state_j | state_i)
    transition_matrix: Vec<Vec<f64>>,
    
    /// Initial state probabilities
    initial_probs: Vec<f64>,
    
    /// Current state probabilities (filtered)
    state_probs: Vec<f64>,
    
    /// History of returns for batch updates
    returns_history: VecDeque<f64>,
    
    /// History of prices for return calculation
    prices: VecDeque<f64>,
    
    /// Current most likely state
    current_state: usize,
    
    /// Confidence in current state
    current_confidence: f64,
    
    /// Total observations processed
    n_observations: usize,
    
    /// Last detected regime
    last_regime: MarketRegime,
}

impl HMMRegimeDetector {
    pub fn new(config: HMMConfig) -> Self {
        let n = config.n_states;
        
        // Initialize states with reasonable priors for financial returns
        // State 0: Bull (positive returns, low vol)
        // State 1: Bear (negative returns, higher vol)  
        // State 2: High Vol (any direction, high vol)
        let states = match n {
            2 => vec![
                GaussianState::new(0.001, 0.0001),   // Bull: ~0.1% daily, low vol
                GaussianState::new(-0.001, 0.0004),  // Bear: -0.1% daily, higher vol
            ],
            3 => vec![
                GaussianState::new(0.001, 0.0001),   // Bull: positive, low vol
                GaussianState::new(-0.001, 0.0002),  // Bear: negative, medium vol
                GaussianState::new(0.0, 0.0009),     // High Vol: neutral, high vol
            ],
            _ => (0..n).map(|i| {
                let mean = (i as f64 - n as f64 / 2.0) * 0.001;
                let var = 0.0001 * (1.0 + i as f64);
                GaussianState::new(mean, var)
            }).collect(),
        };
        
        // Initialize transition matrix with slight persistence
        // Higher diagonal = states tend to persist
        let mut transition_matrix = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    transition_matrix[i][j] = 0.9;  // 90% stay in same state
                } else {
                    transition_matrix[i][j] = 0.1 / (n - 1) as f64;
                }
            }
        }
        
        // Equal initial probabilities
        let initial_probs = vec![1.0 / n as f64; n];
        let state_probs = initial_probs.clone();
        
        Self {
            config: config.clone(),
            states,
            transition_matrix,
            initial_probs,
            state_probs,
            returns_history: VecDeque::with_capacity(config.lookback_window),
            prices: VecDeque::with_capacity(10),
            current_state: 0,
            current_confidence: 0.0,
            n_observations: 0,
            last_regime: MarketRegime::Uncertain,
        }
    }
    
    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(HMMConfig::default())
    }
    
    /// Create optimized for crypto
    pub fn crypto_optimized() -> Self {
        Self::new(HMMConfig::crypto_optimized())
    }
    
    /// Update with new price and get regime
    pub fn update(&mut self, close: f64) -> RegimeConfidence {
        // Calculate log return
        if let Some(&prev_close) = self.prices.back() {
            let log_return = (close / prev_close).ln();
            self.process_return(log_return);
        }
        
        // Store price
        self.prices.push_back(close);
        if self.prices.len() > 10 {
            self.prices.pop_front();
        }
        
        // Return current regime
        self.get_regime_confidence()
    }
    
    /// Update with OHLC data
    pub fn update_ohlc(&mut self, _high: f64, _low: f64, close: f64) -> RegimeConfidence {
        self.update(close)
    }
    
    /// Process a single return observation
    fn process_return(&mut self, ret: f64) {
        self.n_observations += 1;
        
        // Store return
        self.returns_history.push_back(ret);
        if self.returns_history.len() > self.config.lookback_window {
            self.returns_history.pop_front();
        }
        
        // Forward algorithm step (filtering)
        self.forward_step(ret);
        
        // Update state parameters if we have enough data
        if self.n_observations > self.config.min_observations && self.config.learning_rate > 0.0 {
            self.online_parameter_update(ret);
        }
        
        // Periodically re-estimate with Baum-Welch if we have enough data
        if self.n_observations > 0 && 
           self.n_observations % (self.config.lookback_window / 2) == 0 &&
           self.returns_history.len() >= self.config.min_observations {
            self.baum_welch_update();
        }
    }
    
    /// Forward algorithm step - update state probabilities given new observation
    fn forward_step(&mut self, ret: f64) {
        let n = self.config.n_states;
        let mut new_probs = vec![0.0; n];
        
        // Calculate emission probabilities
        let emissions: Vec<f64> = self.states.iter()
            .map(|s| s.pdf(ret))
            .collect();
        
        // Forward step: P(state_j | obs) ∝ P(obs | state_j) * Σᵢ P(state_j | state_i) * P(state_i)
        for j in 0..n {
            let mut sum = 0.0;
            for i in 0..n {
                sum += self.transition_matrix[i][j] * self.state_probs[i];
            }
            new_probs[j] = emissions[j] * sum;
        }
        
        // Normalize
        let total: f64 = new_probs.iter().sum();
        if total > 1e-300 {
            for p in &mut new_probs {
                *p /= total;
            }
        } else {
            // Reset to uniform if probabilities collapse
            new_probs = vec![1.0 / n as f64; n];
        }
        
        self.state_probs = new_probs;
        
        // Update current state and confidence
        let (max_idx, max_prob) = self.state_probs.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
        
        self.current_state = max_idx;
        self.current_confidence = *max_prob;
    }
    
    /// Online parameter update using soft assignments
    fn online_parameter_update(&mut self, ret: f64) {
        let lr = self.config.learning_rate;
        
        for (i, state) in self.states.iter_mut().enumerate() {
            let weight = self.state_probs[i];
            state.update(ret, weight, lr);
        }
        
        // Update transition matrix (soft transitions)
        // This is a simplified online update
        let smoothing = self.config.transition_smoothing;
        for i in 0..self.config.n_states {
            for j in 0..self.config.n_states {
                let target = if i == j { 0.9 } else { 0.1 / (self.config.n_states - 1) as f64 };
                self.transition_matrix[i][j] = 
                    (1.0 - smoothing) * self.transition_matrix[i][j] + 
                    smoothing * target;
            }
        }
    }
    
    /// Baum-Welch algorithm for batch parameter re-estimation
    fn baum_welch_update(&mut self) {
        let returns: Vec<f64> = self.returns_history.iter().copied().collect();
        if returns.len() < self.config.min_observations {
            return;
        }
        
        let n = self.config.n_states;
        let t = returns.len();
        
        // Forward pass
        let mut alpha = vec![vec![0.0; n]; t];
        
        // Initialize
        for j in 0..n {
            alpha[0][j] = self.initial_probs[j] * self.states[j].pdf(returns[0]);
        }
        self.normalize_vec(&mut alpha[0]);
        
        // Forward
        for time in 1..t {
            for j in 0..n {
                let mut sum = 0.0;
                for i in 0..n {
                    sum += alpha[time - 1][i] * self.transition_matrix[i][j];
                }
                alpha[time][j] = sum * self.states[j].pdf(returns[time]);
            }
            self.normalize_vec(&mut alpha[time]);
        }
        
        // Backward pass
        let mut beta = vec![vec![1.0; n]; t];
        
        for time in (0..t - 1).rev() {
            for i in 0..n {
                let mut sum = 0.0;
                for j in 0..n {
                    sum += self.transition_matrix[i][j] * 
                           self.states[j].pdf(returns[time + 1]) * 
                           beta[time + 1][j];
                }
                beta[time][i] = sum;
            }
            self.normalize_vec(&mut beta[time]);
        }
        
        // Compute gamma (state occupancy probabilities)
        let mut gamma = vec![vec![0.0; n]; t];
        for time in 0..t {
            let mut sum = 0.0;
            for j in 0..n {
                gamma[time][j] = alpha[time][j] * beta[time][j];
                sum += gamma[time][j];
            }
            if sum > 1e-300 {
                for j in 0..n {
                    gamma[time][j] /= sum;
                }
            }
        }
        
        // Re-estimate emission parameters
        for j in 0..n {
            let mut weight_sum = 0.0;
            let mut mean_sum = 0.0;
            let mut var_sum = 0.0;
            
            for time in 0..t {
                let w = gamma[time][j];
                weight_sum += w;
                mean_sum += w * returns[time];
            }
            
            if weight_sum > 1e-8 {
                let new_mean = mean_sum / weight_sum;
                
                for time in 0..t {
                    let w = gamma[time][j];
                    var_sum += w * (returns[time] - new_mean).powi(2);
                }
                
                let new_var = (var_sum / weight_sum).max(1e-8);
                
                // Blend with existing parameters (prevents sudden jumps)
                let blend = 0.3;
                self.states[j].mean = (1.0 - blend) * self.states[j].mean + blend * new_mean;
                self.states[j].variance = (1.0 - blend) * self.states[j].variance + blend * new_var;
            }
        }
    }
    
    /// Helper to normalize a probability vector
    fn normalize_vec(&self, vec: &mut [f64]) {
        let sum: f64 = vec.iter().sum();
        if sum > 1e-300 {
            for v in vec.iter_mut() {
                *v /= sum;
            }
        }
    }
    
    /// Get current regime with confidence
    pub fn get_regime_confidence(&self) -> RegimeConfidence {
        if self.n_observations < self.config.min_observations {
            return RegimeConfidence::new(MarketRegime::Uncertain, 0.0);
        }
        
        let regime = self.state_to_regime(self.current_state);
        let confidence = self.current_confidence;
        
        RegimeConfidence::with_metrics(
            regime,
            confidence,
            self.states[self.current_state].mean * 100.0 * 252.0,  // Annualized return %
            self.states[self.current_state].variance.sqrt() * 100.0 * 252.0_f64.sqrt(),  // Annualized vol %
            0.0,  // No trend strength in HMM
        )
    }
    
    /// Map state index to MarketRegime
    fn state_to_regime(&self, state: usize) -> MarketRegime {
        let state_params = &self.states[state];
        let mean = state_params.mean;
        let vol = state_params.variance.sqrt();
        
        // Classify based on learned parameters
        let is_high_vol = vol > 0.02;  // > 2% daily vol
        let is_positive = mean > 0.0005;  // > 0.05% daily
        let is_negative = mean < -0.0005;
        
        if is_high_vol {
            MarketRegime::Volatile
        } else if is_positive {
            MarketRegime::Trending(TrendDirection::Bullish)
        } else if is_negative {
            MarketRegime::Trending(TrendDirection::Bearish)
        } else {
            MarketRegime::MeanReverting  // Low vol, neutral returns = ranging
        }
    }
    
    /// Get state probabilities
    pub fn state_probabilities(&self) -> &[f64] {
        &self.state_probs
    }
    
    /// Get state parameters (mean, variance) for inspection
    pub fn state_parameters(&self) -> Vec<(f64, f64)> {
        self.states.iter()
            .map(|s| (s.mean, s.variance))
            .collect()
    }
    
    /// Get transition matrix
    pub fn transition_matrix(&self) -> &Vec<Vec<f64>> {
        &self.transition_matrix
    }
    
    /// Get current state index
    pub fn current_state_index(&self) -> usize {
        self.current_state
    }
    
    /// Check if model is warmed up
    pub fn is_ready(&self) -> bool {
        self.n_observations >= self.config.min_observations
    }
    
    /// Get expected regime duration (from transition matrix)
    pub fn expected_regime_duration(&self, state: usize) -> f64 {
        if state < self.config.n_states {
            // Expected duration = 1 / (1 - P(stay in state))
            1.0 / (1.0 - self.transition_matrix[state][state])
        } else {
            0.0
        }
    }
    
    /// Predict most likely next state
    pub fn predict_next_state(&self) -> (usize, f64) {
        let mut next_probs = vec![0.0; self.config.n_states];
        
        for j in 0..self.config.n_states {
            for i in 0..self.config.n_states {
                next_probs[j] += self.transition_matrix[i][j] * self.state_probs[i];
            }
        }
        
        let (max_idx, max_prob) = next_probs.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
        
        (max_idx, *max_prob)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hmm_initialization() {
        let hmm = HMMRegimeDetector::default_config();
        assert_eq!(hmm.config.n_states, 3);
        assert!(!hmm.is_ready());
    }
    
    #[test]
    fn test_hmm_warmup() {
        let mut hmm = HMMRegimeDetector::new(HMMConfig {
            min_observations: 10,
            ..Default::default()
        });
        
        // Feed some returns
        let prices = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0, 106.0, 107.0, 108.0];
        for price in prices {
            hmm.update(price);
        }
        
        assert!(hmm.is_ready());
    }
    
    #[test]
    fn test_bull_market_detection() {
        let mut hmm = HMMRegimeDetector::new(HMMConfig {
            min_observations: 20,
            n_states: 2,
            ..Default::default()
        });
        
        // Simulate bull market: steady upward movement
        let mut price = 100.0;
        for _ in 0..50 {
            price *= 1.002;  // ~0.2% daily gain
            hmm.update(price);
        }
        
        let regime = hmm.get_regime_confidence();
        println!("Bull market regime: {:?}", regime);
        // Should detect trending/bullish
    }
    
    #[test]
    fn test_volatile_market_detection() {
        let mut hmm = HMMRegimeDetector::new(HMMConfig {
            min_observations: 20,
            n_states: 3,
            ..Default::default()
        });
        
        // Simulate volatile market: large swings
        let mut price = 100.0;
        for i in 0..50 {
            let change = if i % 2 == 0 { 1.03 } else { 0.97 };  // ±3% swings
            price *= change;
            hmm.update(price);
        }
        
        let regime = hmm.get_regime_confidence();
        println!("Volatile market regime: {:?}", regime);
        // Should detect volatile
    }
    
    #[test]
    fn test_state_probabilities_sum_to_one() {
        let mut hmm = HMMRegimeDetector::default_config();
        
        let mut price = 100.0;
        for _ in 0..150 {
            price *= 1.001;
            hmm.update(price);
        }
        
        let probs = hmm.state_probabilities();
        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 0.001, "Probabilities should sum to 1");
    }
}
