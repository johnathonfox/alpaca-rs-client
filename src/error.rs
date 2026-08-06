//! Crate-wide error type and result alias.

/// Errors returned by this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// HTTP transport error from `reqwest`.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    /// The API returned a non-success status code.
    #[error("api error (status {status}): {message}")]
    Api {
        /// HTTP status code returned by the API.
        status: u16,
        /// Response body returned by the API.
        message: String,
    },
    /// JSON (de)serialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// WebSocket protocol or transport error.
    #[error("websocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    /// URL parse error.
    #[error("url error: {0}")]
    Url(#[from] url::ParseError),
    /// Credentials were not provided and could not be read from the environment.
    #[error(
        "missing credentials: set the APCA_API_KEY_ID and APCA_API_SECRET_KEY environment variables"
    )]
    MissingCredentials,
    /// The stream was closed unexpectedly.
    #[error("stream closed")]
    StreamClosed,
    /// Protocol-level surprise on a WebSocket stream (e.g. an `error` control
    /// message or an unexpected handshake response).
    #[error("stream protocol error: {0}")]
    Stream(String),
    /// An invalid [`crate::data::enums::TimeFrame`] amount/unit combination.
    #[error("invalid timeframe: {0}")]
    InvalidTimeFrame(String),
    /// A request failed client-side validation (e.g. malformed multi-leg
    /// order).
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

/// Convenience alias for results returned by this crate.
pub type Result<T> = std::result::Result<T, Error>;
