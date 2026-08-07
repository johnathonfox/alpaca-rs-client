//! Shared Server-Sent Events transport for the event-stream clients.
//!
//! Alpaca's events endpoints are plain HTTP `GET`s that never finish: the
//! response body is a `text/event-stream` of JSON payloads. [`EventStream`]
//! wraps such a response, feeding its bytes through
//! [`SseParser`] and decoding each frame into `T`.
//!
//! Both endpoints replay history before switching to live pushes, which makes
//! resumption the interesting part: every frame carries a lexically sortable
//! ULID event id, and reconnecting with that id in the `Last-Event-Id` header
//! resumes where the previous connection stopped. [`EventStream`] tracks the
//! id for you and, when auto-reconnect is enabled via
//! [`set_auto_reconnect`](EventStream::set_auto_reconnect), sends it
//! automatically. The event with that id is redelivered, so consumers should
//! deduplicate on it.

use std::collections::VecDeque;

/// Cap on bytes buffered while waiting for a line terminator.
///
/// SSE lines are short, so this is only reached by a peer (or a broken proxy)
/// streaming endlessly without a newline. Bounding it turns what would be
/// unbounded memory growth on a long-lived stream into an error.
const MAX_PENDING_FRAME_BYTES: usize = 8 * 1024 * 1024;

use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

use crate::error::{Error, Result};
use crate::rest::Credentials;
use crate::stream::ReconnectOptions;
use crate::stream::sse::SseParser;

/// Rejects a query parameter combination the events endpoints refuse.
///
/// Both endpoints require `until`/`until_id` to be paired with the matching
/// `since`/`since_id`, and treat `since` and `since_id` as mutually
/// exclusive; sending an invalid combination is a 400 from the server.
pub(crate) fn validate_replay_window(
    since: bool,
    until: bool,
    since_id: bool,
    until_id: bool,
) -> Result<()> {
    if until && !since {
        return Err(Error::InvalidRequest(
            "`until` requires `since` on an event stream".into(),
        ));
    }
    if until_id && !since_id {
        return Err(Error::InvalidRequest(
            "`until_id` requires `since_id` on an event stream".into(),
        ));
    }
    if since && since_id {
        return Err(Error::InvalidRequest(
            "`since` and `since_id` cannot both be set on an event stream".into(),
        ));
    }
    Ok(())
}

/// A live Server-Sent Events connection yielding decoded events of type `T`.
///
/// Obtained from
/// [`ActivityEventsClient::subscribe`](super::activity_events::ActivityEventsClient::subscribe)
/// or
/// [`CorporateActionEventsClient::subscribe`](super::ca_events::CorporateActionEventsClient::subscribe).
/// Drive it by calling [`next`](Self::next) in a loop; it returns `Ok(None)`
/// once the server closes the stream (which it does at `until`/`until_id`,
/// and on an unexpected disconnect when auto-reconnect is off).
pub struct EventStream<T> {
    /// HTTP client reused across reconnects.
    http: reqwest::Client,
    /// Fully resolved endpoint URL, without the query string.
    url: Url,
    /// Serialized query parameters, replayed on every reconnect.
    query: serde_json::Value,
    /// Credentials for the `APCA-API-*` (or bearer) headers.
    creds: Credentials,
    /// Frame parser fed by the response body.
    parser: SseParser,
    /// The open response, or `None` once its body is exhausted.
    response: Option<reqwest::Response>,
    /// Events decoded from frames but not yet returned to the caller.
    pending: VecDeque<T>,
    /// Most recent event id, sent as `Last-Event-Id` when reconnecting.
    last_event_id: Option<String>,
    /// Auto-reconnect settings; `None` (the default) disables reconnection.
    reconnect: Option<ReconnectOptions>,
}

/// Written by hand rather than derived so that it works for any `T` and so
/// that the credentials never reach the formatter.
impl<T> std::fmt::Debug for EventStream<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventStream")
            .field("url", &self.url)
            .field("query", &self.query)
            .field("connected", &self.response.is_some())
            .field("pending", &self.pending.len())
            .field("last_event_id", &self.last_event_id)
            .field("reconnect", &self.reconnect)
            .finish()
    }
}

