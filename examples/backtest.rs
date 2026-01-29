//! Backtest Example
//!
//! Test the regime-aware trading system on historical data to evaluate
//! strategy performance across different market conditions.
//!
//! Run with: cargo run --example backtest

use kraken_regime::prelude::*;
use kraken_regime::regime::{RegimeConfig, RegimeDetector};
use kraken_regime::strategy::router::RouterStats;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct OhlcRecord {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug, Default)]
struct BacktestResults {
    total_trades: u32,
    winning_trades: u32,
    losing_trades: u32,
    total_pnl: f64,
    max_drawdown: f64,
    regime_distribution: HashMap<String, u32>,
    strategy_usage: RouterStats,
}

impl BacktestResults {
    fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            return 0.0;
        }
        (self.winning_trades as f64 / self.total_trades as f64) * 100.0
    }
    
    fn print_summary(&self) {
        println!("\n═══════════════════════════════════════════════════════");
        println!("                    BACKTEST RESULTS                    ");
        println!("═══════════════════════════════════════════════════════\n");
        
        println!("📊 Trade Statistics:");
        println!("   Total Trades:    {}", self.total_trades);
        println!("   Winning Trades:  {} ({:.1}%)", self.winning_trades, self.win_rate());
        println!("   Losing Trades:   {}", self.losing_trades);
        println!("   Total P&L:       ${:.2}", self.total_pnl);
        println!("   Max Drawdown:    ${:.2}", self.max_drawdown);
        
        println!("\n🔄 Regime Distribution:");
        for (regime, count) in &self.regime_distribution {
            println!("   {:20} {} bars", regime, count);
        }
        
        println!("\n📈 Strategy Usage:");
        println!("   Trend Following:  {} signals", self.strategy_usage.trend_following_signals);
        println!("   Mean Reversion:   {} signals", self.strategy_usage.mean_reversion_signals);
        println!("   No Trade:         {} periods", self.strategy_usage.no_trade_periods);
        
        println!("\n═══════════════════════════════════════════════════════\n");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Kraken Regime-Aware Trading Backtest\n");
    
    // Test with synthetic data first to verify system works
    println!("Testing with synthetic data...\n");
    
    // Generate synthetic market data
    let trending_data = generate_trending_market(500, 50000.0, 100.0);
    let ranging_data = generate_ranging_market(500, 50000.0, 2.0);
    let volatile_data = generate_volatile_market(500, 50000.0, 5.0);
    
    // Test regime detection on each type
    test_regime_detection("Trending Market", &trending_data);
    test_regime_detection("Ranging Market", &ranging_data);
    test_regime_detection("Volatile Market", &volatile_data);
    
    // Run full backtest with mixed conditions
    println!("\n📈 Running full backtest with mixed market conditions...\n");
    
    let mixed_data = generate_mixed_market(2000, 50000.0);
    let results = run_backtest(&mixed_data);
    results.print_summary();
    
    // Compare with static strategies
    println!("📊 Comparison with static strategies:\n");
    compare_strategies(&mixed_data);
    
    Ok(())
}

/// Test regime detection accuracy
fn test_regime_detection(name: &str, data: &[Candle]) {
    let mut detector = RegimeDetector::crypto_optimized();
    let mut regime_counts: HashMap<String, u32> = HashMap::new();
    
    for candle in data {
        let result = detector.update(candle.high, candle.low, candle.close);
        if detector.is_ready() {
            *regime_counts.entry(result.regime.to_string()).or_insert(0) += 1;
        }
    }
    
    println!("📊 {} Detection Results:", name);
    for (regime, count) in &regime_counts {
        let pct = (*count as f64 / data.len() as f64) * 100.0;
        println!("   {:25} {:4} bars ({:5.1}%)", regime, count, pct);
    }
    println!();
}

