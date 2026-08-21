//! Fixed-income asset discovery on the Alpaca **Broker API**.
//!
//! This is a different product from the Trading API the rest of the crate
//! covers: it authenticates with HTTP Basic Auth
//! ([`Credentials::BasicAuth`], readable from the environment via
//! [`Credentials::from_broker_env`]) and requires broker partner onboarding
//! with Alpaca. The live base URL is `https://broker-api.alpaca.markets`,
//! the sandbox one is `https://broker-api.sandbox.alpaca.markets`.
//!
//! The client intentionally covers **only** the public fixed-income asset
//! list endpoints — the discovery side of bond trading — not the rest of the
//! broker surface (accounts, KYC, journals, ...). Bond *orders* are
//! broker-only as well and stay out of scope.
//!
//! All dates are `YYYY-MM-DD` ([`NaiveDate`]) and all numerics are plain JSON
//! numbers (`f64`/`bool`), unlike the trading host's string-encoded decimals.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize, Serializer};

use crate::error::{Error, Result};
use crate::rest::{Credentials, RestClient};

const LIVE_BASE: &str = "https://broker-api.alpaca.markets";
const SANDBOX_BASE: &str = "https://broker-api.sandbox.alpaca.markets";

/// Maximum number of CUSIPs/ISINs/tickers accepted per list request;
/// oversized lists are rejected client-side because the API answers with an
/// HTTP 400.
const MAX_IDENTIFIERS: usize = 1_000;

/// Client for the fixed-income asset list endpoints of the Alpaca Broker API
/// (`/v1/assets/fixed_income/*`).
///
/// See the [module documentation](self) for how this differs from the
/// Trading API clients.
pub struct FixedIncomeAssetsClient {
    rest: RestClient,
}

impl FixedIncomeAssetsClient {
    /// Creates a new broker fixed-income client. `sandbox: true` targets the
    /// broker sandbox environment.
    pub fn new(creds: Credentials, sandbox: bool) -> Result<Self> {
        let base = if sandbox { SANDBOX_BASE } else { LIVE_BASE };
        Self::with_base_url(creds, base)
    }

    /// Creates a new client targeting a custom base URL instead of the
    /// default broker endpoints.
    pub fn with_base_url(creds: Credentials, base_url: &str) -> Result<Self> {
        Ok(Self {
            rest: RestClient::new(base_url, creds)?,
        })
    }

    /// `GET /v1/assets/fixed_income/us_treasuries` — lists US treasury
    /// securities, optionally filtered by subtype, status, CUSIPs or ISINs.
    ///
    /// The full matching list is returned in one response; the endpoint does
    /// not paginate. Identifier lists are capped at 1000 entries each
    /// (validated client-side).
    pub async fn get_us_treasuries(
        &self,
        req: &UsTreasuriesRequest,
    ) -> Result<UsTreasuriesResponse> {
        req.validate()?;
        self.rest
            .get("/v1/assets/fixed_income/us_treasuries", req)
            .await
    }

    /// `GET /v1/assets/fixed_income/us_corporates` — lists US corporate
    /// bonds, optionally filtered by status, ISINs, CUSIPs or tickers.
    ///
    /// The full matching list is returned in one response; the endpoint does
    /// not paginate. Identifier lists are capped at 1000 entries each
    /// (validated client-side).
    pub async fn get_us_corporates(
        &self,
        req: &UsCorporatesRequest,
    ) -> Result<UsCorporatesResponse> {
        req.validate()?;
        self.rest
            .get("/v1/assets/fixed_income/us_corporates", req)
            .await
    }
}

/// The subtype of a US treasury security.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TreasurySubtype {
    /// Treasury bond (longest maturity).
    Bond,
    /// Treasury bill (zero-coupon, up to one year).
    Bill,
    /// Treasury note.
    Note,
    /// Separate Trading of Registered Interest and Principal of Securities.
    Strips,
    /// Treasury Inflation-Protected Securities.
    Tips,
    /// Floating-rate note.
    Floating,
    /// A subtype added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
}

