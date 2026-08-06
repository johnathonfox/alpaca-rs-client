//! Real-time market data WebSocket stream.
//!
//! JSON text frames (not msgpack). Protocol: on connect the server sends
//! `[{"T":"success","msg":"connected"}]`; the client must authenticate within
//! 10 seconds with `{"action":"auth","key":..,"secret":..}` and then
//! subscribe to channels.
//!
//! Auto-reconnect is opt-in: by default callers should reconnect (and
//! re-subscribe) when [`MarketDataStream::next`] returns an error or
//! `Ok(None)`. With [`MarketDataStream::set_auto_reconnect`] the client does
//! this itself, restoring the last subscription set.

use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{Message, http};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::data::enums::{CryptoFeed, DataFeed, OptionsFeed};
use crate::data::models::{OrderbookEntry, TradeId};
use crate::error::{Error, Result};
use crate::rest::Credentials;
use crate::stream::ReconnectOptions;

const STREAM_HOST: &str = "stream.data.alpaca.markets";

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// A trade message from the data stream (`T = "t"`).
#[derive(Debug, Clone, Deserialize)]
pub struct StreamTrade {
    /// Symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Trade price.
    #[serde(rename = "p")]
    pub price: f64,
    /// Trade size.
    #[serde(rename = "s")]
    pub size: f64,
    /// Exchange code.
    #[serde(rename = "x")]
    pub exchange: Option<String>,
    /// Trade condition codes.
    #[serde(rename = "c", default)]
    pub conditions: Vec<String>,
    /// Trade id (numeric for stocks/options, string for crypto).
    #[serde(rename = "i")]
    pub id: Option<TradeId>,
    /// Tape.
    #[serde(rename = "z")]
    pub tape: Option<String>,
    /// Trade timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
}

/// A quote message from the data stream (`T = "q"`).
#[derive(Debug, Clone, Deserialize)]
pub struct StreamQuote {
    /// Symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Bid exchange code.
    #[serde(rename = "bx")]
    pub bid_exchange: Option<String>,
    /// Bid price.
    #[serde(rename = "bp")]
    pub bid_price: f64,
    /// Bid size.
    #[serde(rename = "bs")]
    pub bid_size: f64,
    /// Ask exchange code.
    #[serde(rename = "ax")]
    pub ask_exchange: Option<String>,
    /// Ask price.
    #[serde(rename = "ap")]
    pub ask_price: f64,
    /// Ask size.
    #[serde(rename = "as")]
    pub ask_size: f64,
    /// Quote condition codes.
    #[serde(rename = "c", default)]
    pub conditions: Vec<String>,
    /// Tape.
    #[serde(rename = "z")]
    pub tape: Option<String>,
    /// Quote timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
}

/// A bar message from the data stream (`T = "b"`, `"u"` or `"d"`).
#[derive(Debug, Clone, Deserialize)]
pub struct StreamBar {
    /// Symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Open price.
    #[serde(rename = "o")]
    pub open: f64,
    /// High price.
    #[serde(rename = "h")]
    pub high: f64,
    /// Low price.
    #[serde(rename = "l")]
    pub low: f64,
    /// Close price.
    #[serde(rename = "c")]
    pub close: f64,
    /// Volume.
    #[serde(rename = "v")]
    pub volume: f64,
    /// Number of trades in the bar.
    #[serde(rename = "n")]
    pub trade_count: Option<u64>,
    /// Volume-weighted average price.
    #[serde(rename = "vw")]
    pub vwap: Option<f64>,
    /// Bar timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
}

/// An orderbook message from the data stream (`T = "o"`).
#[derive(Debug, Clone, Deserialize)]
pub struct StreamOrderbook {
    /// Symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Bid levels.
    #[serde(rename = "b", default)]
    pub bids: Vec<OrderbookEntry>,
    /// Ask levels.
    #[serde(rename = "a", default)]
    pub asks: Vec<OrderbookEntry>,
    /// Book timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
}

