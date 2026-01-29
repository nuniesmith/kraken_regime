//! Regime Detection Method Comparison
//! 
//! Compares three approaches to regime detection:
//! 1. Technical Indicators (ADX, BB, ATR)
//! 2. Hidden Markov Model
//! 3. Ensemble (combines both)
//! 
//! Run with: cargo run --example compare_methods

use std::collections::HashMap;

// Import from the crate
mod regime {
    pub mod types;
    pub mod indicators;
    pub mod detector;
    pub mod hmm;
    pub mod ensemble;
    
    pub use types::*;
    pub use detector::RegimeDetector;
    pub use hmm::{HMMRegimeDetector, HMMConfig};
    pub use ensemble::{EnsembleRegimeDetector, EnsembleConfig, EnsembleResult};
}

use regime::*;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     Regime Detection Method Comparison                       ║");
    println!("║     Indicators vs HMM vs Ensemble                            ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    
    // Test on different market scenarios
    test_bull_market();
    println!("\n{}\n", "=".repeat(70));
    
    test_bear_market();
    println!("\n{}\n", "=".repeat(70));
    
    test_volatile_market();
    println!("\n{}\n", "=".repeat(70));
    
    test_ranging_market();
    println!("\n{}\n", "=".repeat(70));
    
    test_mixed_market();
}

fn test_bull_market() {
    println!("📈 TEST: Bull Market (Steady Uptrend)");
    println!("-".repeat(50));
    
    let (indicator_results, hmm_results, ensemble_results) = run_scenario(|i, price| {
        // Steady 0.3% daily gains with small noise
        let noise = ((i as f64 * 0.1).sin() * 0.001) + 0.003;
        price * (1.0 + noise)
    }, 200);
    
    print_results("Bull Market", &indicator_results, &hmm_results, &ensemble_results);
}

fn test_bear_market() {
    println!("📉 TEST: Bear Market (Steady Downtrend)");
    println!("-".repeat(50));
    
    let (indicator_results, hmm_results, ensemble_results) = run_scenario(|i, price| {
        // Steady 0.3% daily losses with noise
        let noise = ((i as f64 * 0.1).sin() * 0.001) - 0.003;
        price * (1.0 + noise)
    }, 200);
    
    print_results("Bear Market", &indicator_results, &hmm_results, &ensemble_results);
}

fn test_volatile_market() {
    println!("🎢 TEST: Volatile Market (Large Swings)");
    println!("-".repeat(50));
    
    let (indicator_results, hmm_results, ensemble_results) = run_scenario(|i, price| {
        // Large alternating swings of ±3-5%
        let direction = if (i / 3) % 2 == 0 { 1.0 } else { -1.0 };
        let magnitude = 0.03 + (i as f64 * 0.2).sin().abs() * 0.02;
        price * (1.0 + direction * magnitude)
    }, 200);
    
    print_results("Volatile Market", &indicator_results, &hmm_results, &ensemble_results);
}

fn test_ranging_market() {
    println!("↔️ TEST: Ranging Market (Sideways/Mean-Reverting)");
    println!("-".repeat(50));
    
    let (indicator_results, hmm_results, ensemble_results) = run_scenario(|i, price| {
        // Oscillate around a mean with small amplitude
        let cycle = (i as f64 * 0.15).sin() * 0.01;
        price * (1.0 + cycle)
    }, 200);
    
    print_results("Ranging Market", &indicator_results, &hmm_results, &ensemble_results);
}

fn test_mixed_market() {
    println!("🔄 TEST: Mixed Market (Regime Transitions)");
    println!("-".repeat(50));
    
    let (indicator_results, hmm_results, ensemble_results) = run_scenario(|i, price| {
        // Phase 1 (0-50): Bull
        // Phase 2 (50-100): Volatile
        // Phase 3 (100-150): Ranging
        // Phase 4 (150-200): Bear
        
        let change = if i < 50 {
            0.003  // Bull
        } else if i < 100 {
            if i % 2 == 0 { 0.03 } else { -0.03 }  // Volatile
        } else if i < 150 {
            (i as f64 * 0.2).sin() * 0.005  // Ranging
        } else {
            -0.003  // Bear
        };
        
        price * (1.0 + change)
    }, 200);
    
    print_results("Mixed Market", &indicator_results, &hmm_results, &ensemble_results);
    
    // Also show transition detection
    println!("\n  Regime Transitions Detected:");
    println!("  - Mixed markets test the ability to detect regime changes");
    println!("  - Ensemble should show lower confidence during transitions");
}