/// The lifecycle status of a fixed-income security.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BondStatus {
    /// The bond is issued and outstanding.
    Outstanding,
    /// The bond has matured.
    Matured,
    /// The bond is announced but not yet issued (when-issued).
    PreIssuance,
}

impl BondStatus {
    /// The status as it appears in query parameters (e.g. `"outstanding"`).
    pub fn as_str(&self) -> &str {
        match self {
            Self::Outstanding => "outstanding",
            Self::Matured => "matured",
            Self::PreIssuance => "pre_issuance",
        }
    }
}

/// The coupon type of a fixed-income security.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouponType {
    /// Fixed-rate coupon.
    Fixed,
    /// Floating-rate coupon.
    Floating,
    /// Zero-coupon.
    Zero,
    /// A coupon type added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
}

/// How often a fixed-income security pays its coupon.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouponFrequency {
    /// Once a year.
    Annual,
    /// Twice a year.
    SemiAnnual,
    /// Four times a year.
    Quarterly,
    /// Twelve times a year.
    Monthly,
    /// No coupon payments (zero-coupon securities).
    Zero,
    /// A frequency added by the API after this crate was released.
    #[serde(untagged)]
    Other(String),
}

/// Serializes an identifier list as a single comma-separated query value.
fn join_comma<S: Serializer>(
    values: &[String],
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    serializer.serialize_str(&values.join(","))
}

/// Rejects identifier lists larger than [`MAX_IDENTIFIERS`].
fn validate_identifiers(name: &str, values: &[String]) -> Result<()> {
    if values.len() > MAX_IDENTIFIERS {
        return Err(Error::InvalidRequest(format!(
            "at most {MAX_IDENTIFIERS} {name} per request, got {}",
            values.len()
        )));
    }
    Ok(())
}

/// Query parameters for listing US treasuries
/// ([`FixedIncomeAssetsClient::get_us_treasuries`]). All fields are optional.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UsTreasuriesRequest {
    /// Only treasuries of this subtype.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<TreasurySubtype>,
    /// Only treasuries in this lifecycle status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bond_status: Option<BondStatus>,
    /// Only these CUSIPs (comma-joined on the wire; at most 1000).
    #[serde(skip_serializing_if = "Vec::is_empty", serialize_with = "join_comma")]
    pub cusips: Vec<String>,
    /// Only these ISINs (comma-joined on the wire; at most 1000).
    #[serde(skip_serializing_if = "Vec::is_empty", serialize_with = "join_comma")]
    pub isins: Vec<String>,
}

impl UsTreasuriesRequest {
    /// Validates the client-side limits of the treasuries endpoint.
    pub(crate) fn validate(&self) -> Result<()> {
        validate_identifiers("cusips", &self.cusips)?;
        validate_identifiers("isins", &self.isins)
    }
}

/// Query parameters for listing US corporate bonds
/// ([`FixedIncomeAssetsClient::get_us_corporates`]). All fields are optional.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UsCorporatesRequest {
    /// Only bonds in this lifecycle status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bond_status: Option<BondStatus>,
    /// Only these ISINs (comma-joined on the wire; at most 1000).
    #[serde(skip_serializing_if = "Vec::is_empty", serialize_with = "join_comma")]
    pub isins: Vec<String>,
    /// Only these CUSIPs (comma-joined on the wire; at most 1000).
    #[serde(skip_serializing_if = "Vec::is_empty", serialize_with = "join_comma")]
    pub cusips: Vec<String>,
    /// Only bonds of issuers with these equity tickers (comma-joined on the
    /// wire as the `tickers` parameter; at most 1000).
    #[serde(skip_serializing_if = "Vec::is_empty", serialize_with = "join_comma")]
    pub tickers: Vec<String>,
}

