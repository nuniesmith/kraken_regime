//! Regime Detection Module
//!
//! Detects market regime (Trending, Mean-Reverting, Volatile) to enable
//! strategy switching based on current market conditions.
//!
//! Three detection approaches available:
//! 1. **Technical Indicators** (RegimeDetector) - Fast, rule-based using ADX/BB/ATR
//! 2. **Hidden Markov Model** (HMMRegimeDetector) - Statistical, learns from returns
//! 3. **Ensemble** (EnsembleRegimeDetector) - Combines both for robustness
//!
//! Based on research showing regime-aware strategies outperform static ones by 20-40%

mod detector;
mod ensemble;
mod hmm;
mod indicators;
mod types;

pub use detector::RegimeDetector;
pub use ensemble::{EnsembleConfig, EnsembleRegimeDetector, EnsembleResult, EnsembleStatus};
pub use hmm::{HMMConfig, HMMRegimeDetector};
pub use indicators::*;
pub use types::*;