/// A trading status message (`T = "s"`, e.g. halts and resumptions).
#[derive(Debug, Clone, Deserialize)]
pub struct StreamStatus {
    /// Symbol.
    #[serde(rename = "S")]
    pub symbol: Option<String>,
    /// Status code.
    #[serde(rename = "sc")]
    pub status_code: Option<String>,
    /// Status message.
    #[serde(rename = "sm")]
    pub status_message: Option<String>,
    /// Timestamp.
    #[serde(rename = "t")]
    pub timestamp: Option<DateTime<Utc>>,
}

/// A limit-up/limit-down message (`T = "l"`).
#[derive(Debug, Clone, Deserialize)]
pub struct StreamLuld {
    /// Symbol.
    #[serde(rename = "S")]
    pub symbol: Option<String>,
    /// Limit-up price band.
    #[serde(rename = "u")]
    pub limit_up: Option<f64>,
    /// Limit-down price band.
    #[serde(rename = "d")]
    pub limit_down: Option<f64>,
    /// LULD indicator.
    #[serde(rename = "i")]
    pub indicator: Option<String>,
    /// Tape.
    #[serde(rename = "z")]
    pub tape: Option<String>,
    /// Timestamp.
    #[serde(rename = "t")]
    pub timestamp: Option<DateTime<Utc>>,
}

/// A news message (`T = "n"`).
#[derive(Debug, Clone, Deserialize)]
pub struct StreamNews {
    /// Article id.
    pub id: Option<u64>,
    /// Headline.
    pub headline: Option<String>,
    /// Summary.
    pub summary: Option<String>,
    /// Author.
    pub author: Option<String>,
    /// Symbols mentioned.
    #[serde(default)]
    pub symbols: Vec<String>,
    /// Source.
    pub source: Option<String>,
}

/// The action reported by a cancel error message (the `"a"` field).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CancelErrorAction {
    /// The trade was canceled.
    #[serde(rename = "canceled")]
    Canceled,
    /// The trade was marked as erroneous.
    #[serde(rename = "errored")]
    Errored,
}

/// A trade correction message (`T = "c"`): reports both the corrected trade
/// and the original values it replaces.
#[derive(Debug, Clone, Deserialize)]
pub struct StreamCorrection {
    /// Symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Trade id of the corrected trade.
    #[serde(rename = "i")]
    pub id: Option<TradeId>,
    /// Exchange code.
    #[serde(rename = "x")]
    pub exchange: Option<String>,
    /// Corrected trade price.
    #[serde(rename = "p")]
    pub price: f64,
    /// Corrected trade size.
    #[serde(rename = "s")]
    pub size: f64,
    /// Correction timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Corrected trade condition codes.
    #[serde(rename = "c", default)]
    pub conditions: Vec<String>,
    /// Tape.
    #[serde(rename = "z")]
    pub tape: Option<String>,
    /// Trade id of the original trade.
    #[serde(rename = "oi")]
    pub original_id: Option<TradeId>,
    /// Price of the original trade.
    #[serde(rename = "op")]
    pub original_price: Option<f64>,
    /// Size of the original trade.
    #[serde(rename = "os")]
    pub original_size: Option<f64>,
    /// Timestamp of the original trade.
    #[serde(rename = "ot")]
    pub original_timestamp: Option<DateTime<Utc>>,
    /// Condition codes of the original trade.
    #[serde(rename = "oc", default)]
    pub original_conditions: Vec<String>,
}