impl UsCorporatesRequest {
    /// Validates the client-side limits of the corporates endpoint.
    pub(crate) fn validate(&self) -> Result<()> {
        validate_identifiers("isins", &self.isins)?;
        validate_identifiers("cusips", &self.cusips)?;
        validate_identifiers("tickers", &self.tickers)
    }
}

/// A US treasury security, as returned by
/// [`FixedIncomeAssetsClient::get_us_treasuries`].
///
/// Mirrors the published schema; the optional fields may be absent depending
/// on the security and where it is in its lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsTreasury {
    /// International Securities Identification Number.
    pub isin: String,
    /// CUSIP identifier.
    pub cusip: String,
    /// Lifecycle status of the security.
    pub bond_status: BondStatus,
    /// Whether the security can be traded.
    pub tradable: bool,
    /// Whether fractional quantities can be traded.
    pub fractionable: bool,
    /// The treasury subtype.
    pub subtype: TreasurySubtype,
    /// Date the security was issued.
    pub issue_date: NaiveDate,
    /// Date the security matures.
    pub maturity_date: NaiveDate,
    /// Full description (e.g. `"United States Treasury 0.0%, 03/27/2025"`).
    pub description: String,
    /// Short description (e.g. `"UST 0.0% 03/27/2025"`).
    pub description_short: String,
    /// Annual coupon rate, in percent.
    pub coupon: f64,
    /// Coupon type.
    pub coupon_type: CouponType,
    /// Coupon payment frequency.
    pub coupon_frequency: CouponFrequency,
    /// Latest closing price, as a percentage of par.
    #[serde(default)]
    pub close_price: Option<f64>,
    /// Date of the latest closing price.
    #[serde(default)]
    pub close_price_date: Option<NaiveDate>,
    /// Yield to maturity at the latest close, in percent.
    #[serde(default)]
    pub close_yield_to_maturity: Option<f64>,
    /// Yield to worst at the latest close, in percent.
    #[serde(default)]
    pub close_yield_to_worst: Option<f64>,
    /// Date of the first coupon payment.
    #[serde(default)]
    pub first_coupon_date: Option<NaiveDate>,
    /// Date of the next coupon payment.
    #[serde(default)]
    pub next_coupon_date: Option<NaiveDate>,
    /// Date of the last coupon payment.
    #[serde(default)]
    pub last_coupon_date: Option<NaiveDate>,
}

/// Response of [`FixedIncomeAssetsClient::get_us_treasuries`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsTreasuriesResponse {
    /// The matching treasury securities.
    pub us_treasuries: Vec<UsTreasury>,
}

