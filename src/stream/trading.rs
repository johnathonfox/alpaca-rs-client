//! Trading updates WebSocket stream (`/stream`).
//!
//! This stream uses the legacy envelope shape (`{"stream": .., "data": ..}`),
//! unlike the market data streams which use JSON arrays. Protocol: connect,
//! then immediately send `{"action":"auth","key":..,"secret":..}` — the
//! server waits for the client and sends no greeting first. Expect
//! `{"stream":"authorization","data":{"status":"authorized"}}`, then
//! `{"action":"listen","data":{"streams":["trade_updates"]}}`.
//!
//! Frame shape differs by environment (observed live): paper sends JSON as
//! **binary** frames, live as text frames; both are accepted.
//!
//! Auto-reconnect is opt-in: by default callers should reconnect when
//! [`TradingStream::next`] returns an error or `Ok(None)`. With
//! [`TradingStream::set_auto_reconnect`] the client reconnects and restores
//! the `trade_updates` listener itself.

use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{Message, http};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::error::{Error, Result};
use crate::rest::Credentials;
use crate::stream::ReconnectOptions;
use crate::trading::Order;

const PAPER_STREAM: &str = "wss://paper-api.alpaca.markets/stream";
const LIVE_STREAM: &str = "wss://api.alpaca.markets/stream";

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// The lifecycle event of a [`TradeUpdate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeEvent {
    /// New order.
    New,
    /// Order filled.
    Fill,
    /// Order partially filled.
    PartialFill,
    /// Order canceled.
    Canceled,
    /// Order expired.
    Expired,
    /// Order done for the day.
    DoneForDay,
    /// Order replaced.
    Replaced,
    /// Order accepted.
    Accepted,
    /// Order rejected.
    Rejected,
    /// Order pending new.
    PendingNew,
    /// Order stopped.
    Stopped,
    /// Order pending cancel.
    PendingCancel,
    /// Order pending replace.
    PendingReplace,
    /// Order calculated.
    Calculated,
    /// Order suspended.
    Suspended,
    /// Order replace rejected.
    OrderReplaceRejected,
    /// Order cancel rejected.
    OrderCancelRejected,
}

/// A `trade_updates` message payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeUpdate {
    /// The lifecycle event.
    pub event: TradeEvent,
    /// Execution id (fills only).
    pub execution_id: Option<String>,
    /// The full order object.
    pub order: Order,
    /// Event timestamp.
    pub timestamp: DateTime<Utc>,
    /// Fill price.
    pub price: Option<Decimal>,
    /// Fill quantity.
    pub qty: Option<Decimal>,
    /// Resulting position quantity.
    pub position_qty: Option<Decimal>,
}

/// A message from the trading stream.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// A trade update. Boxed to keep the enum small.
    TradeUpdate(Box<TradeUpdate>),
    /// Any other message (control messages, unmodeled streams), kept as raw
    /// JSON.
    Other(serde_json::Value),
}

async fn send_json(ws: &mut Ws, value: &impl Serialize) -> Result<()> {
    let text = serde_json::to_string(value)?;
    ws.send(Message::Text(text.into())).await?;
    Ok(())
}

/// Reads the next text or binary message as raw JSON. Paper sends JSON as
/// binary frames (observed live), so both shapes are decoded.
async fn read_json(ws: &mut Ws) -> Result<serde_json::Value> {
    loop {
        match ws.next().await {
            None => return Err(Error::StreamClosed),
            Some(Err(e)) => return Err(Error::WebSocket(e)),
            Some(Ok(Message::Text(text))) => return Ok(serde_json::from_str(text.as_str())?),
            Some(Ok(Message::Binary(bytes))) => return Ok(serde_json::from_slice(&bytes)?),
            Some(Ok(Message::Close(_))) => return Err(Error::StreamClosed),
            // Ping/Pong handled by tungstenite; skip other frame types.
            Some(Ok(_)) => continue,
        }
    }
}

