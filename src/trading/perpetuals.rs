//! Crypto perpetual futures endpoints (`/v2/perpetuals/*`).
//!
//! Beta, and a non-US offering. The wallet/transfer/whitelist endpoints
//! mirror the spot crypto funding surface of
//! [`wallets`](super::wallets) under `/v2/perpetuals/wallets` and reuse its
//! models. Perp *orders* go through the standard `POST /v2/orders` with
//! [`AssetClass::CryptoPerp`](super::AssetClass::CryptoPerp). None of these
//! endpoints paginate.

use serde::Serialize;

use crate::error::Result;
use crate::rest::encode_segment;

use super::wallets::{CryptoWalletsWire, WhitelistedAddressWire};
use super::{
    CreatePerpTransferRequest, CreatePerpWhitelistedAddressRequest, CryptoTransfer, CryptoWallet,
    GetWalletsRequest, PerpAccountVitals, PerpLeverage, TradingClient, TransferFeeEstimate,
    TransferFeeEstimateRequest, WhitelistedAddress,
};

/// Query parameters of the leverage endpoints (`symbol` for the GET,
/// `symbol` + `leverage` for the bodyless POST).
#[derive(Serialize)]
struct PerpLeverageQuery<'a> {
    symbol: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    leverage: Option<u32>,
}

impl TradingClient {
    /// `GET /v2/perpetuals/wallets` — lists the perpetuals account's crypto
    /// wallets (optionally filtered by asset).
    ///
    /// Same single-object-vs-array quirk as
    /// [`get_wallets`](Self::get_wallets): with `asset` set the endpoint
    /// answers with one wallet object; this method always returns a vector.
    pub async fn get_perp_wallets(&self, asset: Option<String>) -> Result<Vec<CryptoWallet>> {
        let wire: CryptoWalletsWire = self
            .rest
            .get(
                "/v2/perpetuals/wallets",
                &GetWalletsRequest { asset, chain: None },
            )
            .await?;
        Ok(wire.into_vec())
    }

    /// `GET /v2/perpetuals/wallets/transfers` — lists the perpetuals
    /// account's crypto transfers.
    pub async fn get_perp_wallet_transfers(&self) -> Result<Vec<CryptoTransfer>> {
        self.rest.get("/v2/perpetuals/wallets/transfers", &()).await
    }

    /// `GET /v2/perpetuals/wallets/transfers/{transfer_id}` — returns a single
    /// perpetuals wallet transfer.
    pub async fn get_perp_wallet_transfer(&self, transfer_id: &str) -> Result<CryptoTransfer> {
        self.rest
            .get(
                &format!(
                    "/v2/perpetuals/wallets/transfers/{}",
                    encode_segment(transfer_id)
                ),
                &(),
            )
            .await
    }

    /// `POST /v2/perpetuals/wallets/transfers` — creates a perpetuals wallet
    /// transfer. (Unlike the spot equivalent, this endpoint is not
    /// deprecated.)
    pub async fn create_perp_wallet_transfer(
        &self,
        req: &CreatePerpTransferRequest,
    ) -> Result<CryptoTransfer> {
        self.rest
            .post("/v2/perpetuals/wallets/transfers", req)
            .await
    }

    /// `GET /v2/perpetuals/wallets/whitelists` — lists whitelisted
    /// perpetuals withdrawal addresses.
    pub async fn get_perp_whitelisted_addresses(&self) -> Result<Vec<WhitelistedAddress>> {
        self.rest
            .get("/v2/perpetuals/wallets/whitelists", &())
            .await
    }

    /// `POST /v2/perpetuals/wallets/whitelists` — whitelists a perpetuals
    /// withdrawal address. Accepts both response shapes documented for the
    /// spot equivalent (single object or array-wrapped).
    pub async fn create_perp_whitelisted_address(
        &self,
        req: &CreatePerpWhitelistedAddressRequest,
    ) -> Result<WhitelistedAddress> {
        let wire: WhitelistedAddressWire = self
            .rest
            .post("/v2/perpetuals/wallets/whitelists", req)
            .await?;
        wire.into_one()
    }

    /// `DELETE /v2/perpetuals/wallets/whitelists/{whitelist_id}` — removes a
    /// whitelisted perpetuals address. The endpoint answers 200 with an empty
    /// body.
    pub async fn delete_perp_whitelisted_address(&self, whitelist_id: &str) -> Result<()> {
        self.rest
            .delete(
                &format!(
                    "/v2/perpetuals/wallets/whitelists/{}",
                    encode_segment(whitelist_id)
                ),
                &(),
            )
            .await
    }

    /// `GET /v2/perpetuals/wallets/fees/estimate` — estimates the fee of a
    /// perpetuals transfer. Unlike the spot endpoint, the response carries
    /// only `fee` (no `network_fee`).
    pub async fn get_perp_transfer_fee_estimate(
        &self,
        req: &TransferFeeEstimateRequest,
    ) -> Result<TransferFeeEstimate> {
        self.rest
            .get("/v2/perpetuals/wallets/fees/estimate", req)
            .await
    }

    /// `GET /v2/perpetuals/leverage` — returns the leverage in effect for a
    /// perpetual symbol.
    pub async fn get_perp_leverage(&self, symbol: &str) -> Result<PerpLeverage> {
        self.rest
            .get(
                "/v2/perpetuals/leverage",
                &PerpLeverageQuery {
                    symbol,
                    leverage: None,
                },
            )
            .await
    }

    /// `POST /v2/perpetuals/leverage` — sets the leverage of a perpetual
    /// symbol. The API takes `symbol` and `leverage` as query parameters and
    /// no request body.
    pub async fn set_perp_leverage(&self, symbol: &str, leverage: u32) -> Result<PerpLeverage> {
        self.rest
            .post_query(
                "/v2/perpetuals/leverage",
                &PerpLeverageQuery {
                    symbol,
                    leverage: Some(leverage),
                },
            )
            .await
    }

    /// `GET /v2/perpetuals/account_vitals` — returns margin and collateral
    /// vitals of the perpetuals account.
    pub async fn get_perp_account_vitals(&self) -> Result<PerpAccountVitals> {
        self.rest.get("/v2/perpetuals/account_vitals", &()).await
    }
}
