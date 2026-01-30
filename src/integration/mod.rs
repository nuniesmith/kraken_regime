//! Kraken Exchange Integration Module
//!
//! Provides integration with Kraken's WebSocket and REST APIs for live trading.

mod kraken;

pub use kraken::{
    rest_integration, websocket_integration, Candle, KrakenIntegrationConfig, KrakenRegimeTrader,
    PairStatus, TradeAction, TradeType,
};