/// Trading updates WebSocket stream client.
pub struct TradingStream {
    ws: Ws,
    url: String,
    creds: Credentials,
    listening: bool,
    reconnect: Option<ReconnectOptions>,
}

impl TradingStream {
    /// Connects and authenticates to an arbitrary trading stream URL.
    ///
    /// This is the URL-override entry point (parity with alpaca-py's
    /// `url_override`): pass a `ws(s)://` URL to target a mock server or a
    /// non-default endpoint. Prefer [`TradingStream::paper`] or
    /// [`TradingStream::live`] for the standard Alpaca environments.
    pub async fn connect(url: &str, creds: &Credentials) -> Result<Self> {
        let ws = Self::handshake(url, creds).await?;
        Ok(Self {
            ws,
            url: url.to_string(),
            creds: creds.clone(),
            listening: false,
            reconnect: None,
        })
    }

    /// Connects to the paper trading stream.
    pub async fn paper(creds: &Credentials) -> Result<Self> {
        Self::connect(PAPER_STREAM, creds).await
    }

    /// Connects to the live trading stream.
    pub async fn live(creds: &Credentials) -> Result<Self> {
        Self::connect(LIVE_STREAM, creds).await
    }

    /// Performs the connect/auth handshake against `url`.
    async fn handshake(url: &str, creds: &Credentials) -> Result<Ws> {
        let mut request = url.into_client_request()?;
        request.headers_mut().insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/json"),
        );
        let (mut ws, _response) = connect_async(request).await?;

        // The server waits for the client and sends no greeting; auth must
        // be the first message (observed live: waiting for a greeting first
        // deadlocks until the server resets the connection).
        let (key, secret) = creds.key_secret();
        send_json(
            &mut ws,
            &serde_json::json!({"action": "auth", "key": key, "secret": secret}),
        )
        .await?;

        // Read until the authorization reply arrives, tolerating any other
        // control messages that may precede it.
        let reply = loop {
            let message = read_json(&mut ws).await?;
            if message.get("stream").and_then(|s| s.as_str()) == Some("authorization") {
                break message;
            }
        };
        let authorized =
            reply.pointer("/data/status").and_then(|s| s.as_str()) == Some("authorized");
        if !authorized {
            return Err(Error::Stream(format!(
                "unexpected authorization reply: {reply}"
            )));
        }

