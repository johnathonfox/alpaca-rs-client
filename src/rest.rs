//! Credentials and the shared low-level REST client used by all API clients.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use url::Url;

use crate::error::{Error, Result};

/// Credentials used to authenticate with the Alpaca API.
#[derive(Debug, Clone)]
pub enum Credentials {
    /// API key pair from the Alpaca dashboard.
    Key {
        /// The API key id (`APCA_API_KEY_ID`).
        key_id: String,
        /// The API secret key (`APCA_API_SECRET_KEY`).
        secret_key: String,
    },
    /// An OAuth access token.
    OAuth(String),
}

impl Credentials {
    /// Reads credentials from the `APCA_API_KEY_ID` and `APCA_API_SECRET_KEY`
    /// environment variables.
    ///
    /// Returns [`Error::MissingCredentials`] if either variable is unset or
    /// empty.
    pub fn from_env() -> Result<Self> {
        let key_id = std::env::var("APCA_API_KEY_ID").ok();
        let secret_key = std::env::var("APCA_API_SECRET_KEY").ok();
        match (key_id, secret_key) {
            (Some(key_id), Some(secret_key)) if !key_id.is_empty() && !secret_key.is_empty() => {
                Ok(Self::Key { key_id, secret_key })
            }
            _ => Err(Error::MissingCredentials),
        }
    }

    /// Returns the `(key, secret)` pair used by the WebSocket `auth` action.
    ///
    /// For OAuth credentials the access token is sent as the key with an
    /// empty secret, matching the behaviour of the Python SDK.
    pub(crate) fn key_secret(&self) -> (&str, &str) {
        match self {
            Credentials::Key { key_id, secret_key } => (key_id, secret_key),
            Credentials::OAuth(token) => (token, ""),
        }
    }

    /// Adds the authentication headers for these credentials to an HTTP
    /// request: the `APCA-API-*` pair for key credentials, a bearer token for
    /// OAuth ones.
    pub(crate) fn authenticate(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self {
            Credentials::Key { key_id, secret_key } => req
                .header("APCA-API-KEY-ID", key_id)
                .header("APCA-API-SECRET-KEY", secret_key),
            Credentials::OAuth(token) => req.bearer_auth(token),
        }
    }
}

/// Percent-encodes a single URL path segment (e.g. symbols such as
/// `BTC/USD`).
pub(crate) fn encode_segment(segment: &str) -> String {
    url::form_urlencoded::byte_serialize(segment.as_bytes()).collect()
}

/// A response that can span multiple pages, implemented by the paginated
/// market-data response wrappers. Used by [`RestClient::get_paginated`].
pub(crate) trait PaginatedResponse: DeserializeOwned {
    /// Token for the next page; `None` (or an empty token) ends pagination.
    fn next_page_token(&self) -> Option<&str>;
    /// Merges the next page of results into this one, taking over its
    /// `next_page_token`.
    fn merge(&mut self, next: Self);
}

/// Default page size for bars, trades and quotes, mirroring alpaca-py.
pub(crate) const PAGE_LIMIT_BARS_TRADES_QUOTES: u32 = 10_000;
/// Default page size for option snapshots, mirroring alpaca-py.
pub(crate) const PAGE_LIMIT_OPTIONS_SNAPSHOTS: u32 = 1_000;
/// Default page size for news, mirroring alpaca-py.
pub(crate) const PAGE_LIMIT_NEWS: u32 = 50;
/// Default page size for corporate actions, mirroring alpaca-py.
pub(crate) const PAGE_LIMIT_CORPORATE_ACTIONS: u32 = 1_000;

/// Sets `limit` on a serialized query object unless the caller supplied one.
fn apply_default_limit(query: &mut Value, default_limit: Option<u32>) {
    if let Some(limit) = default_limit
        && query.get("limit").is_none()
    {
        query["limit"] = limit.into();
    }
}

/// Returns a copy of the serialized query object with `page_token` injected.
fn query_with_page_token(query: &Value, page_token: Option<&str>) -> Value {
    let mut query = query.clone();
    if let Some(token) = page_token {
        // Indexing a `Value` with a string turns `Null` into an object, so
        // this also works for parameterless queries.
        query["page_token"] = token.into();
    }
    query
}

/// Drives the pagination loop: fetches the first page, then keeps fetching
/// and merging while the merged result has a non-empty next page token.
async fn paginate<'f, T, F>(mut fetch: F) -> Result<T>
where
    T: PaginatedResponse,
    F: FnMut(Option<String>) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'f>>,
{
    let mut merged = fetch(None).await?;
    while let Some(token) = merged
        .next_page_token()
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
    {
        merged.merge(fetch(Some(token)).await?);
    }
    Ok(merged)
}

