//! Live Trading Example
//!
//! Demonstrates how to use the regime-aware trading system with Kraken's
//! WebSocket for live market data and automatic strategy switching.
//!
//! Run with: cargo run --example live_trading

use futures_util::{SinkExt, StreamExt};
use kraken_regime::prelude::*;
use kraken_regime::TradeType;
use serde_json::{json, Value};
use std::env;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

const KRAKEN_WS_URL: &str = "wss://ws.kraken.com";
#[allow(dead_code)]
const KRAKEN_WS_URL_BETA: &str = "wss://beta-ws.kraken.com";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    dotenv::dotenv().ok();

    info!("🦑 Kraken Regime-Aware Trading Bot Starting...");

    // Configuration
    let config = KrakenIntegrationConfig {
        pairs: vec![
            "BTC/USD".to_string(),
            "ETH/USD".to_string(),
            "SOL/USD".to_string(),
        ],
        timeframe_minutes: 15,
        live_trading: false, // Signal-only mode for safety
        min_trade_usd: 10.0,
        max_trade_usd: 250.0,
        risk_per_trade_pct: 1.0,
        ..Default::default()
    };

    // Create trader instance
    let mut trader = KrakenRegimeTrader::new(config.clone());

    // Create channel for trade signals
    let (signal_tx, mut signal_rx) = mpsc::channel::<TradeAction>(100);
    trader.set_signal_channel(signal_tx);

    // Warmup with historical data
    info!("📊 Fetching historical data for warmup...");
    for pair in &config.pairs {
        match warmup_pair(&mut trader, pair).await {
            Ok(_) => info!("✅ {} warmup complete", pair),
            Err(e) => warn!("⚠️ {} warmup failed: {}", pair, e),
        }
    }

    // Print initial status
    print_status(&trader);

    // Spawn signal handler
    tokio::spawn(async move {
        while let Some(action) = signal_rx.recv().await {
            handle_trade_signal(action).await;
        }
    });

    // Connect to WebSocket and start processing
    info!("🔌 Connecting to Kraken WebSocket...");
    run_websocket_loop(&config, trader).await?;

    Ok(())
}

/// Warmup a trading pair with historical OHLC data
async fn warmup_pair(
    trader: &mut KrakenRegimeTrader,
    pair: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert pair format for Kraken API
    let kraken_pair = pair.replace("/", "").replace("BTC", "XBT");

    // Fetch last 720 candles (15 min * 720 = 7.5 days)
    let url = format!(
        "https://api.kraken.com/0/public/OHLC?pair={}&interval=15",
        kraken_pair
    );

    let response: Value = reqwest::get(&url).await?.json().await?;

    if let Some(error) = response.get("error").and_then(|e| e.as_array()) {
        if !error.is_empty() {
            return Err(format!("Kraken API error: {:?}", error).into());
        }
    }

    // Parse OHLC data
    let mut candles = Vec::new();
    if let Some(result) = response.get("result") {
        // Find the data array (key varies by pair)
        for (key, data) in result.as_object().unwrap_or(&serde_json::Map::new()) {
            if key == "last" {
                continue;
            }

            if let Some(arr) = data.as_array() {
                for item in arr {
                    if let Some(ohlc) = item.as_array() {
                        if ohlc.len() >= 6 {
                            let candle = Candle {
                                timestamp: ohlc[0].as_i64().unwrap_or(0),
                                open: parse_price(&ohlc[1]),
                                high: parse_price(&ohlc[2]),
                                low: parse_price(&ohlc[3]),
                                close: parse_price(&ohlc[4]),
                                volume: parse_price(&ohlc[6]),
                            };
                            candles.push(candle);
                        }
                    }
                }
            }
        }
    }

    info!("  Loaded {} candles for {}", candles.len(), pair);
    trader.warmup_with_history(pair, &candles);

    Ok(())
}

fn parse_price(value: &Value) -> f64 {
    value
        .as_str()
        .and_then(|s| s.parse().ok())
        .or_else(|| value.as_f64())
        .unwrap_or(0.0)
}

/// Run the main WebSocket loop
async fn run_websocket_loop(
    config: &KrakenIntegrationConfig,
    mut trader: KrakenRegimeTrader,
) -> Result<(), Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async(KRAKEN_WS_URL).await?;
    let (mut write, mut read) = ws_stream.split();

    info!("✅ Connected to Kraken WebSocket");

    // Subscribe to OHLC channels for each pair
    for pair in &config.pairs {
        let sub_msg = json!({
            "event": "subscribe",
            "pair": [pair],
            "subscription": {
                "name": "ohlc",
                "interval": config.timeframe_minutes
            }
        });

        write.send(Message::Text(sub_msg.to_string())).await?;
        info!("📡 Subscribed to {} OHLC", pair);
    }

    // Also subscribe to ticker for real-time price updates
    let ticker_sub = json!({
        "event": "subscribe",
        "pair": config.pairs,
        "subscription": {
            "name": "ticker"
        }
    });
    write.send(Message::Text(ticker_sub.to_string())).await?;

    // Process messages
    let mut status_interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

    loop {
        tokio::select! {
            Some(msg) = read.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Err(e) = process_ws_message(&text, &mut trader) {
                            warn!("Error processing message: {}", e);
                        }
                    }
                    Ok(Message::Ping(data)) => {
                        let _ = write.send(Message::Pong(data)).await;
                    }
                    Ok(Message::Close(_)) => {
                        warn!("WebSocket closed by server");
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            _ = status_interval.tick() => {
                print_status(&trader);
            }
        }
    }

    Ok(())
}

