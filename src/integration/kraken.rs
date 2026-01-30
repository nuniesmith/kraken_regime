//! Kraken Integration for Regime-Aware Trading
//!
//! Integrates the strategy router with Kraken's WebSocket (live data) and REST API (historical data).
//! This module bridges your existing kraken codebase with the new regime detection system.

use crate::regime::MarketRegime;
use crate::strategy::mean_reversion::Signal;
use crate::strategy::router::{ActiveStrategy, RoutedSignal, StrategyRouter, StrategyRouterConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

/// OHLCV candle data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Trade action to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAction {
    pub symbol: String,
    pub action: TradeType,
    pub price: f64,
    pub size_factor: f64, // 0.0 - 1.0, multiply by max position
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub source_strategy: String,
    pub regime: String,
    pub confidence: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TradeType {
    Buy,
    Sell,
    Hold,
}

impl From<Signal> for TradeType {
    fn from(signal: Signal) -> Self {
        match signal {
            Signal::Buy => TradeType::Buy,
            Signal::Sell => TradeType::Sell,
            Signal::Hold => TradeType::Hold,
        }
    }
}

/// Configuration for Kraken integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KrakenIntegrationConfig {
    /// Trading pairs to monitor
    pub pairs: Vec<String>,

    /// Timeframe for candles (in minutes)
    pub timeframe_minutes: u32,

    /// Strategy router configuration
    pub router_config: StrategyRouterConfig,

    /// Whether to execute trades or just signal
    pub live_trading: bool,

    /// Minimum USD value per trade
    pub min_trade_usd: f64,

    /// Maximum USD value per trade
    pub max_trade_usd: f64,

    /// Risk per trade as percentage of account
    pub risk_per_trade_pct: f64,
}

impl Default for KrakenIntegrationConfig {
    fn default() -> Self {
        Self {
            pairs: vec![
                "BTC/USD".to_string(),
                "ETH/USD".to_string(),
                "SOL/USD".to_string(),
            ],
            timeframe_minutes: 15, // 15-minute candles
            router_config: StrategyRouterConfig::default(),
            live_trading: false, // Start in signal-only mode
            min_trade_usd: 10.0,
            max_trade_usd: 250.0,
            risk_per_trade_pct: 1.0,
        }
    }
}

/// Main Kraken trading integration
pub struct KrakenRegimeTrader {
    config: KrakenIntegrationConfig,
    router: StrategyRouter,

    // Candle aggregation (for building candles from ticks)
    candle_builders: HashMap<String, CandleBuilder>,

    // Last processed candle timestamp per pair
    last_candle_time: HashMap<String, i64>,

    // Signal channel
    signal_tx: Option<mpsc::Sender<TradeAction>>,
}

impl KrakenRegimeTrader {
    pub fn new(config: KrakenIntegrationConfig) -> Self {
        let mut router = StrategyRouter::new(config.router_config.clone());

        // Register all pairs
        for pair in &config.pairs {
            router.register_asset(pair);
        }

        Self {
            config,
            router,
            candle_builders: HashMap::new(),
            last_candle_time: HashMap::new(),
            signal_tx: None,
        }
    }

    /// Set signal channel for async notification
    pub fn set_signal_channel(&mut self, tx: mpsc::Sender<TradeAction>) {
        self.signal_tx = Some(tx);
    }

    /// Process a completed candle
    pub fn process_candle(&mut self, pair: &str, candle: &Candle) -> Option<TradeAction> {
        // Check if this is a new candle (avoid reprocessing)
        if let Some(&last_time) = self.last_candle_time.get(pair) {
            if candle.timestamp <= last_time {
                return None; // Already processed
            }
        }
        self.last_candle_time
            .insert(pair.to_string(), candle.timestamp);

        // Update router and get signal
        let routed = self
            .router
            .update(pair, candle.high, candle.low, candle.close)?;

        // Convert to trade action
        let action = self.routed_to_action(pair, candle.close, &routed);

        // Send to channel if set and not Hold
        if action.action != TradeType::Hold {
            if let Some(tx) = &self.signal_tx {
                let tx_clone = tx.clone();
                let action_clone = action.clone();
                tokio::spawn(async move {
                    let _ = tx_clone.send(action_clone).await;
                });
            }
        }

        Some(action)
    }

    /// Process real-time tick data (aggregates into candles)
    pub fn process_tick(&mut self, pair: &str, price: f64, timestamp: i64) -> Option<TradeAction> {
        let timeframe_secs = self.config.timeframe_minutes as i64 * 60;

        // Get or create candle builder
        let builder = self
            .candle_builders
            .entry(pair.to_string())
            .or_insert_with(|| CandleBuilder::new(timeframe_secs));

        // Add tick to builder
        if let Some(completed_candle) = builder.add_tick(price, timestamp) {
            return self.process_candle(pair, &completed_candle);
        }

        None
    }