        tracing::debug!(url, "trading stream connected and authenticated");
        Ok(ws)
    }

    /// Enables automatic reconnection. On an unexpected close or transport
    /// error, [`TradingStream::next`] reconnects (with the backoff described
    /// by [`ReconnectOptions`]), re-authenticates and re-listens to
    /// `trade_updates` if it was active. Disabled by default.
    pub fn set_auto_reconnect(&mut self, options: ReconnectOptions) {
        self.reconnect = Some(options);
    }

    /// Disables automatic reconnection (the default).
    pub fn disable_auto_reconnect(&mut self) {
        self.reconnect = None;
    }

    /// Subscribes to the `trade_updates` stream.
    pub async fn listen_trade_updates(&mut self) -> Result<()> {
        Self::send_listen(&mut self.ws).await?;
        self.listening = true;
        Ok(())
    }

    /// Sends the `listen` action and waits for the `listening` reply.
    async fn send_listen(ws: &mut Ws) -> Result<()> {
        send_json(
            ws,
            &serde_json::json!({
                "action": "listen",
                "data": {"streams": ["trade_updates"]}
            }),
        )
        .await?;

        let reply = read_json(ws).await?;
        let listening = reply.get("stream").and_then(|s| s.as_str()) == Some("listening");
        if !listening {
            return Err(Error::Stream(format!("unexpected listen reply: {reply}")));
        }
        Ok(())
    }

    /// Reconnects, re-authenticates and restores the `trade_updates` listener
    /// (if it was active), with exponential backoff between attempts.
    async fn reconnect(&mut self, options: &ReconnectOptions) -> Result<()> {
        let mut backoff = options.initial_backoff;
        let mut last_error = Error::StreamClosed;
        for attempt in 1..=options.max_attempts {
            match Self::handshake(&self.url, &self.creds).await {
                Ok(ws) => {
                    self.ws = ws;
                    if self.listening {
                        Self::send_listen(&mut self.ws).await?;
                    }
                    tracing::info!(attempt, "trading stream reconnected");
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!(attempt, error = %e, "trading stream reconnect failed");
                    last_error = e;
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(options.max_backoff);
                }
            }
        }
        Err(last_error)
    }

    /// Reads the next stream event. Returns `Ok(None)` when the server
    /// closes the connection cleanly and auto-reconnect is disabled; when
    /// auto-reconnect is enabled, a close or transport error instead triggers
    /// a reconnect (see [`TradingStream::set_auto_reconnect`]).
    pub async fn next(&mut self) -> Result<Option<StreamEvent>> {
        let result = Self::read_event(&mut self.ws).await;
        let Some(options) = self.reconnect else {
            return result;
        };
        match result {
            Ok(Some(event)) => Ok(Some(event)),
            Ok(None) | Err(_) => {
                self.reconnect(&options).await?;
                Self::read_event(&mut self.ws).await
            }
        }
    }

    /// Reads the next text or binary frame and parses it into a stream event.
    async fn read_event(ws: &mut Ws) -> Result<Option<StreamEvent>> {
        loop {
            match ws.next().await {
                None => return Ok(None),
                Some(Err(e)) => return Err(Error::WebSocket(e)),
                Some(Ok(Message::Text(text))) => {
                    return Self::parse_event(serde_json::from_str(text.as_str())?);
                }
                Some(Ok(Message::Binary(bytes))) => {
                    return Self::parse_event(serde_json::from_slice(&bytes)?);
                }
                Some(Ok(Message::Close(_))) => return Ok(None),
                // Ping/Pong handled by tungstenite; skip other frame types.
                Some(Ok(_)) => continue,
            }
        }
    }

    /// Maps a parsed JSON message onto the corresponding stream event.
    fn parse_event(value: serde_json::Value) -> Result<Option<StreamEvent>> {
        if value.get("stream").and_then(|s| s.as_str()) == Some("trade_updates") {
            let data = value
                .get("data")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            let update: TradeUpdate = serde_json::from_value(data)?;
            return Ok(Some(StreamEvent::TradeUpdate(Box::new(update))));
        }
        Ok(Some(StreamEvent::Other(value)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trading::OrderStatus;

    const TRADE_UPDATE_JSON: &str = r#"{
        "stream": "trade_updates",
        "data": {
            "event": "fill",
            "execution_id": "9a1b2c3d-0000-4000-8000-000000000001",
            "order": {
                "id": "61e69015-8549-4bfd-b9c3-01e75843f47d",
                "client_order_id": "eb9e2aaa-f71a-4f51-b5b4-52a6c565dad4",
                "created_at": "2024-07-24T07:56:53.123456Z",
                "updated_at": "2024-07-24T07:57:00.123456Z",
                "submitted_at": "2024-07-24T07:56:53.123456Z",
                "filled_at": "2024-07-24T07:56:55.123456Z",
                "expired_at": null,
                "canceled_at": null,
                "failed_at": null,
                "replaced_at": null,
                "replaced_by": null,
                "replaces": null,
                "asset_id": "b0b6dd9d-8b9b-48a9-ba46-b9d54906e415",
                "symbol": "AAPL",
                "asset_class": "us_equity",
                "notional": null,
                "qty": "1790.86",
                "filled_qty": "1790.86",
                "filled_avg_price": "105.89",
                "order_class": "simple",
                "type": "market",
                "side": "buy",
                "time_in_force": "day",
                "limit_price": null,
                "stop_price": null,
                "status": "filled",
                "extended_hours": false,
                "legs": null,
                "trail_percent": null,
                "trail_price": null,
                "hwm": null
            },
            "timestamp": "2024-07-24T07:56:55.123456Z",
            "price": "105.89",
            "qty": "1790.86",
            "position_qty": "0"
        }
    }"#;

    #[test]
    fn parse_trade_update() {
        let value: serde_json::Value = serde_json::from_str(TRADE_UPDATE_JSON).unwrap();
        assert_eq!(value["stream"], "trade_updates");
        let update: TradeUpdate = serde_json::from_value(value["data"].clone()).unwrap();
        assert_eq!(update.event, TradeEvent::Fill);
        assert_eq!(
            update.execution_id.as_deref(),
            Some("9a1b2c3d-0000-4000-8000-000000000001")
        );
        assert_eq!(update.order.symbol, "AAPL");
        assert_eq!(update.order.status, OrderStatus::Filled);
        assert_eq!(update.price, Some(Decimal::new(10589, 2)));
        assert_eq!(update.qty, Some(Decimal::new(179086, 2)));
        assert_eq!(update.position_qty, Some(Decimal::ZERO));
    }

    #[test]
    fn stream_event_from_other_message() {
        let json = r#"{"stream":"listening","data":{"streams":["trade_updates"]}}"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let event = if value.get("stream").and_then(|s| s.as_str()) == Some("trade_updates") {
            None
        } else {
            Some(StreamEvent::Other(value))
        };
        assert!(matches!(event, Some(StreamEvent::Other(_))));
    }

    /// A minimal mock trading-stream server: expects `auth` then `listen`,
    /// answers both, then sends one trade update and closes.
    async fn serve_once(listener: tokio::net::TcpListener) {
        type ServerWs = WebSocketStream<tokio::net::TcpStream>;

        async fn read_frame(ws: &mut ServerWs) -> serde_json::Value {
            match ws.next().await {
                Some(Ok(Message::Text(text))) => {
                    serde_json::from_str::<serde_json::Value>(text.as_str()).unwrap()
                }
                other => panic!("expected a text frame, got: {other:?}"),
            }
        }
        async fn send_frame(ws: &mut ServerWs, text: &str) {
            ws.send(Message::Text(text.into())).await.unwrap();
        }

        let (tcp, _) = listener.accept().await.unwrap();
        let mut ws = tokio_tungstenite::accept_async(tcp).await.unwrap();

        let auth = read_frame(&mut ws).await;
        assert_eq!(auth["action"], "auth");
        assert_eq!(auth["key"], "key-id");
        assert_eq!(auth["secret"], "secret");
        send_frame(
            &mut ws,
            r#"{"stream":"authorization","data":{"status":"authorized"}}"#,
        )
        .await;

        let listen = read_frame(&mut ws).await;
        assert_eq!(listen["action"], "listen");
        assert_eq!(listen["data"]["streams"][0], "trade_updates");
        send_frame(
            &mut ws,
            r#"{"stream":"listening","data":{"streams":["trade_updates"]}}"#,
        )
        .await;

        send_frame(&mut ws, TRADE_UPDATE_JSON).await;
    }

    /// The full connect/auth/listen/read flow against a local mock server,
    /// exercising the URL override (`TradingStream::connect`) offline.
    #[tokio::test]
    async fn connect_auth_listen_against_mock_server() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(serve_once(listener));

        let creds = Credentials::Key {
            key_id: "key-id".into(),
            secret_key: "secret".into(),
        };
        let mut stream = TradingStream::connect(&format!("ws://{addr}/stream"), &creds)
            .await
            .unwrap();
        stream.listen_trade_updates().await.unwrap();

        match stream.next().await.unwrap() {
            Some(StreamEvent::TradeUpdate(update)) => {
                assert_eq!(update.event, TradeEvent::Fill);
                assert_eq!(update.order.symbol, "AAPL");
            }
            other => panic!("expected a trade update, got: {other:?}"),
        }
        server.await.unwrap();
    }
}