/// A cancel error message (`T = "x"`): a previously reported trade was
/// canceled or marked as an error.
#[derive(Debug, Clone, Deserialize)]
pub struct StreamCancelError {
    /// Symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Trade id of the canceled/erroneous trade.
    #[serde(rename = "i")]
    pub id: Option<TradeId>,
    /// Exchange code.
    #[serde(rename = "x")]
    pub exchange: Option<String>,
    /// Price of the canceled/erroneous trade.
    #[serde(rename = "p")]
    pub price: f64,
    /// Size of the canceled/erroneous trade.
    #[serde(rename = "s")]
    pub size: f64,
    /// Timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Whether the trade was canceled or marked as erroneous.
    #[serde(rename = "a")]
    pub action: Option<CancelErrorAction>,
    /// Tape.
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

/// A success control message (`T = "success"`, e.g. `"connected"` or
/// `"authenticated"`).
#[derive(Debug, Clone, Deserialize)]
pub struct StreamSuccess {
    /// Human-readable message.
    #[serde(default)]
    pub msg: String,
}

/// An error control message (`T = "error"`).
#[derive(Debug, Clone, Deserialize)]
pub struct StreamError {
    /// Error code (e.g. 406 for "connection limit exceeded").
    #[serde(default)]
    pub code: Option<i32>,
    /// Error message.
    #[serde(default)]
    pub msg: String,
}

/// One message from the market data stream, tagged by the `"T"` field.
///
/// Any unmodeled message types (including future additions) deserialize as
/// [`DataMessage::Unknown`].
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "T")]
pub enum DataMessage {
    /// Trade (`T = "t"`).
    #[serde(rename = "t")]
    Trade(StreamTrade),
    /// Quote (`T = "q"`).
    #[serde(rename = "q")]
    Quote(StreamQuote),
    /// Minute bar (`T = "b"`).
    #[serde(rename = "b")]
    Bar(StreamBar),
    /// Updated bar (`T = "u"`).
    #[serde(rename = "u")]
    UpdatedBar(StreamBar),
    /// Daily bar (`T = "d"`).
    #[serde(rename = "d")]
    DailyBar(StreamBar),
    /// Orderbook (`T = "o"`).
    #[serde(rename = "o")]
    Orderbook(StreamOrderbook),
    /// Trading status (`T = "s"`).
    #[serde(rename = "s")]
    Status(StreamStatus),
    /// Limit up/limit down (`T = "l"`).
    #[serde(rename = "l")]
    Luld(StreamLuld),
    /// News (`T = "n"`).
    #[serde(rename = "n")]
    News(StreamNews),
    /// Trade correction (`T = "c"`).
    #[serde(rename = "c")]
    Correction(StreamCorrection),
    /// Cancel error (`T = "x"`).
    #[serde(rename = "x")]
    CancelError(StreamCancelError),
    /// Success control message (`T = "success"`).
    #[serde(rename = "success")]
    Success(StreamSuccess),
    /// Error control message (`T = "error"`).
    #[serde(rename = "error")]
    Error(StreamError),
    /// Subscription confirmation (`T = "subscription"`): the full set of
    /// channels/symbols the server currently considers subscribed.
    #[serde(rename = "subscription")]
    Subscription(Subscription),
    /// Any other message type (future additions).
    #[serde(other)]
    Unknown,
}

/// Channels and symbols to subscribe to. `"*"` subscribes to all symbols of
/// a channel; empty channels are omitted from the subscribe message. Also
/// used as the payload of subscription confirmation messages.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    /// Trade symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trades: Vec<String>,
    /// Quote symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quotes: Vec<String>,
    /// Minute bar symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bars: Vec<String>,
    /// Updated bar symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub updated_bars: Vec<String>,
    /// Daily bar symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub daily_bars: Vec<String>,
    /// Trading status symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<String>,
    /// LULD symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lulds: Vec<String>,
    /// Orderbook symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub orderbooks: Vec<String>,
}

impl Subscription {
    /// Returns true when no channel has any symbols.
    fn is_empty(&self) -> bool {
        self.trades.is_empty()
            && self.quotes.is_empty()
            && self.bars.is_empty()
            && self.updated_bars.is_empty()
            && self.daily_bars.is_empty()
            && self.statuses.is_empty()
            && self.lulds.is_empty()
            && self.orderbooks.is_empty()
    }

    /// Merges `other` into `self`, skipping symbols already present.
    fn add(&mut self, other: &Subscription) {
        fn add_symbols(target: &mut Vec<String>, new: &[String]) {
            for symbol in new {
                if !target.contains(symbol) {
                    target.push(symbol.clone());
                }
            }
        }
        add_symbols(&mut self.trades, &other.trades);
        add_symbols(&mut self.quotes, &other.quotes);
        add_symbols(&mut self.bars, &other.bars);
        add_symbols(&mut self.updated_bars, &other.updated_bars);
        add_symbols(&mut self.daily_bars, &other.daily_bars);
        add_symbols(&mut self.statuses, &other.statuses);
        add_symbols(&mut self.lulds, &other.lulds);
        add_symbols(&mut self.orderbooks, &other.orderbooks);
    }