/// Run full backtest
fn run_backtest(data: &[Candle]) -> BacktestResults {
    let config = KrakenIntegrationConfig::default();
    let mut trader = KrakenRegimeTrader::new(config);
    
    let mut results = BacktestResults::default();
    let mut position: Option<Position> = None;
    let mut equity = 10000.0;
    let mut peak_equity = equity;
    
    for candle in data {
        // Update regime counts
        if let Some(regime) = trader.get_regime("BTC/USD") {
            *results.regime_distribution.entry(regime.to_string()).or_insert(0) += 1;
        }
        
        // Process candle
        if let Some(action) = trader.process_candle("BTC/USD", candle) {
            results.strategy_usage.record_signal(&kraken_regime::strategy::router::RoutedSignal {
                signal: match action.action {
                    TradeType::Buy => Signal::Buy,
                    TradeType::Sell => Signal::Sell,
                    TradeType::Hold => Signal::Hold,
                },
                source_strategy: match action.source_strategy.as_str() {
                    "Trend Following" => kraken_regime::strategy::router::ActiveStrategy::TrendFollowing,
                    "Mean Reversion" => kraken_regime::strategy::router::ActiveStrategy::MeanReversion,
                    _ => kraken_regime::strategy::router::ActiveStrategy::NoTrade,
                },
                regime: trader.get_regime("BTC/USD").unwrap_or(MarketRegime::Uncertain),
                confidence: action.confidence,
                position_size_factor: action.size_factor,
                reason: action.reason.clone(),
                stop_loss: action.stop_loss,
                take_profit: action.take_profit,
            });
            
            match action.action {
                TradeType::Buy if position.is_none() => {
                    let size = equity * 0.01 * action.size_factor;  // 1% risk adjusted by factor
                    position = Some(Position {
                        entry_price: candle.close,
                        size,
                        stop_loss: action.stop_loss,
                        take_profit: action.take_profit,
                    });
                }
                TradeType::Sell if position.is_some() => {
                    if let Some(pos) = position.take() {
                        let pnl = (candle.close - pos.entry_price) / pos.entry_price * pos.size;
                        equity += pnl;
                        results.total_pnl += pnl;
                        results.total_trades += 1;
                        
                        if pnl > 0.0 {
                            results.winning_trades += 1;
                        } else {
                            results.losing_trades += 1;
                        }
                        
                        peak_equity = peak_equity.max(equity);
                        let drawdown = peak_equity - equity;
                        results.max_drawdown = results.max_drawdown.max(drawdown);
                    }
                }
                _ => {}
            }
            
            // Check stops on existing position
            if let Some(ref pos) = position {
                if let Some(stop) = pos.stop_loss {
                    if candle.low <= stop {
                        let pnl = (stop - pos.entry_price) / pos.entry_price * pos.size;
                        equity += pnl;
                        results.total_pnl += pnl;
                        results.total_trades += 1;
                        results.losing_trades += 1;
                        position = None;
                        
                        let drawdown = peak_equity - equity;
                        results.max_drawdown = results.max_drawdown.max(drawdown);
                    }
                }
                if let Some(tp) = pos.take_profit {
                    if candle.high >= tp {
                        let pnl = (tp - pos.entry_price) / pos.entry_price * pos.size;
                        equity += pnl;
                        results.total_pnl += pnl;
                        results.total_trades += 1;
                        results.winning_trades += 1;
                        position = None;
                        
                        peak_equity = peak_equity.max(equity);
                    }
                }
            }
        }
    }
    
    results
}

/// Compare regime-aware strategy vs static strategies
fn compare_strategies(data: &[Candle]) {
    // Regime-aware backtest
    let regime_aware = run_backtest(data);
    
    // Static trend-following (simple MA crossover)
    let trend_only = run_static_trend_backtest(data);
    
    // Static mean-reversion (simple BB)
    let mean_rev_only = run_static_mean_reversion_backtest(data);
    
    println!("┌─────────────────────┬─────────────┬─────────────┬─────────────┐");
    println!("│ Metric              │ Regime-Aware│ Trend Only  │ MeanRev Only│");
    println!("├─────────────────────┼─────────────┼─────────────┼─────────────┤");
    println!("│ Total Trades        │ {:>11} │ {:>11} │ {:>11} │", 
        regime_aware.total_trades, trend_only.total_trades, mean_rev_only.total_trades);
    println!("│ Win Rate            │ {:>10.1}% │ {:>10.1}% │ {:>10.1}% │",
        regime_aware.win_rate(), trend_only.win_rate(), mean_rev_only.win_rate());
    println!("│ Total P&L           │ ${:>10.2} │ ${:>10.2} │ ${:>10.2} │",
        regime_aware.total_pnl, trend_only.total_pnl, mean_rev_only.total_pnl);
    println!("│ Max Drawdown        │ ${:>10.2} │ ${:>10.2} │ ${:>10.2} │",
        regime_aware.max_drawdown, trend_only.max_drawdown, mean_rev_only.max_drawdown);
    println!("└─────────────────────┴─────────────┴─────────────┴─────────────┘");
}

/// Run static trend-following backtest
fn run_static_trend_backtest(data: &[Candle]) -> BacktestResults {
    use kraken_regime::regime::indicators::EMA;
    
    let mut results = BacktestResults::default();
    let mut ema_short = EMA::new(50);
    let mut ema_long = EMA::new(200);
    let mut position: Option<Position> = None;
    let mut equity = 10000.0;
    let mut peak_equity = equity;
    
    for candle in data {
        let short = ema_short.update(candle.close);
        let long = ema_long.update(candle.close);
        
        if let (Some(s), Some(l)) = (short, long) {
            // Golden cross = buy
            if s > l && position.is_none() {
                position = Some(Position {
                    entry_price: candle.close,
                    size: equity * 0.01,
                    stop_loss: Some(candle.close * 0.95),
                    take_profit: Some(candle.close * 1.10),
                });
            }
            // Death cross = sell
            else if s < l && position.is_some() {
                if let Some(pos) = position.take() {
                    let pnl = (candle.close - pos.entry_price) / pos.entry_price * pos.size;
                    equity += pnl;
                    results.total_pnl += pnl;
                    results.total_trades += 1;
                    
                    if pnl > 0.0 {
                        results.winning_trades += 1;
                    } else {
                        results.losing_trades += 1;
                    }
                    
                    peak_equity = peak_equity.max(equity);
                    let drawdown = peak_equity - equity;
                    results.max_drawdown = results.max_drawdown.max(drawdown);
                }
            }
        }
    }
    
    results
}