/// Low-level shared REST client. Handles authentication headers, URL joining
/// and error mapping; the individual API clients build on top of it.
///
/// Requests are retried on transient failures: HTTP 429 and 5xx responses as
/// well as connect/timeout transport errors. The policy mirrors alpaca-py's
/// default: up to [`MAX_ATTEMPTS`] total attempts with a backoff that doubles
/// each time (1s, 2s, ...).
pub(crate) struct RestClient {
    http: reqwest::Client,
    base: Url,
    creds: Credentials,
}

/// Total number of attempts for a request (1 initial + 2 retries), mirroring
/// alpaca-py's default.
const MAX_ATTEMPTS: u32 = 3;

/// Delay before the first retry; doubles on each subsequent attempt.
const INITIAL_BACKOFF: Duration = Duration::from_secs(1);

/// Whether an HTTP status is transient and worth retrying: rate limiting
/// (429) or a server-side error (5xx).
fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

/// Whether a transport error is transient and worth retrying (connection
/// failures and timeouts).
fn is_retryable_error(err: &reqwest::Error) -> bool {
    err.is_connect() || err.is_timeout()
}

/// Delay before retrying after failed attempt `attempt` (1-based): 1s after
/// the first attempt, 2s after the second, and so on.
fn backoff(attempt: u32) -> Duration {
    INITIAL_BACKOFF * 2u32.pow(attempt.saturating_sub(1))
}

/// Delay before the next attempt, or `None` once the attempt budget is
/// exhausted.
fn retry_delay(attempt: u32) -> Option<Duration> {
    (attempt < MAX_ATTEMPTS).then(|| backoff(attempt))
}

impl RestClient {
    pub(crate) fn new(base: &str, creds: Credentials) -> Result<Self> {
        Ok(Self {
            http: reqwest::Client::new(),
            base: Url::parse(base)?,
            creds,
        })
    }

    fn url(&self, path: &str) -> Result<Url> {
        Ok(self.base.join(path)?)
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        self.creds.authenticate(req)
    }

    async fn send<T: DeserializeOwned>(&self, req: reqwest::RequestBuilder) -> Result<T> {
        Self::decode(self.send_raw(req).await?).await
    }

