//! WebSocket stream clients: real-time market data and trading updates.

use std::time::Duration;

pub mod data;
pub mod trading;

pub use data::{DataMessage, MarketDataStream, Subscription};
pub use trading::{StreamEvent, TradeEvent, TradeUpdate, TradingStream};

/// Options for opt-in automatic reconnection of the stream clients.
///
/// When enabled via [`data::MarketDataStream::set_auto_reconnect`] or
/// [`trading::TradingStream::set_auto_reconnect`], the client reconnects on an
/// unexpected close or transport error: it re-connects, re-authenticates and
/// restores the previous subscription set before yielding further messages.
/// Attempts are separated by an exponential backoff that starts at
/// [`ReconnectOptions::initial_backoff`] and doubles up to
/// [`ReconnectOptions::max_backoff`].
#[derive(Debug, Clone, Copy)]
pub struct ReconnectOptions {
    /// Backoff before the first reconnect attempt; doubles after each failure.
    pub initial_backoff: Duration,
    /// Upper bound on the backoff between attempts.
    pub max_backoff: Duration,
    /// Maximum number of consecutive reconnect attempts before the last error
    /// is returned to the caller.
    pub max_attempts: u32,
}

impl Default for ReconnectOptions {
    fn default() -> Self {
        Self {
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
            max_attempts: 10,
        }
    }
}
