//! Kraken Exchange Integration Module
//!
//! Provides integration with Kraken's WebSocket and REST APIs for live trading.

mod kraken;

pub use kraken::{
    KrakenRegimeTrader,
    KrakenIntegrationConfig,
    Candle,
    TradeAction,
    TradeType,
    PairStatus,
    websocket_integration,
    rest_integration,
};