impl<T: DeserializeOwned> EventStream<T> {
    /// Opens a stream against `base` + `path` with the given query
    /// parameters.
    pub(crate) async fn connect(
        base: &Url,
        path: &str,
        query: &(impl Serialize + ?Sized),
        creds: &Credentials,
    ) -> Result<Self> {
        let http = reqwest::Client::new();
        let url = base.join(path)?;
        let query = serde_json::to_value(query)?;
        let response = Self::open(&http, &url, &query, creds, None).await?;
        tracing::debug!(url = %url, "event stream connected");
        Ok(Self {
            http,
            url,
            query,
            creds: creds.clone(),
            parser: SseParser::new(),
            response: Some(response),
            pending: VecDeque::new(),
            last_event_id: None,
            reconnect: None,
        })
    }

    /// Issues the streaming request, mapping a non-success status onto
    /// [`Error::Api`].
    async fn open(
        http: &reqwest::Client,
        url: &Url,
        query: &serde_json::Value,
        creds: &Credentials,
        last_event_id: Option<&str>,
    ) -> Result<reqwest::Response> {
        let mut req = http
            .get(url.clone())
            .query(query)
            .header(reqwest::header::ACCEPT, "text/event-stream");
        if let Some(id) = last_event_id {
            req = req.header("Last-Event-Id", id);
        }
        let response = creds.authenticate(req).send().await?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::Api {
                status: status.as_u16(),
                message: response.text().await?,
            });
        }
        Ok(response)
    }

    /// Enables automatic reconnection. On an unexpected close or transport
    /// error, [`next`](Self::next) reopens the stream (with the backoff
    /// described by [`ReconnectOptions`]), resuming from the last event id
    /// via the `Last-Event-Id` header. Disabled by default.
    pub fn set_auto_reconnect(&mut self, options: ReconnectOptions) {
        self.reconnect = Some(options);
    }

    /// Disables automatic reconnection (the default).
    pub fn disable_auto_reconnect(&mut self) {
        self.reconnect = None;
    }

    /// The id of the most recently seen event, or `None` before the first one
    /// arrives. Persist it to resume a later run with `since_id`.
    pub fn last_event_id(&self) -> Option<&str> {
        self.last_event_id.as_deref()
    }

    /// Returns the next event. Yields `Ok(None)` when the stream is finished.
    ///
    /// With auto-reconnect enabled (see
    /// [`set_auto_reconnect`](Self::set_auto_reconnect)) a transport error
    /// triggers a reconnect instead of surfacing. So does a clean
    /// end-of-stream on an *unbounded* subscription, where the body ending is
    /// the server dropping a stream that was meant to run forever.
    ///
    /// A *bounded* subscription — one carrying `until` or `until_id` — is the
    /// exception: there, the server closing the body means it delivered the
    /// whole requested window, so this returns `Ok(None)` even with
    /// auto-reconnect on. Reconnecting would re-request the same finite
    /// window and replay it forever.
    pub async fn next(&mut self) -> Result<Option<T>> {
        let result = self.next_buffered().await;
        let Some(options) = self.reconnect else {
            return result;
        };
        match result {
            Ok(Some(event)) => Ok(Some(event)),
            Ok(None) if self.is_bounded() => Ok(None),
            Ok(None) | Err(_) => {
                self.reconnect(&options).await?;
                self.next_buffered().await
            }
        }
    }

    /// Whether the subscription asked for a finite window (`until` /
    /// `until_id`), which the server terminates deliberately once delivered.
    fn is_bounded(&self) -> bool {
        ["until", "until_id"]
            .iter()
            .any(|key| self.query.get(key).is_some_and(|value| !value.is_null()))
    }

    /// Drains the decoded backlog, reading more of the body when it is empty.
    async fn next_buffered(&mut self) -> Result<Option<T>> {
        loop {
            if let Some(event) = self.pending.pop_front() {
                return Ok(Some(event));
            }
            if !self.read_chunk().await? {
                return Ok(None);
            }
        }
    }

    /// Reads one body chunk and decodes any frames it completes. Returns
    /// `false` once the response body is exhausted.
    async fn read_chunk(&mut self) -> Result<bool> {
        let Some(response) = self.response.as_mut() else {
            return Ok(false);
        };
        let Some(chunk) = response.chunk().await? else {
            self.response = None;
            return Ok(false);
        };
        self.parser.push(&chunk);
        if self.parser.buffered_len() > MAX_PENDING_FRAME_BYTES {
            self.response = None;
            return Err(Error::Stream(format!(
                "event stream sent over {MAX_PENDING_FRAME_BYTES} bytes with no line terminator"
            )));
        }
        while let Some(frame) = self.parser.next_frame() {
            for event in decode_frame(&frame.data)? {
                self.pending.push_back(event);
            }
        }
        // Read straight off the parser: `id:` lines also arrive on frames that
        // carry no data, and those never surface as a `SseFrame`.
        if let Some(id) = self.parser.last_event_id() {
            self.last_event_id = Some(id.to_string());
        }
        Ok(true)
    }

    /// Reopens the stream from the last event id, with exponential backoff
    /// between attempts.
    async fn reconnect(&mut self, options: &ReconnectOptions) -> Result<()> {
        let mut backoff = options.initial_backoff;
        let mut last_error = Error::StreamClosed;
        for attempt in 1..=options.max_attempts {
            match Self::open(
                &self.http,
                &self.url,
                &self.query,
                &self.creds,
                self.last_event_id.as_deref(),
            )
            .await
            {
                Ok(response) => {
                    self.parser = SseParser::new();
                    self.response = Some(response);
                    tracing::info!(attempt, "event stream reconnected");
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!(attempt, error = %e, "event stream reconnect failed");
                    last_error = e;
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(options.max_backoff);
                }
            }
        }
        Err(last_error)
    }
}