    /// Removes the symbols of `other` from `self` (subtractive).
    fn remove(&mut self, other: &Subscription) {
        fn remove_symbols(target: &mut Vec<String>, gone: &[String]) {
            target.retain(|symbol| !gone.contains(symbol));
        }
        remove_symbols(&mut self.trades, &other.trades);
        remove_symbols(&mut self.quotes, &other.quotes);
        remove_symbols(&mut self.bars, &other.bars);
        remove_symbols(&mut self.updated_bars, &other.updated_bars);
        remove_symbols(&mut self.daily_bars, &other.daily_bars);
        remove_symbols(&mut self.statuses, &other.statuses);
        remove_symbols(&mut self.lulds, &other.lulds);
        remove_symbols(&mut self.orderbooks, &other.orderbooks);
    }
}

#[derive(Serialize)]
struct StreamAction<'a> {
    action: &'a str,
    #[serde(flatten)]
    channels: &'a Subscription,
}

async fn send_json(ws: &mut Ws, value: &impl Serialize) -> Result<()> {
    let text = serde_json::to_string(value)?;
    ws.send(Message::Text(text.into())).await?;
    Ok(())
}

/// Reads one text message and returns its raw string.
async fn read_text(ws: &mut Ws) -> Result<String> {
    loop {
        match ws.next().await {
            None => return Err(Error::StreamClosed),
            Some(Err(e)) => return Err(Error::WebSocket(e)),
            Some(Ok(Message::Text(text))) => return Ok(text.to_string()),
            Some(Ok(Message::Close(_))) => return Err(Error::StreamClosed),
            // Ping/Pong handled by tungstenite; skip binary/frame messages.
            Some(Ok(_)) => continue,
        }
    }
}

/// Waits for a `[{"T":"success","msg":"..expected.."}]` control message;
/// returns [`Error::Stream`] on an `error` control message.
async fn expect_control(ws: &mut Ws, expected: &str) -> Result<()> {
    let text = read_text(ws).await?;
    let messages: Vec<serde_json::Value> = serde_json::from_str(&text)?;
    for message in &messages {
        let kind = message.get("T").and_then(|t| t.as_str()).unwrap_or("");
        match kind {
            "success" => {
                let msg = message.get("msg").and_then(|m| m.as_str()).unwrap_or("");
                if msg.contains(expected) {
                    return Ok(());
                }
            }
            "error" => {
                let msg = message
                    .get("msg")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown error");
                return Err(Error::Stream(msg.to_string()));
            }
            _ => {}
        }
    }
    Err(Error::Stream(format!(
        "expected a '{expected}' control message, got: {text}"
    )))
}

/// Market data WebSocket stream client.
pub struct MarketDataStream {
    ws: Ws,
    url: String,
    creds: Credentials,
    subscriptions: Subscription,
    reconnect: Option<ReconnectOptions>,
}

impl MarketDataStream {
    /// Connects and authenticates to an arbitrary data stream URL (e.g.
    /// `wss://stream.data.alpaca.markets/v2/iex`).
    pub async fn connect(url: &str, creds: &Credentials) -> Result<Self> {
        let mut request = url.into_client_request()?;
        request.headers_mut().insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/json"),
        );
        let (mut ws, _response) = connect_async(request).await?;

        expect_control(&mut ws, "connected").await?;

        let (key, secret) = creds.key_secret();
        send_json(
            &mut ws,
            &serde_json::json!({"action": "auth", "key": key, "secret": secret}),
        )
        .await?;
        expect_control(&mut ws, "authenticated").await?;