fn run_scenario<F>(price_fn: F, n_bars: usize) -> (Vec<MarketRegime>, Vec<MarketRegime>, Vec<MarketRegime>)
where
    F: Fn(usize, f64) -> f64,
{
    // Create detectors
    let mut indicator = RegimeDetector::crypto_optimized();
    let mut hmm = HMMRegimeDetector::new(HMMConfig {
        min_observations: 30,
        n_states: 3,
        ..HMMConfig::crypto_optimized()
    });
    let mut ensemble = EnsembleRegimeDetector::default_config();
    
    let mut indicator_regimes = Vec::new();
    let mut hmm_regimes = Vec::new();
    let mut ensemble_regimes = Vec::new();
    
    let mut price = 100.0;
    
    for i in 0..n_bars {
        price = price_fn(i, price);
        let high = price * 1.005;
        let low = price * 0.995;
        
        let ind_result = indicator.update(high, low, price);
        let hmm_result = hmm.update_ohlc(high, low, price);
        let ens_result = ensemble.update(high, low, price);
        
        // Only collect after warmup
        if i >= 50 {
            indicator_regimes.push(ind_result.regime);
            hmm_regimes.push(hmm_result.regime);
            ensemble_regimes.push(ens_result.regime);
        }
    }
    
    (indicator_regimes, hmm_regimes, ensemble_regimes)
}

fn print_results(
    scenario: &str,
    indicator: &[MarketRegime],
    hmm: &[MarketRegime],
    ensemble: &[MarketRegime],
) {
    fn count_regimes(regimes: &[MarketRegime]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for r in regimes {
            let key = match r {
                MarketRegime::Trending(TrendDirection::Bullish) => "Trending↑",
                MarketRegime::Trending(TrendDirection::Bearish) => "Trending↓",
                MarketRegime::MeanReverting => "Ranging",
                MarketRegime::Volatile => "Volatile",
                MarketRegime::Uncertain => "Uncertain",
            };
            *counts.entry(key.to_string()).or_insert(0) += 1;
        }
        counts
    }
    
    fn format_counts(counts: &HashMap<String, usize>, total: usize) -> String {
        let mut result = Vec::new();
        for regime in ["Trending↑", "Trending↓", "Ranging", "Volatile", "Uncertain"] {
            if let Some(&count) = counts.get(regime) {
                let pct = count as f64 / total as f64 * 100.0;
                result.push(format!("{}: {:.0}%", regime, pct));
            }
        }
        result.join(", ")
    }
    
    let ind_counts = count_regimes(indicator);
    let hmm_counts = count_regimes(hmm);
    let ens_counts = count_regimes(ensemble);
    
    let total = indicator.len();
    
    println!();
    println!("  📊 Results for {} ({} bars after warmup):", scenario, total);
    println!();
    println!("  ┌─────────────┬────────────────────────────────────────────────┐");
    println!("  │ Method      │ Regime Distribution                           │");
    println!("  ├─────────────┼────────────────────────────────────────────────┤");
    println!("  │ Indicators  │ {:<46} │", format_counts(&ind_counts, total));
    println!("  │ HMM         │ {:<46} │", format_counts(&hmm_counts, total));
    println!("  │ Ensemble    │ {:<46} │", format_counts(&ens_counts, total));
    println!("  └─────────────┴────────────────────────────────────────────────┘");
    
    // Calculate agreement
    let mut agree_count = 0;
    for i in 0..total {
        let ind_type = regime_type(&indicator[i]);
        let hmm_type = regime_type(&hmm[i]);
        if ind_type == hmm_type {
            agree_count += 1;
        }
    }
    let agreement_rate = agree_count as f64 / total as f64 * 100.0;
    
    println!();
    println!("  Agreement Rate (Indicator vs HMM): {:.1}%", agreement_rate);
}

fn regime_type(r: &MarketRegime) -> &str {
    match r {
        MarketRegime::Trending(_) => "Trending",
        MarketRegime::MeanReverting => "Ranging",
        MarketRegime::Volatile => "Volatile",
        MarketRegime::Uncertain => "Uncertain",
    }
}
