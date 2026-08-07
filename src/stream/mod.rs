//! Streaming clients: real-time market data and trading updates over
//! WebSocket, plus account activity and corporate action events over
//! Server-Sent Events.

use std::time::Duration;

pub mod activity_events;
pub mod ca_events;
pub mod ca_payloads;
pub mod data;
pub mod events;
pub mod sse;
pub mod trading;

pub use activity_events::{
    ActivityEvent, ActivityEventDetails, ActivityEventsClient, ActivityEventsRequest,
    NonTradeActivityEventDetails, TradeActivityEventDetails, TradeExecutionType,
};
pub use ca_events::{
    CorporateActionEvent, CorporateActionEventRegion, CorporateActionEventType,
    CorporateActionEventsClient, CorporateActionEventsRequest, CorporateActionMutation,
    CorporateActionPayload, CorporateActionRegionFilter,
};
pub use ca_payloads::*;
pub use data::{DataMessage, MarketDataStream, Subscription};
pub use events::EventStream;
pub use sse::{SseFrame, SseParser};
pub use trading::{StreamEvent, TradeEvent, TradeUpdate, TradingStream};

/// Options for opt-in automatic reconnection of the stream clients.
///
/// When enabled via [`data::MarketDataStream::set_auto_reconnect`],
/// [`trading::TradingStream::set_auto_reconnect`] or
/// [`events::EventStream::set_auto_reconnect`], the client reconnects on an
/// unexpected close or transport error: it re-connects, re-authenticates and
/// restores the previous subscription set (or, for event streams, resumes from
/// the last event id) before yielding further messages. Attempts are separated
/// by an exponential backoff that starts at
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