    fn routed_to_action(&self, pair: &str, price: f64, routed: &RoutedSignal) -> TradeAction {
        TradeAction {
            symbol: pair.to_string(),
            action: routed.signal.into(),
            price,
            size_factor: routed.position_size_factor,
            stop_loss: routed.stop_loss,
            take_profit: routed.take_profit,
            source_strategy: routed.source_strategy.to_string(),
            regime: routed.regime.to_string(),
            confidence: routed.confidence,
            reason: routed.reason.clone(),
        }
    }

    /// Initialize with historical data from REST API
    pub fn warmup_with_history(&mut self, pair: &str, candles: &[Candle]) {
        println!(
            "[{}] Warming up with {} historical candles",
            pair,
            candles.len()
        );

        for candle in candles {
            self.router
                .update(pair, candle.high, candle.low, candle.close);
        }

        if self.router.is_ready(pair) {
            println!(
                "[{}] Warmup complete. Current regime: {:?}",
                pair,
                self.router.get_regime(pair)
            );
        } else {
            println!("[{}] Still warming up, need more data", pair);
        }
    }

    /// Get current regime for a pair
    pub fn get_regime(&self, pair: &str) -> Option<MarketRegime> {
        self.router.get_regime(pair)
    }

    /// Get active strategy for a pair
    pub fn get_active_strategy(&self, pair: &str) -> Option<ActiveStrategy> {
        self.router.get_active_strategy(pair)
    }

    /// Is the system ready to trade a pair?
    pub fn is_ready(&self, pair: &str) -> bool {
        self.router.is_ready(pair)
    }

    /// Get status summary for all pairs
    pub fn status_summary(&self) -> HashMap<String, PairStatus> {
        let mut status = HashMap::new();

        for pair in &self.config.pairs {
            status.insert(
                pair.clone(),
                PairStatus {
                    ready: self.is_ready(pair),
                    regime: self.get_regime(pair),
                    strategy: self.get_active_strategy(pair),
                    regime_changes: self.router.regime_changes(pair).unwrap_or(0),
                },
            );
        }

        status
    }
}

/// Status for a trading pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairStatus {
    pub ready: bool,
    pub regime: Option<MarketRegime>,
    pub strategy: Option<ActiveStrategy>,
    pub regime_changes: u32,
}

/// Builds candles from tick data
#[derive(Debug)]
struct CandleBuilder {
    timeframe_secs: i64,
    current_candle: Option<PartialCandle>,
}

#[derive(Debug)]
struct PartialCandle {
    start_time: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl CandleBuilder {
    fn new(timeframe_secs: i64) -> Self {
        Self {
            timeframe_secs,
            current_candle: None,
        }
    }

    fn add_tick(&mut self, price: f64, timestamp: i64) -> Option<Candle> {
        let candle_start = (timestamp / self.timeframe_secs) * self.timeframe_secs;

        match &mut self.current_candle {
            Some(candle) if candle.start_time == candle_start => {
                // Update existing candle
                candle.high = candle.high.max(price);
                candle.low = candle.low.min(price);
                candle.close = price;
                candle.volume += 1.0; // Simplified - real impl would use actual volume
                None
            }
            Some(candle) => {
                // New candle period - complete the old one
                let completed = Candle {
                    timestamp: candle.start_time,
                    open: candle.open,
                    high: candle.high,
                    low: candle.low,
                    close: candle.close,
                    volume: candle.volume,
                };

                // Start new candle
                self.current_candle = Some(PartialCandle {
                    start_time: candle_start,
                    open: price,
                    high: price,
                    low: price,
                    close: price,
                    volume: 1.0,
                });

                Some(completed)
            }
            None => {
                // First tick
                self.current_candle = Some(PartialCandle {
                    start_time: candle_start,
                    open: price,
                    high: price,
                    low: price,
                    close: price,
                    volume: 1.0,
                });
                None
            }
        }
    }
}

// ============================================================================
// Example integration with your existing Kraken WebSocket
// ============================================================================

/// Example WebSocket message handler
/// Integrate this with your existing src/websocket.rs
pub mod websocket_integration {
    use super::*;