        tracing::debug!(url, "market data stream connected and authenticated");
        Ok(Self {
            ws,
            url: url.to_string(),
            creds: creds.clone(),
            subscriptions: Subscription::default(),
            reconnect: None,
        })
    }

    /// Connects to the stock data stream for the given feed.
    pub async fn stocks(feed: DataFeed, creds: &Credentials) -> Result<Self> {
        Self::connect(&format!("wss://{STREAM_HOST}/v2/{}", feed.as_str()), creds).await
    }

    /// Connects to the crypto data stream for the given feed.
    pub async fn crypto(feed: CryptoFeed, creds: &Credentials) -> Result<Self> {
        Self::connect(
            &format!("wss://{STREAM_HOST}/v1beta3/crypto/{}", feed.as_str()),
            creds,
        )
        .await
    }

    /// Connects to the option data stream for the given feed.
    pub async fn options(feed: OptionsFeed, creds: &Credentials) -> Result<Self> {
        Self::connect(
            &format!("wss://{STREAM_HOST}/v1beta1/{}", feed.as_str()),
            creds,
        )
        .await
    }

    /// Connects to the news stream.
    pub async fn news(creds: &Credentials) -> Result<Self> {
        Self::connect(&format!("wss://{STREAM_HOST}/v1beta1/news"), creds).await
    }

    /// Enables automatic reconnection. On an unexpected close or transport
    /// error, [`MarketDataStream::next`] reconnects (with the backoff
    /// described by [`ReconnectOptions`]), re-authenticates and resubscribes
    /// the current [`MarketDataStream::subscriptions`] before yielding the
    /// next messages. Disabled by default.
    pub fn set_auto_reconnect(&mut self, options: ReconnectOptions) {
        self.reconnect = Some(options);
    }

    /// Disables automatic reconnection (the default).
    pub fn disable_auto_reconnect(&mut self) {
        self.reconnect = None;
    }

    /// The subscription set currently tracked by the client — the union of
    /// all successful [`MarketDataStream::subscribe`] calls minus all
    /// [`MarketDataStream::unsubscribe`] calls. This is the set restored after
    /// an automatic reconnect.
    pub fn subscriptions(&self) -> &Subscription {
        &self.subscriptions
    }

    /// Subscribes to the given channels/symbols. The server's subscription
    /// confirmation arrives via [`MarketDataStream::next`] as
    /// [`DataMessage::Subscription`].
    pub async fn subscribe(&mut self, subscription: &Subscription) -> Result<()> {
        send_json(
            &mut self.ws,
            &StreamAction {
                action: "subscribe",
                channels: subscription,
            },
        )
        .await?;
        self.subscriptions.add(subscription);
        Ok(())
    }

    /// Unsubscribes from the given channels/symbols (subtractive).
    pub async fn unsubscribe(&mut self, subscription: &Subscription) -> Result<()> {
        send_json(
            &mut self.ws,
            &StreamAction {
                action: "unsubscribe",
                channels: subscription,
            },
        )
        .await?;
        self.subscriptions.remove(subscription);
        Ok(())
    }

    /// Reconnects, re-authenticates and resubscribes the tracked
    /// subscription set, with exponential backoff between attempts.
    async fn reconnect(&mut self, options: &ReconnectOptions) -> Result<()> {
        let mut backoff = options.initial_backoff;
        let mut last_error = Error::StreamClosed;
        for attempt in 1..=options.max_attempts {
            match Self::connect(&self.url, &self.creds).await {
                Ok(stream) => {
                    self.ws = stream.ws;
                    if !self.subscriptions.is_empty() {
                        send_json(
                            &mut self.ws,
                            &StreamAction {
                                action: "subscribe",
                                channels: &self.subscriptions,
                            },
                        )
                        .await?;
                    }
                    tracing::info!(attempt, "market data stream reconnected and resubscribed");
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!(attempt, error = %e, "market data stream reconnect failed");
                    last_error = e;
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(options.max_backoff);
                }
            }
        }
        Err(last_error)
    }

    /// Reads the next batch of messages. Returns `Ok(None)` when the server
    /// closes the connection cleanly and auto-reconnect is disabled; when
    /// auto-reconnect is enabled, a close or transport error instead triggers
    /// a reconnect (see [`MarketDataStream::set_auto_reconnect`]).
    pub async fn next(&mut self) -> Result<Option<Vec<DataMessage>>> {
        let result = Self::read_batch(&mut self.ws).await;
        let Some(options) = self.reconnect else {
            return result;
        };
        match result {
            Ok(Some(messages)) => Ok(Some(messages)),
            Ok(None) | Err(_) => {
                self.reconnect(&options).await?;
                Self::read_batch(&mut self.ws).await
            }
        }
    }

    /// Reads the next text frame and parses it into a batch of messages.
    async fn read_batch(ws: &mut Ws) -> Result<Option<Vec<DataMessage>>> {
        loop {
            match ws.next().await {
                None => return Ok(None),
                Some(Err(e)) => return Err(Error::WebSocket(e)),
                Some(Ok(Message::Text(text))) => {
                    let messages: Vec<DataMessage> = serde_json::from_str(text.as_str())?;
                    return Ok(Some(messages));
                }
                Some(Ok(Message::Close(_))) => return Ok(None),
                // Ping/Pong handled by tungstenite; skip binary/frame messages.
                Some(Ok(_)) => continue,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_quote_message() {
        let json = r#"[{"T":"q","S":"FAKEPACA","bx":"O","bp":133.85,"bs":4,"ax":"R","ap":135.77,"as":5,"c":["R"],"z":"A","t":"2024-07-24T07:56:53.639713735Z"}]"#;
        let messages: Vec<DataMessage> = serde_json::from_str(json).unwrap();
        assert_eq!(messages.len(), 1);
        match &messages[0] {
            DataMessage::Quote(q) => {
                assert_eq!(q.symbol, "FAKEPACA");
                assert_eq!(q.bid_exchange.as_deref(), Some("O"));
                assert!((q.bid_price - 133.85).abs() < 1e-9);
                assert_eq!(q.bid_size, 4.0);
                assert!((q.ask_price - 135.77).abs() < 1e-9);
                assert_eq!(q.conditions, vec!["R"]);
                assert_eq!(q.tape.as_deref(), Some("A"));
            }
            other => panic!("expected quote, got {other:?}"),
        }
    }

    #[test]
    fn parse_bar_message() {
        let json = r#"[{"T":"b","S":"FAKEPACA","o":132.65,"h":136,"l":132.12,"c":134.65,"v":205,"t":"2024-07-24T07:56:00Z","n":16,"vw":133.7}]"#;
        let messages: Vec<DataMessage> = serde_json::from_str(json).unwrap();
        match &messages[0] {
            DataMessage::Bar(b) => {
                assert_eq!(b.symbol, "FAKEPACA");
                assert!((b.open - 132.65).abs() < 1e-9);
                assert_eq!(b.volume, 205.0);
                assert_eq!(b.trade_count, Some(16));
                assert_eq!(b.vwap, Some(133.7));
            }
            other => panic!("expected bar, got {other:?}"),
        }
    }

    #[test]
    fn parse_correction_message() {
        let json = r#"[{"T":"c","S":"AAPL","i":52876214943535,"x":"Q","p":127.55,"s":50,"t":"2022-08-01T19:40:01.018306912Z","c":["@","T","I"],"z":"C","oi":52876214943399,"op":127.5,"os":40,"ot":"2022-08-01T19:40:00.018306912Z","oc":["@"]}]"#;
        let messages: Vec<DataMessage> = serde_json::from_str(json).unwrap();
        assert_eq!(messages.len(), 1);
        match &messages[0] {
            DataMessage::Correction(c) => {
                assert_eq!(c.symbol, "AAPL");
                assert_eq!(c.id, Some(TradeId::Int(52876214943535)));
                assert_eq!(c.exchange.as_deref(), Some("Q"));
                assert!((c.price - 127.55).abs() < 1e-9);
                assert_eq!(c.size, 50.0);
                assert_eq!(c.conditions, vec!["@", "T", "I"]);
                assert_eq!(c.tape.as_deref(), Some("C"));
                assert_eq!(c.original_id, Some(TradeId::Int(52876214943399)));
                assert_eq!(c.original_price, Some(127.5));
                assert_eq!(c.original_size, Some(40.0));
                assert!(c.original_timestamp.is_some());
                assert_eq!(c.original_conditions, vec!["@"]);
            }
            other => panic!("expected correction, got {other:?}"),
        }
    }

    #[test]
    fn parse_cancel_error_message() {
        let json = r#"[{"T":"x","S":"AAPL","i":52876214943535,"x":"Q","p":127.55,"s":50,"t":"2022-08-01T19:40:01.018306912Z","a":"canceled","z":"C"}]"#;
        let messages: Vec<DataMessage> = serde_json::from_str(json).unwrap();
        match &messages[0] {
            DataMessage::CancelError(x) => {
                assert_eq!(x.symbol, "AAPL");
                assert_eq!(x.id, Some(TradeId::Int(52876214943535)));
                assert_eq!(x.exchange.as_deref(), Some("Q"));
                assert!((x.price - 127.55).abs() < 1e-9);
                assert_eq!(x.size, 50.0);
                assert_eq!(x.action, Some(CancelErrorAction::Canceled));
                assert_eq!(x.tape.as_deref(), Some("C"));
            }
            other => panic!("expected cancel error, got {other:?}"),
        }
    }

    #[test]
    fn parse_control_messages() {
        let json = r#"[
            {"T":"success","msg":"connected"},
            {"T":"success","msg":"authenticated"},
            {"T":"subscription","trades":["AAPL"],"quotes":[],"bars":[],"dailyBars":["*"],"updatedBars":[],"statuses":[],"lulds":[],"orderbooks":[]},
            {"T":"error","code":406,"msg":"connection limit exceeded"}
        ]"#;
        let messages: Vec<DataMessage> = serde_json::from_str(json).unwrap();
        assert_eq!(messages.len(), 4);
        match &messages[0] {
            DataMessage::Success(s) => assert_eq!(s.msg, "connected"),
            other => panic!("expected success, got {other:?}"),
        }
        match &messages[2] {
            DataMessage::Subscription(s) => {
                assert_eq!(s.trades, vec!["AAPL"]);
                assert_eq!(s.daily_bars, vec!["*"]);
                assert!(s.quotes.is_empty());
            }
            other => panic!("expected subscription confirmation, got {other:?}"),
        }
        match &messages[3] {
            DataMessage::Error(e) => {
                assert_eq!(e.code, Some(406));
                assert_eq!(e.msg, "connection limit exceeded");
            }
            other => panic!("expected error, got {other:?}"),
        }
    }

    #[test]
    fn subscription_bookkeeping_add_and_remove() {
        let mut tracked = Subscription::default();
        assert!(tracked.is_empty());

        let first = Subscription {
            trades: vec!["AAPL".to_string(), "MSFT".to_string()],
            bars: vec!["AAPL".to_string()],
            ..Default::default()
        };
        tracked.add(&first);
        tracked.add(&first); // duplicates are ignored
        assert_eq!(tracked.trades, vec!["AAPL", "MSFT"]);
        assert_eq!(tracked.bars, vec!["AAPL"]);

        let second = Subscription {
            trades: vec!["TSLA".to_string()],
            ..Default::default()
        };
        tracked.add(&second);
        assert_eq!(tracked.trades, vec!["AAPL", "MSFT", "TSLA"]);

        tracked.remove(&Subscription {
            trades: vec!["MSFT".to_string()],
            bars: vec!["AAPL".to_string()],
            ..Default::default()
        });
        assert_eq!(tracked.trades, vec!["AAPL", "TSLA"]);
        assert!(tracked.bars.is_empty());

        // Removing symbols that were never subscribed is a no-op.
        tracked.remove(&Subscription {
            trades: vec!["GOOG".to_string()],
            ..Default::default()
        });
        assert_eq!(tracked.trades, vec!["AAPL", "TSLA"]);

        tracked.remove(&tracked.clone());
        assert!(tracked.is_empty());
    }

    #[test]
    fn subscription_omits_empty_channels() {
        let subscription = Subscription {
            trades: vec!["AAPL".to_string()],
            bars: vec!["*".to_string()],
            ..Default::default()
        };
        let json = serde_json::to_value(&StreamAction {
            action: "subscribe",
            channels: &subscription,
        })
        .unwrap();
        assert_eq!(json["action"], "subscribe");
        assert_eq!(json["trades"], serde_json::json!(["AAPL"]));
        assert_eq!(json["bars"], serde_json::json!(["*"]));
        assert!(json.get("quotes").is_none());
        assert!(json.get("updatedBars").is_none());
    }
}