/// A US corporate bond, as returned by
/// [`FixedIncomeAssetsClient::get_us_corporates`].
///
/// Mirrors the published schema; optional fields (ratings, liquidity scores,
/// call/reissue details, coupon dates, ...) may be absent depending on the
/// bond.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsCorporate {
    /// International Securities Identification Number.
    pub isin: String,
    /// CUSIP identifier.
    pub cusip: String,
    /// Lifecycle status of the bond.
    pub bond_status: BondStatus,
    /// Whether the bond can be traded.
    pub tradable: bool,
    /// Whether the bond can be traded on margin.
    pub marginable: bool,
    /// Whether fractional quantities can be traded.
    pub fractionable: bool,
    /// Date the bond was issued.
    pub issue_date: NaiveDate,
    /// Country the issuer is domiciled in.
    pub country_domicile: String,
    /// Equity ticker of the issuer.
    pub ticker: String,
    /// Seniority of the bond (e.g. `"senior unsecured"`).
    pub seniority: String,
    /// Name of the issuer.
    pub issuer: String,
    /// Sector of the issuer.
    pub sector: String,
    /// Full description of the bond.
    pub description: String,
    /// Short description of the bond.
    pub description_short: String,
    /// Annual coupon rate, in percent.
    pub coupon: f64,
    /// Coupon type.
    pub coupon_type: CouponType,
    /// Coupon payment frequency.
    pub coupon_frequency: CouponFrequency,
    /// Whether the bond is perpetual (no maturity date).
    pub perpetual: bool,
    /// Day-count convention (e.g. `"30/360"`).
    pub day_count: String,
    /// Date from which interest accrues.
    pub dated_date: NaiveDate,
    /// Total issue size, in USD.
    pub issue_size: f64,
    /// Issue price, as a percentage of par.
    pub issue_price: f64,
    /// Minimum denomination at issue, in USD.
    pub issue_minimum_denomination: f64,
    /// Par (face) value of the bond, in USD.
    pub par_value: f64,
    /// Whether the bond is callable by the issuer.
    pub callable: bool,
    /// Whether the bond is puttable by the holder.
    pub puttable: bool,
    /// Whether the bond is convertible into equity.
    pub convertible: bool,
    /// Whether the bond is a Regulation S security.
    pub reg_s: bool,
    /// Accrued interest, in percent of par.
    #[serde(default)]
    pub accrued_interest: Option<f64>,
    /// Type of the next call (e.g. `"make whole"`).
    #[serde(default)]
    pub call_type: Option<String>,
    /// Latest closing price, as a percentage of par.
    #[serde(default)]
    pub close_price: Option<f64>,
    /// Date of the latest closing price.
    #[serde(default)]
    pub close_price_date: Option<NaiveDate>,
    /// Yield to maturity at the latest close, in percent.
    #[serde(default)]
    pub close_yield_to_maturity: Option<f64>,
    /// Yield to worst at the latest close, in percent.
    #[serde(default)]
    pub close_yield_to_worst: Option<f64>,
    /// Date of the first coupon payment.
    #[serde(default)]
    pub first_coupon_date: Option<NaiveDate>,
    /// Date of the last coupon payment.
    #[serde(default)]
    pub last_coupon_date: Option<NaiveDate>,
    /// Date of the next coupon payment.
    #[serde(default)]
    pub next_coupon_date: Option<NaiveDate>,
    /// Date the bond matures; absent for perpetual bonds.
    #[serde(default)]
    pub maturity_date: Option<NaiveDate>,
    /// Date of the next call.
    #[serde(default)]
    pub next_call_date: Option<NaiveDate>,
    /// Price of the next call, as a percentage of par.
    #[serde(default)]
    pub next_call_price: Option<f64>,
    /// Date of a reissue (reopening) of the bond.
    #[serde(default)]
    pub reissue_date: Option<NaiveDate>,
    /// Price of a reissue, as a percentage of par.
    #[serde(default)]
    pub reissue_price: Option<f64>,
    /// Size of a reissue, in USD.
    #[serde(default)]
    pub reissue_size: Option<f64>,
    /// Aggregate institutional liquidity score.
    #[serde(default)]
    pub liquidity_institutional_aggregate: Option<f64>,
    /// Institutional bid-side liquidity score.
    #[serde(default)]
    pub liquidity_institutional_buy: Option<f64>,
    /// Institutional ask-side liquidity score.
    #[serde(default)]
    pub liquidity_institutional_sell: Option<f64>,
    /// Aggregate micro-lot liquidity score.
    #[serde(default)]
    pub liquidity_micro_aggregate: Option<f64>,
    /// Micro-lot bid-side liquidity score.
    #[serde(default)]
    pub liquidity_micro_buy: Option<f64>,
    /// Micro-lot ask-side liquidity score.
    #[serde(default)]
    pub liquidity_micro_sell: Option<f64>,
    /// Aggregate retail liquidity score.
    #[serde(default)]
    pub liquidity_retail_aggregate: Option<f64>,
    /// Retail bid-side liquidity score.
    #[serde(default)]
    pub liquidity_retail_buy: Option<f64>,
    /// Retail ask-side liquidity score.
    #[serde(default)]
    pub liquidity_retail_sell: Option<f64>,
    /// S&P credit rating (e.g. `"AA+"`).
    #[serde(default)]
    pub sp_rating: Option<String>,
    /// Date of the S&P credit rating.
    #[serde(default)]
    pub sp_rating_date: Option<NaiveDate>,
    /// S&P rating outlook (e.g. `"stable"`).
    #[serde(default)]
    pub sp_outlook: Option<String>,
    /// Date of the S&P rating outlook.
    #[serde(default)]
    pub sp_outlook_date: Option<NaiveDate>,
    /// S&P credit-watch status.
    #[serde(default)]
    pub sp_creditwatch: Option<String>,
    /// Date of the S&P credit-watch status.
    #[serde(default)]
    pub sp_creditwatch_date: Option<NaiveDate>,
}