/// Decodes one frame's `data:` payload.
///
/// Both endpoints document the payload as an array of events, but send single
/// objects in practice, so either shape is accepted.
fn decode_frame<T: DeserializeOwned>(data: &str) -> Result<Vec<T>> {
    match serde_json::from_str::<serde_json::Value>(data)? {
        serde_json::Value::Array(items) => items
            .into_iter()
            .map(|item| Ok(serde_json::from_value(item)?))
            .collect(),
        other => Ok(vec![serde_json::from_value(other)?]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Sample {
        n: u32,
    }

    #[test]
    fn frames_decode_from_arrays_and_bare_objects() {
        assert_eq!(
            decode_frame::<Sample>(r#"[{"n":1},{"n":2}]"#).unwrap(),
            vec![Sample { n: 1 }, Sample { n: 2 }]
        );
        assert_eq!(
            decode_frame::<Sample>(r#"{"n":3}"#).unwrap(),
            vec![Sample { n: 3 }]
        );
        assert_eq!(decode_frame::<Sample>("[]").unwrap(), vec![]);
    }

    #[test]
    fn malformed_frames_are_reported_as_json_errors() {
        assert!(matches!(
            decode_frame::<Sample>("not json"),
            Err(Error::Json(_))
        ));
        assert!(matches!(
            decode_frame::<Sample>(r#"[{"n":"one"}]"#),
            Err(Error::Json(_))
        ));
    }

    #[test]
    fn replay_window_rules_are_enforced() {
        assert!(validate_replay_window(true, true, false, false).is_ok());
        assert!(validate_replay_window(false, false, true, true).is_ok());
        assert!(validate_replay_window(false, false, false, false).is_ok());

        assert!(matches!(
            validate_replay_window(false, true, false, false),
            Err(Error::InvalidRequest(_))
        ));
        assert!(matches!(
            validate_replay_window(false, false, false, true),
            Err(Error::InvalidRequest(_))
        ));
        assert!(matches!(
            validate_replay_window(true, false, true, false),
            Err(Error::InvalidRequest(_))
        ));
    }
}