    /// Message types from Kraken WebSocket
    #[derive(Debug, Clone, Deserialize)]
    #[serde(untagged)]
    pub enum KrakenWsMessage {
        Trade(TradeMessage),
        Ohlc(OhlcMessage),
        // Add other message types as needed
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct TradeMessage {
        pub pair: String,
        pub price: String,
        pub timestamp: String,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct OhlcMessage {
        pub pair: String,
        pub time: i64,
        pub open: String,
        pub high: String,
        pub low: String,
        pub close: String,
        pub volume: String,
    }

    impl OhlcMessage {
        pub fn to_candle(&self) -> Option<Candle> {
            Some(Candle {
                timestamp: self.time,
                open: self.open.parse().ok()?,
                high: self.high.parse().ok()?,
                low: self.low.parse().ok()?,
                close: self.close.parse().ok()?,
                volume: self.volume.parse().ok()?,
            })
        }
    }

    /// Example handler - integrate with your WebSocket loop
    pub async fn handle_ws_message(
        trader: &mut KrakenRegimeTrader,
        msg: KrakenWsMessage,
    ) -> Option<TradeAction> {
        match msg {
            KrakenWsMessage::Trade(trade) => {
                let price: f64 = trade.price.parse().ok()?;
                let timestamp: i64 = trade.timestamp.parse::<f64>().ok()? as i64;
                trader.process_tick(&trade.pair, price, timestamp)
            }
            KrakenWsMessage::Ohlc(ohlc) => {
                let candle = ohlc.to_candle()?;
                trader.process_candle(&ohlc.pair, &candle)
            }
        }
    }
}

// ============================================================================
// Example integration with your existing Kraken REST API
// ============================================================================

/// Example REST API integration
/// Integrate this with your existing src/api.rs
pub mod rest_integration {
    use super::*;

    /// Kraken OHLC response format
    #[derive(Debug, Clone, Deserialize)]
    pub struct KrakenOhlcResponse {
        pub error: Vec<String>,
        pub result: HashMap<String, serde_json::Value>,
    }

    /// Parse Kraken OHLC response into candles
    pub fn parse_ohlc_response(response: &KrakenOhlcResponse, pair: &str) -> Vec<Candle> {
        let mut candles = Vec::new();

        // Kraken returns array format: [time, open, high, low, close, vwap, volume, count]
        if let Some(data) = response.result.get(pair) {
            if let Some(arr) = data.as_array() {
                for item in arr {
                    if let Some(ohlc) = item.as_array() {
                        if ohlc.len() >= 6 {
                            let candle = Candle {
                                timestamp: ohlc[0].as_i64().unwrap_or(0),
                                open: ohlc[1].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0),
                                high: ohlc[2].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0),
                                low: ohlc[3].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0),
                                close: ohlc[4].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0),
                                volume: ohlc[6]
                                    .as_str()
                                    .and_then(|s| s.parse().ok())
                                    .unwrap_or(0.0),
                            };
                            candles.push(candle);
                        }
                    }
                }
            }
        }

        candles
    }

    /// Example: Fetch historical OHLC for warmup
    /// Integrate with your existing API client
    pub async fn fetch_historical_ohlc(
        _api_key: &str,
        _api_secret: &str,
        pair: &str,
        interval: u32, // minutes
        since: Option<i64>,
    ) -> Result<Vec<Candle>, Box<dyn std::error::Error + Send + Sync>> {
        // This would use your existing API client
        // Example URL: https://api.kraken.com/0/public/OHLC

        let url = format!(
            "https://api.kraken.com/0/public/OHLC?pair={}&interval={}{}",
            pair,
            interval,
            since.map(|s| format!("&since={}", s)).unwrap_or_default()
        );

        let response: KrakenOhlcResponse = reqwest::get(&url).await?.json().await?;

        if !response.error.is_empty() {
            return Err(format!("Kraken API error: {:?}", response.error).into());
        }

        // Convert pair format (e.g., "BTCUSD" vs "XBTUSD")
        let kraken_pair = convert_pair_format(pair);
        Ok(parse_ohlc_response(&response, &kraken_pair))
    }

    fn convert_pair_format(pair: &str) -> String {
        // Handle Kraken's naming convention
        pair.replace("/", "").replace("BTC", "XBT") // Kraken uses XBT for Bitcoin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candle_builder() {
        let mut builder = CandleBuilder::new(60); // 1 minute candles

        // First tick
        assert!(builder.add_tick(100.0, 0).is_none());

        // More ticks in same period
        assert!(builder.add_tick(101.0, 30).is_none());
        assert!(builder.add_tick(99.0, 45).is_none());

        // New period - should complete candle
        let candle = builder.add_tick(102.0, 60);
        assert!(candle.is_some());

        let c = candle.unwrap();
        assert_eq!(c.open, 100.0);
        assert_eq!(c.high, 101.0);
        assert_eq!(c.low, 99.0);
        assert_eq!(c.close, 99.0);
    }

    #[tokio::test]
    async fn test_trader_initialization() {
        let config = KrakenIntegrationConfig::default();
        let trader = KrakenRegimeTrader::new(config);

        assert!(!trader.is_ready("BTC/USD")); // Not warmed up yet
    }
}
