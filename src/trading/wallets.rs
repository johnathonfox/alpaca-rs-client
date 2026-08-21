//! Crypto wallets and funding endpoints (`/v2/wallets*`).
//!
//! These endpoints are GA but gated: crypto funding must be enabled on the
//! account by Alpaca first. Paper trading is supported. None of them
//! paginate.

use serde::Deserialize;

use crate::error::{Error, Result};
use crate::rest::encode_segment;

use super::{
    CreateWalletTransferRequest, CreateWhitelistedAddressRequest, CryptoTransfer, CryptoWallet,
    GetWalletsRequest, TradingClient, TransferFeeEstimate, TransferFeeEstimateRequest,
    WhitelistedAddress,
};

/// Wire shape of `GET /v2/wallets` (and the perpetuals equivalent): a single
/// wallet object when the `asset` filter is given, an array otherwise. This
/// untagged enum absorbs both shapes.
#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum CryptoWalletsWire {
    /// A single wallet (asset-filtered response).
    One(CryptoWallet),
    /// A list of wallets.
    Many(Vec<CryptoWallet>),
}

impl CryptoWalletsWire {
    /// Normalizes both wire shapes into a list.
    pub(crate) fn into_vec(self) -> Vec<CryptoWallet> {
        match self {
            Self::One(wallet) => vec![wallet],
            Self::Many(wallets) => wallets,
        }
    }
}

/// Wire shape of the create-whitelist response: the guide shows it wrapped in
/// an array, the schema says a single object. Accept either.
#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum WhitelistedAddressWire {
    /// A single whitelist entry (schema shape).
    One(WhitelistedAddress),
    /// An array-wrapped whitelist entry (guide shape).
    Many(Vec<WhitelistedAddress>),
}

impl WhitelistedAddressWire {
    /// Normalizes both wire shapes into a single entry.
    pub(crate) fn into_one(self) -> Result<WhitelistedAddress> {
        match self {
            Self::One(address) => Ok(address),
            Self::Many(mut addresses) if !addresses.is_empty() => Ok(addresses.swap_remove(0)),
            Self::Many(_) => Err(Error::Json(
                <serde_json::Error as serde::de::Error>::custom("empty whitelists response"),
            )),
        }
    }
}

impl TradingClient {
    /// `GET /v2/wallets` — lists the account's crypto wallets.
    ///
    /// Quirk: the endpoint answers with a single wallet object when
    /// [`GetWalletsRequest::asset`] is set and with an array otherwise; this
    /// method always returns a vector.
    pub async fn get_wallets(&self, req: &GetWalletsRequest) -> Result<Vec<CryptoWallet>> {
        let wire: CryptoWalletsWire = self.rest.get("/v2/wallets", req).await?;
        Ok(wire.into_vec())
    }

    /// `GET /v2/wallets/transfers` — lists the account's crypto transfers.
    pub async fn get_wallet_transfers(&self) -> Result<Vec<CryptoTransfer>> {
        self.rest.get("/v2/wallets/transfers", &()).await
    }

    /// `GET /v2/wallets/transfers/{transfer_id}` — returns a single crypto
    /// transfer.
    pub async fn get_wallet_transfer(&self, transfer_id: &str) -> Result<CryptoTransfer> {
        self.rest
            .get(
                &format!("/v2/wallets/transfers/{}", encode_segment(transfer_id)),
                &(),
            )
            .await
    }

    /// `POST /v2/wallets/transfers` — creates a crypto withdrawal.
    ///
    /// Deprecated: the API sunsets this endpoint on 2026-10-09; withdrawals
    /// are moving to the web app.
    pub async fn create_wallet_transfer(
        &self,
        req: &CreateWalletTransferRequest,
    ) -> Result<CryptoTransfer> {
        self.rest.post("/v2/wallets/transfers", req).await
    }

    /// `GET /v2/wallets/whitelists` — lists whitelisted withdrawal addresses.
    pub async fn get_whitelisted_addresses(&self) -> Result<Vec<WhitelistedAddress>> {
        self.rest.get("/v2/wallets/whitelists", &()).await
    }

    /// `POST /v2/wallets/whitelists` — whitelists a withdrawal address.
    ///
    /// Quirk: the guide shows the response array-wrapped while the schema
    /// declares a single object; both shapes are accepted.
    pub async fn create_whitelisted_address(
        &self,
        req: &CreateWhitelistedAddressRequest,
    ) -> Result<WhitelistedAddress> {
        let wire: WhitelistedAddressWire = self.rest.post("/v2/wallets/whitelists", req).await?;
        wire.into_one()
    }

    /// `DELETE /v2/wallets/whitelists/{whitelist_id}` — removes a whitelisted
    /// address. The endpoint answers 200 with an empty body.
    pub async fn delete_whitelisted_address(&self, whitelist_id: &str) -> Result<()> {
        self.rest
            .delete(
                &format!("/v2/wallets/whitelists/{}", encode_segment(whitelist_id)),
                &(),
            )
            .await
    }

    /// `GET /v2/wallets/fees/estimate` — estimates the fee of a transfer.
    pub async fn get_transfer_fee_estimate(
        &self,
        req: &TransferFeeEstimateRequest,
    ) -> Result<TransferFeeEstimate> {
        self.rest.get("/v2/wallets/fees/estimate", req).await
    }
}
