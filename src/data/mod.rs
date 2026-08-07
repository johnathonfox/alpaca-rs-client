//! Market data API: enums, models, request parameters and the historical
//! data clients. Base URL for all data clients is
//! `https://data.alpaca.markets`.

pub mod corporate_actions;
pub mod crypto;
pub mod enums;
pub mod forex;
pub mod logos;
pub mod models;
pub mod news;
pub mod option;
pub mod requests;
pub mod screener;
pub mod stock;

pub use corporate_actions::CorporateActionsClient;
pub use crypto::CryptoHistoricalDataClient;
pub use enums::*;
pub use forex::ForexClient;
pub use logos::LogoClient;
pub use models::*;
pub use news::NewsClient;
pub use option::{OptionHistoricalDataClient, OptionSnapshotsRequest};
pub use requests::*;
pub use screener::ScreenerClient;
pub use stock::StockHistoricalDataClient;