/// Process a WebSocket message
fn process_ws_message(
    text: &str,
    trader: &mut KrakenRegimeTrader,
) -> Result<(), Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_str(text)?;

    // Skip heartbeats and status messages
    if value.get("event").is_some() {
        return Ok(());
    }

    // OHLC format: [channelId, [time, etime, open, high, low, close, vwap, volume, count], "ohlc-15", "XBT/USD"]
    if let Some(arr) = value.as_array() {
        if arr.len() >= 4 {
            let channel = arr.get(2).and_then(|v| v.as_str()).unwrap_or("");

            if channel.starts_with("ohlc") {
                if let (Some(data), Some(pair)) = (
                    arr.get(1).and_then(|v| v.as_array()),
                    arr.get(3).and_then(|v| v.as_str()),
                ) {
                    if data.len() >= 8 {
                        let candle = Candle {
                            timestamp: parse_price(&data[0]) as i64,
                            open: parse_price(&data[2]),
                            high: parse_price(&data[3]),
                            low: parse_price(&data[4]),
                            close: parse_price(&data[5]),
                            volume: parse_price(&data[7]),
                        };

                        // Convert Kraken pair format
                        let normalized_pair = normalize_pair(pair);

                        if let Some(action) = trader.process_candle(&normalized_pair, &candle) {
                            if action.action != TradeType::Hold {
                                info!(
                                    "📊 {} | Regime: {} | Strategy: {} | Signal: {:?}",
                                    normalized_pair,
                                    action.regime,
                                    action.source_strategy,
                                    action.action
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Normalize Kraken pair format to standard format
fn normalize_pair(kraken_pair: &str) -> String {
    kraken_pair.replace("XBT", "BTC").replace("/", "/")
}

/// Handle a trade signal
async fn handle_trade_signal(action: TradeAction) {
    match action.action {
        TradeType::Buy => {
            info!("🟢 BUY SIGNAL");
            info!("   Pair: {}", action.symbol);
            info!("   Price: ${:.2}", action.price);
            info!("   Size Factor: {:.1}%", action.size_factor * 100.0);
            info!(
                "   Stop Loss: {:?}",
                action.stop_loss.map(|p| format!("${:.2}", p))
            );
            info!(
                "   Take Profit: {:?}",
                action.take_profit.map(|p| format!("${:.2}", p))
            );
            info!("   Strategy: {}", action.source_strategy);
            info!(
                "   Regime: {} (confidence: {:.1}%)",
                action.regime,
                action.confidence * 100.0
            );
            info!("   Reason: {}", action.reason);

            // Here you would integrate with your existing order execution
            // In DRY_RUN mode, just send Discord notification
            send_discord_notification(&action).await;
        }
        TradeType::Sell => {
            info!("🔴 SELL SIGNAL");
            info!("   Pair: {}", action.symbol);
            info!("   Price: ${:.2}", action.price);
            info!("   Strategy: {}", action.source_strategy);
            info!("   Reason: {}", action.reason);

            send_discord_notification(&action).await;
        }
        TradeType::Hold => {
            // No action needed
        }
    }
}

/// Send notification to Discord
async fn send_discord_notification(action: &TradeAction) {
    if let Ok(webhook_url) = env::var("DISCORD_WEBHOOK_URL") {
        let emoji = match action.action {
            TradeType::Buy => "🟢",
            TradeType::Sell => "🔴",
            TradeType::Hold => "⚪",
        };

        let payload = json!({
            "embeds": [{
                "title": format!("{} {} Signal - {}", emoji,
                    match action.action {
                        TradeType::Buy => "BUY",
                        TradeType::Sell => "SELL",
                        TradeType::Hold => "HOLD",
                    },
                    action.symbol
                ),
                "color": match action.action {
                    TradeType::Buy => 0x00ff00,
                    TradeType::Sell => 0xff0000,
                    TradeType::Hold => 0xffffff,
                },
                "fields": [
                    {"name": "Price", "value": format!("${:.2}", action.price), "inline": true},
                    {"name": "Regime", "value": &action.regime, "inline": true},
                    {"name": "Strategy", "value": &action.source_strategy, "inline": true},
                    {"name": "Confidence", "value": format!("{:.1}%", action.confidence * 100.0), "inline": true},
                    {"name": "Position Size", "value": format!("{:.0}%", action.size_factor * 100.0), "inline": true},
                    {"name": "Reason", "value": &action.reason, "inline": false},
                ],
                "footer": {"text": "Kraken Regime-Aware Trading Bot"}
            }]
        });

        if let Err(e) = reqwest::Client::new()
            .post(&webhook_url)
            .json(&payload)
            .send()
            .await
        {
            warn!("Failed to send Discord notification: {}", e);
        }
    }
}

/// Print current status of all pairs
fn print_status(trader: &KrakenRegimeTrader) {
    info!("📈 Current Status:");
    info!("─────────────────────────────────────────────────────");

    for (pair, status) in trader.status_summary() {
        let ready_icon = if status.ready { "✅" } else { "⏳" };
        let regime = status
            .regime
            .map(|r| r.to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let strategy = status
            .strategy
            .map(|s| s.to_string())
            .unwrap_or_else(|| "None".to_string());

        info!(
            "{} {} | Regime: {:20} | Strategy: {:15} | Changes: {}",
            ready_icon, pair, regime, strategy, status.regime_changes
        );
    }
    info!("─────────────────────────────────────────────────────");
}
