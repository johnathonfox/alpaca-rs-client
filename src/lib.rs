//! # alpaca-rs
//!
//! An async Rust client library for the [Alpaca](https://alpaca.markets)
//! markets API, mirroring the Python SDK (`alpaca-py`).
//!
//! - [`trading`]: account, orders, positions, assets, watchlists, corporate
//!   actions and option contracts (`TradingClient`).
//! - [`data`]: historical and latest market data for stocks, crypto, options
//!   and news, plus the screener and corporate actions endpoints.
//! - [`stream`]: WebSocket streams for real-time market data and trading
//!   updates.
//! - [`rest`]: credentials handling ([`rest::Credentials`]) shared by all
//!   clients.
//!
//! All clients are async (tokio) and return [`Result`] with the crate
//! [`Error`] type.

pub mod data;
pub mod error;
pub mod rest;
pub mod stream;
pub mod trading;

pub use error::{Error, Result};