/// Run static mean-reversion backtest
fn run_static_mean_reversion_backtest(data: &[Candle]) -> BacktestResults {
    use kraken_regime::strategy::mean_reversion::{MeanReversionStrategy, Signal};
    
    let mut results = BacktestResults::default();
    let mut strategy = MeanReversionStrategy::default_config();
    let mut position: Option<Position> = None;
    let mut equity = 10000.0;
    let mut peak_equity = equity;
    
    for candle in data {
        let signal = strategy.update(candle.high, candle.low, candle.close);
        
        match signal {
            Signal::Buy if position.is_none() => {
                position = Some(Position {
                    entry_price: candle.close,
                    size: equity * 0.01,
                    stop_loss: strategy.stop_loss(),
                    take_profit: strategy.take_profit(),
                });
            }
            Signal::Sell if position.is_some() => {
                if let Some(pos) = position.take() {
                    let pnl = (candle.close - pos.entry_price) / pos.entry_price * pos.size;
                    equity += pnl;
                    results.total_pnl += pnl;
                    results.total_trades += 1;
                    
                    if pnl > 0.0 {
                        results.winning_trades += 1;
                    } else {
                        results.losing_trades += 1;
                    }
                    
                    peak_equity = peak_equity.max(equity);
                    let drawdown = peak_equity - equity;
                    results.max_drawdown = results.max_drawdown.max(drawdown);
                }
            }
            _ => {}
        }
    }
    
    results
}

#[derive(Debug)]
struct Position {
    entry_price: f64,
    size: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

// ============================================================================
// Data Generation Functions
// ============================================================================

fn generate_trending_market(bars: usize, start_price: f64, trend_per_bar: f64) -> Vec<Candle> {
    let mut candles = Vec::new();
    let mut price = start_price;
    
    for i in 0..bars {
        let noise = (rand::random::<f64>() - 0.5) * trend_per_bar * 0.5;
        price += trend_per_bar + noise;
        
        let volatility = price * 0.01;
        let high = price + rand::random::<f64>() * volatility;
        let low = price - rand::random::<f64>() * volatility;
        
        candles.push(Candle {
            timestamp: i as i64 * 900,  // 15-min bars
            open: price - trend_per_bar / 2.0,
            high,
            low,
            close: price,
            volume: 100.0 + rand::random::<f64>() * 50.0,
        });
    }
    
    candles
}

fn generate_ranging_market(bars: usize, center_price: f64, range_pct: f64) -> Vec<Candle> {
    let mut candles = Vec::new();
    
    for i in 0..bars {
        let cycle = (i as f64 * 0.05).sin() * center_price * range_pct / 100.0;
        let noise = (rand::random::<f64>() - 0.5) * center_price * 0.002;
        let price = center_price + cycle + noise;
        
        let volatility = center_price * 0.005;
        let high = price + rand::random::<f64>() * volatility;
        let low = price - rand::random::<f64>() * volatility;
        
        candles.push(Candle {
            timestamp: i as i64 * 900,
            open: price - noise / 2.0,
            high,
            low,
            close: price,
            volume: 100.0 + rand::random::<f64>() * 50.0,
        });
    }
    
    candles
}

fn generate_volatile_market(bars: usize, center_price: f64, volatility_pct: f64) -> Vec<Candle> {
    let mut candles = Vec::new();
    let mut price = center_price;
    
    for i in 0..bars {
        let change = (rand::random::<f64>() - 0.5) * center_price * volatility_pct / 100.0;
        price += change;
        price = price.max(center_price * 0.9).min(center_price * 1.1);  // Keep bounded
        
        let volatility = center_price * volatility_pct / 100.0 * 0.5;
        let high = price + rand::random::<f64>() * volatility;
        let low = price - rand::random::<f64>() * volatility;
        
        candles.push(Candle {
            timestamp: i as i64 * 900,
            open: price - change / 2.0,
            high,
            low,
            close: price,
            volume: 100.0 + rand::random::<f64>() * 100.0,  // Higher volume in volatile markets
        });
    }
    
    candles
}

fn generate_mixed_market(bars: usize, start_price: f64) -> Vec<Candle> {
    let mut candles = Vec::new();
    let segment_size = bars / 4;
    
    // Trending up
    candles.extend(generate_trending_market(segment_size, start_price, 50.0));
    
    // Ranging
    let last_price = candles.last().map(|c| c.close).unwrap_or(start_price);
    candles.extend(generate_ranging_market(segment_size, last_price, 3.0));
    
    // Volatile
    candles.extend(generate_volatile_market(segment_size, last_price, 4.0));
    
    // Trending down
    candles.extend(generate_trending_market(segment_size, last_price, -30.0));
    
    candles
}
