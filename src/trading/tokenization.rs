//! Tokenization endpoints (`/v2/tokenization/*`).
//!
//! Minting and redeeming tokenized assets requires the account to be enabled
//! as an Authorized Participant; paper trading is supported. None of these
//! endpoints paginate.

use serde::Serialize;

use crate::error::Result;
use crate::rest::encode_segment;

use super::{
    GetTokenizationRequestsRequest, TokenizationMintRequest, TokenizationMintResponse,
    TokenizationRequest, TradingClient,
};

/// Query parameters of the `:by_client_request_id` lookup.
#[derive(Serialize)]
struct ClientRequestIdQuery<'a> {
    client_request_id: &'a str,
}

impl TradingClient {
    /// `POST /v2/tokenization/mint` — requests the minting of a tokenized
    /// asset.
    ///
    /// Pass an `idempotency_key` to make retries safe: the API deduplicates
    /// mints on the `Idempotency-Key` header.
    pub async fn mint_tokenized_asset(
        &self,
        req: &TokenizationMintRequest,
        idempotency_key: Option<&str>,
    ) -> Result<TokenizationMintResponse> {
        self.rest
            .post_with_idempotency_key("/v2/tokenization/mint", req, idempotency_key)
            .await
    }

    /// `GET /v2/tokenization/requests` — lists tokenization requests, newest
    /// first.
    pub async fn get_tokenization_requests(
        &self,
        req: &GetTokenizationRequestsRequest,
    ) -> Result<Vec<TokenizationRequest>> {
        self.rest.get("/v2/tokenization/requests", req).await
    }

    /// `GET /v2/tokenization/requests/{tokenization_request_id}` — returns a
    /// single tokenization request.
    pub async fn get_tokenization_request(
        &self,
        tokenization_request_id: &str,
    ) -> Result<TokenizationRequest> {
        self.rest
            .get(
                &format!(
                    "/v2/tokenization/requests/{}",
                    encode_segment(tokenization_request_id)
                ),
                &(),
            )
            .await
    }

    /// `GET /v2/tokenization/requests:by_client_request_id` — returns the
    /// tokenization request created with the given client request id.
    ///
    /// The colon in the path is intentional (a custom method segment, as in
    /// [`get_order_by_client_order_id`](Self::get_order_by_client_order_id))
    /// and must not be percent-encoded; the query value is encoded normally.
    pub async fn get_tokenization_request_by_client_request_id(
        &self,
        client_request_id: &str,
    ) -> Result<TokenizationRequest> {
        self.rest
            .get(
                "/v2/tokenization/requests:by_client_request_id",
                &ClientRequestIdQuery { client_request_id },
            )
            .await
    }
}