    /// Authenticates and sends a request, retrying transient failures, and
    /// hands back the final response without interpreting its body. Used by
    /// [`send`](Self::send) and by the endpoints that return something other
    /// than JSON.
    async fn send_raw(&self, req: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let req = self.auth(req);
        let mut attempt = 1;
        loop {
            // All bodies are in-memory (JSON or query), so cloning always
            // succeeds; if it somehow doesn't, fall back to a single try.
            let Some(cloned) = req.try_clone() else {
                return Ok(req.send().await?);
            };
            match cloned.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    match retry_delay(attempt).filter(|_| is_retryable_status(status)) {
                        Some(delay) => {
                            tracing::warn!(
                                attempt,
                                status = status.as_u16(),
                                "transient response, retrying in {delay:?}"
                            );
                            tokio::time::sleep(delay).await;
                            attempt += 1;
                        }
                        None => return Ok(resp),
                    }
                }
                Err(err) => match retry_delay(attempt).filter(|_| is_retryable_error(&err)) {
                    Some(delay) => {
                        tracing::warn!(
                            attempt,
                            error = %err,
                            "transient error, retrying in {delay:?}"
                        );
                        tokio::time::sleep(delay).await;
                        attempt += 1;
                    }
                    None => return Err(err.into()),
                },
            }
        }
    }

    /// Maps a response onto the expected type: non-2xx becomes
    /// [`Error::Api`], an empty body deserializes from `null` (204 No
    /// Content), anything else is parsed as JSON.
    async fn decode<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(Error::Api {
                status: status.as_u16(),
                message: body,
            });
        }
        if body.is_empty() {
            // Endpoints that return 204 No Content; T is usually `()`.
            return Ok(serde_json::from_value(serde_json::Value::Null)?);
        }
        Ok(serde_json::from_str(&body)?)
    }

    /// Issues a GET request with a serialized query string.
    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &(impl Serialize + ?Sized),
    ) -> Result<T> {
        let req = self.http.get(self.url(path)?).query(query);
        self.send(req).await
    }

    /// Issues a GET request and follows `next_page_token`, returning all
    /// pages merged into a single response (mirroring alpaca-py). An explicit
    /// `page_token` on the query is honored for the first request and
    /// auto-followed from there. If `default_limit` is set it is used as the
    /// page size unless the query already carries a `limit`.
    pub(crate) async fn get_paginated<T: PaginatedResponse>(
        &self,
        path: &str,
        query: &(impl Serialize + ?Sized),
        default_limit: Option<u32>,
    ) -> Result<T> {
        let mut query = serde_json::to_value(query)?;
        apply_default_limit(&mut query, default_limit);
        paginate(|page_token| {
            let query = query_with_page_token(&query, page_token.as_deref());
            Box::pin(async move { self.get(path, &query).await })
        })
        .await
    }

    /// Issues a GET request and returns the raw response body together with
    /// its `Content-Type` header, for the endpoints that answer with
    /// something other than JSON (e.g. logo images).
    pub(crate) async fn get_bytes(
        &self,
        path: &str,
        query: &(impl Serialize + ?Sized),
    ) -> Result<(Vec<u8>, Option<String>)> {
        let req = self.http.get(self.url(path)?).query(query);
        let resp = self.send_raw(req).await?;
        let status = resp.status();
        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let body = resp.bytes().await?;
        if !status.is_success() {
            return Err(Error::Api {
                status: status.as_u16(),
                message: String::from_utf8_lossy(&body).into_owned(),
            });
        }
        Ok((body.to_vec(), content_type))
    }

    /// Issues a POST request with a JSON body.
    pub(crate) async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &(impl Serialize + ?Sized),
    ) -> Result<T> {
        self.post_with_query(path, &(), body).await
    }

    /// Issues a POST request with both a serialized query string and a JSON
    /// body, as the `:by_name` watchlist endpoints require.
    pub(crate) async fn post_with_query<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &(impl Serialize + ?Sized),
        body: &(impl Serialize + ?Sized),
    ) -> Result<T> {
        let req = self.http.post(self.url(path)?).query(query).json(body);
        self.send(req).await
    }

    /// Issues a PATCH request with a JSON body.
    pub(crate) async fn patch<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &(impl Serialize + ?Sized),
    ) -> Result<T> {
        let req = self.http.patch(self.url(path)?).json(body);
        self.send(req).await
    }

    /// Issues a PUT request with a JSON body.
    pub(crate) async fn put<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &(impl Serialize + ?Sized),
    ) -> Result<T> {
        self.put_with_query(path, &(), body).await
    }

    /// Issues a PUT request with both a serialized query string and a JSON
    /// body, as the `:by_name` watchlist endpoints require.
    pub(crate) async fn put_with_query<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &(impl Serialize + ?Sized),
        body: &(impl Serialize + ?Sized),
    ) -> Result<T> {
        let req = self.http.put(self.url(path)?).query(query).json(body);
        self.send(req).await
    }

    /// Issues a DELETE request with a serialized query string.
    pub(crate) async fn delete<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &(impl Serialize + ?Sized),
    ) -> Result<T> {
        let req = self.http.delete(self.url(path)?).query(query);
        self.send(req).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn credentials_from_env() {
        // SAFETY: this is the only test in the crate that touches these
        // variables, so there is no data race with other tests.
        unsafe {
            std::env::remove_var("APCA_API_KEY_ID");
            std::env::remove_var("APCA_API_SECRET_KEY");
        }
        assert!(matches!(
            Credentials::from_env(),
            Err(Error::MissingCredentials)
        ));

        unsafe {
            std::env::set_var("APCA_API_KEY_ID", "test-key");
            std::env::set_var("APCA_API_SECRET_KEY", "test-secret");
        }
        match Credentials::from_env() {
            Ok(Credentials::Key { key_id, secret_key }) => {
                assert_eq!(key_id, "test-key");
                assert_eq!(secret_key, "test-secret");
            }
            other => panic!("expected key credentials, got {other:?}"),
        }

        unsafe {
            std::env::remove_var("APCA_API_KEY_ID");
            std::env::remove_var("APCA_API_SECRET_KEY");
        }
        assert!(matches!(
            Credentials::from_env(),
            Err(Error::MissingCredentials)
        ));
    }

    #[test]
    fn encode_segment_escapes_slashes() {
        assert_eq!(encode_segment("AAPL"), "AAPL");
        assert_eq!(encode_segment("BTC/USD"), "BTC%2FUSD");
    }

    #[derive(Debug, Deserialize)]
    struct FakePage {
        items: Vec<u32>,
        next_page_token: Option<String>,
    }

    impl PaginatedResponse for FakePage {
        fn next_page_token(&self) -> Option<&str> {
            self.next_page_token.as_deref()
        }

        fn merge(&mut self, next: Self) {
            self.items.extend(next.items);
            self.next_page_token = next.next_page_token;
        }
    }

    fn page(items: &[u32], next_page_token: Option<&str>) -> FakePage {
        FakePage {
            items: items.to_vec(),
            next_page_token: next_page_token.map(str::to_owned),
        }
    }

    /// Runs `paginate` against a canned sequence of pages, returning the
    /// merged page plus the tokens the fetch closure was called with.
    async fn paginate_canned(pages: Vec<FakePage>) -> (Result<FakePage>, Vec<Option<String>>) {
        use std::cell::RefCell;
        use std::collections::VecDeque;

        let pages = RefCell::new(VecDeque::from(pages));
        let calls = RefCell::new(Vec::new());
        let result = paginate(|token| {
            calls.borrow_mut().push(token);
            let page = pages.borrow_mut().pop_front();
            Box::pin(
                async move { page.ok_or_else(|| Error::Stream("test ran out of pages".into())) },
            )
        })
        .await;
        (result, calls.into_inner())
    }

    #[tokio::test]
    async fn paginate_merges_pages_until_token_is_none() {
        let pages = vec![
            page(&[1, 2], Some("t1")),
            page(&[3], Some("t2")),
            page(&[4, 5], None),
        ];
        let (result, calls) = paginate_canned(pages).await;
        let merged = result.unwrap();
        assert_eq!(merged.items, vec![1, 2, 3, 4, 5]);
        assert_eq!(merged.next_page_token, None);
        assert_eq!(
            calls,
            vec![None, Some("t1".to_string()), Some("t2".to_string())]
        );
    }

    #[tokio::test]
    async fn paginate_stops_on_empty_token() {
        let pages = vec![page(&[1], Some("")), page(&[2], None)];
        let (result, calls) = paginate_canned(pages).await;
        assert_eq!(result.unwrap().items, vec![1]);
        assert_eq!(calls, vec![None]);
    }

    #[tokio::test]
    async fn paginate_propagates_fetch_errors() {
        let (result, calls) = paginate_canned(vec![]).await;
        assert!(matches!(result, Err(Error::Stream(_))));
        assert_eq!(calls, vec![None]);
    }

    #[test]
    fn default_limit_is_applied_only_when_missing() {
        let mut query = serde_json::json!({"symbols": "AAPL"});
        apply_default_limit(&mut query, Some(10_000));
        assert_eq!(query["limit"], 10_000);

        let mut query = serde_json::json!({"limit": 5});
        apply_default_limit(&mut query, Some(10_000));
        assert_eq!(query["limit"], 5);

        let mut query = serde_json::json!({});
        apply_default_limit(&mut query, None);
        assert_eq!(query, serde_json::json!({}));
    }

    #[test]
    fn page_token_is_injected_into_query() {
        let query = serde_json::json!({"symbols": "AAPL"});
        let with_token = query_with_page_token(&query, Some("tok"));
        assert_eq!(with_token["page_token"], "tok");
        assert_eq!(with_token["symbols"], "AAPL");
        // `None` leaves the query untouched.
        assert_eq!(query_with_page_token(&query, None), query);
        // Works for parameterless queries serialized as `null`.
        let with_token = query_with_page_token(&Value::Null, Some("tok"));
        assert_eq!(with_token["page_token"], "tok");
    }

    #[test]
    fn retryable_statuses_are_429_and_5xx() {
        use reqwest::StatusCode;
        assert!(is_retryable_status(StatusCode::TOO_MANY_REQUESTS));
        assert!(is_retryable_status(StatusCode::INTERNAL_SERVER_ERROR));
        assert!(is_retryable_status(StatusCode::BAD_GATEWAY));
        assert!(is_retryable_status(StatusCode::SERVICE_UNAVAILABLE));
        assert!(is_retryable_status(StatusCode::GATEWAY_TIMEOUT));

        assert!(!is_retryable_status(StatusCode::OK));
        assert!(!is_retryable_status(StatusCode::BAD_REQUEST));
        assert!(!is_retryable_status(StatusCode::UNAUTHORIZED));
        assert!(!is_retryable_status(StatusCode::FORBIDDEN));
        assert!(!is_retryable_status(StatusCode::NOT_FOUND));
        assert!(!is_retryable_status(StatusCode::UNPROCESSABLE_ENTITY));
    }

    #[test]
    fn backoff_doubles_each_attempt() {
        assert_eq!(backoff(1), Duration::from_secs(1));
        assert_eq!(backoff(2), Duration::from_secs(2));
        assert_eq!(backoff(3), Duration::from_secs(4));
    }

    #[test]
    fn retry_delay_stops_after_max_attempts() {
        // Three total attempts: retries happen after attempts 1 and 2 only.
        assert_eq!(retry_delay(1), Some(Duration::from_secs(1)));
        assert_eq!(retry_delay(2), Some(Duration::from_secs(2)));
        assert_eq!(retry_delay(MAX_ATTEMPTS), None);
        assert_eq!(retry_delay(MAX_ATTEMPTS + 1), None);
    }
}