/// Response of [`FixedIncomeAssetsClient::get_us_corporates`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsCorporatesResponse {
    /// The matching corporate bonds.
    pub us_corporates: Vec<UsCorporate>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The treasury example from the official guide, verbatim.
    const TREASURY_JSON: &str = r#"{
        "us_treasuries": [
            {
                "cusip": "912797MU8",
                "isin": "US912797MU86",
                "bond_status": "outstanding",
                "tradable": true,
                "subtype": "bill",
                "issue_date": "2025-02-13",
                "maturity_date": "2025-03-27",
                "description": "United States Treasury 0.0%, 03/27/2025",
                "description_short": "UST 0.0% 03/27/2025",
                "close_price": 99.6839,
                "close_price_date": "2025-02-27",
                "close_yield_to_maturity": 4.214,
                "close_yield_to_worst": 4.214,
                "coupon": 0,
                "coupon_type": "zero",
                "coupon_frequency": "zero",
                "fractionable": false
            }
        ]
    }"#;

    /// A corporate bond carrying only the fields the schema marks required.
    const CORPORATE_MINIMAL_JSON: &str = r#"{
        "us_corporates": [
            {
                "isin": "US037833DY22",
                "cusip": "037833DY2",
                "bond_status": "outstanding",
                "tradable": true,
                "marginable": false,
                "fractionable": false,
                "issue_date": "2020-08-20",
                "country_domicile": "US",
                "ticker": "AAPL",
                "seniority": "senior unsecured",
                "issuer": "Apple Inc",
                "sector": "Technology",
                "description": "Apple Inc 1.65%, 02/08/2031",
                "description_short": "AAPL 1.65% 02/08/2031",
                "coupon": 1.65,
                "coupon_type": "fixed",
                "coupon_frequency": "semi_annual",
                "perpetual": false,
                "day_count": "30/360",
                "dated_date": "2020-08-20",
                "issue_size": 1500000000,
                "issue_price": 99.9,
                "issue_minimum_denomination": 2000,
                "par_value": 1000,
                "callable": true,
                "puttable": false,
                "convertible": false,
                "reg_s": false
            }
        ]
    }"#;

    #[test]
    fn treasury_fixture_round_trips() {
        let resp: UsTreasuriesResponse = serde_json::from_str(TREASURY_JSON).unwrap();
        assert_eq!(resp.us_treasuries.len(), 1);
        let bill = &resp.us_treasuries[0];
        assert_eq!(bill.cusip, "912797MU8");
        assert_eq!(bill.isin, "US912797MU86");
        assert_eq!(bill.bond_status, BondStatus::Outstanding);
        assert!(bill.tradable);
        assert!(!bill.fractionable);
        assert_eq!(bill.subtype, TreasurySubtype::Bill);
        assert_eq!(
            bill.issue_date,
            NaiveDate::from_ymd_opt(2025, 2, 13).unwrap()
        );
        assert_eq!(
            bill.maturity_date,
            NaiveDate::from_ymd_opt(2025, 3, 27).unwrap()
        );
        assert_eq!(bill.coupon, 0.0);
        assert_eq!(bill.coupon_type, CouponType::Zero);
        assert_eq!(bill.coupon_frequency, CouponFrequency::Zero);
        assert_eq!(bill.close_price, Some(99.6839));
        assert_eq!(bill.close_price_date, NaiveDate::from_ymd_opt(2025, 2, 27));
        assert_eq!(bill.close_yield_to_maturity, Some(4.214));
        assert_eq!(bill.first_coupon_date, None);

        let round_tripped: UsTreasuriesResponse =
            serde_json::from_str(&serde_json::to_string(&resp).unwrap()).unwrap();
        assert_eq!(round_tripped.us_treasuries[0].isin, bill.isin);
    }

    #[test]
    fn corporate_minimal_fixture_parses_with_empty_optionals() {
        let resp: UsCorporatesResponse = serde_json::from_str(CORPORATE_MINIMAL_JSON).unwrap();
        assert_eq!(resp.us_corporates.len(), 1);
        let bond = &resp.us_corporates[0];
        assert_eq!(bond.ticker, "AAPL");
        assert_eq!(bond.bond_status, BondStatus::Outstanding);
        assert_eq!(bond.coupon_type, CouponType::Fixed);
        assert_eq!(bond.coupon_frequency, CouponFrequency::SemiAnnual);
        assert!(bond.callable);
        assert!(!bond.perpetual);
        assert_eq!(bond.par_value, 1000.0);
        // Fields not present in the payload default to `None`.
        assert_eq!(bond.maturity_date, None);
        assert_eq!(bond.sp_rating, None);
        assert_eq!(bond.liquidity_retail_aggregate, None);
        assert_eq!(bond.next_call_price, None);
    }

    #[test]
    fn enums_tolerate_unknown_wire_values() {
        let subtype: TreasurySubtype = serde_json::from_str(r#""savings""#).unwrap();
        assert_eq!(subtype, TreasurySubtype::Other("savings".to_string()));
        let coupon_type: CouponType = serde_json::from_str(r#""step_up""#).unwrap();
        assert_eq!(coupon_type, CouponType::Other("step_up".to_string()));
        let frequency: CouponFrequency = serde_json::from_str(r#""weekly""#).unwrap();
        assert_eq!(frequency, CouponFrequency::Other("weekly".to_string()));
    }

    #[test]
    fn treasuries_request_serializes_filters_and_joined_identifiers() {
        let req = UsTreasuriesRequest {
            subtype: Some(TreasurySubtype::Bill),
            bond_status: Some(BondStatus::Outstanding),
            cusips: vec!["912797MU8".to_string(), "912797KJ5".to_string()],
            isins: vec![],
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["subtype"], "bill");
        assert_eq!(json["bond_status"], "outstanding");
        assert_eq!(json["cusips"], "912797MU8,912797KJ5");
        // Empty identifier lists are omitted entirely.
        assert!(json.get("isins").is_none());

        // The default request serializes to an empty query.
        assert_eq!(
            serde_json::to_value(UsTreasuriesRequest::default()).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn corporates_request_uses_tickers_param_name() {
        let req = UsCorporatesRequest {
            tickers: vec!["AAPL".to_string(), "MSFT".to_string()],
            ..UsCorporatesRequest::default()
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["tickers"], "AAPL,MSFT");
    }

    #[test]
    fn identifier_lists_over_the_cap_are_rejected() {
        let too_many: Vec<String> = (0..=MAX_IDENTIFIERS).map(|i| format!("CUSIP{i}")).collect();
        let treasuries = UsTreasuriesRequest {
            cusips: too_many.clone(),
            ..UsTreasuriesRequest::default()
        };
        assert!(matches!(
            treasuries.validate(),
            Err(Error::InvalidRequest(_))
        ));
        let corporates = UsCorporatesRequest {
            tickers: too_many,
            ..UsCorporatesRequest::default()
        };
        assert!(matches!(
            corporates.validate(),
            Err(Error::InvalidRequest(_))
        ));

        // Exactly at the cap is still accepted.
        let at_cap: Vec<String> = (0..MAX_IDENTIFIERS).map(|i| format!("CUSIP{i}")).collect();
        assert!(
            UsTreasuriesRequest {
                cusips: at_cap,
                ..UsTreasuriesRequest::default()
            }
            .validate()
            .is_ok()
        );
    }
}
