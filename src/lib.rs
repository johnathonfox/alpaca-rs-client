//! # alpaca-rs
//!
//! An async Rust client library for the [Alpaca](https://alpaca.markets)
//! markets API, mirroring the Python SDK (`alpaca-py`).
//!
//! - [`trading`]: account, orders, positions, assets, watchlists, corporate
//!   actions and option contracts (`TradingClient`).
//! - [`data`]: historical and latest market data for stocks, crypto, options
//!   and news, plus the screener, corporate actions, forex rates and logo
//!   endpoints.
//! - [`stream`]: WebSocket streams for real-time market data and trading
//!   updates, plus Server-Sent Event streams for account activities and
//!   corporate actions.
//! - [`broker`]: fixed-income asset discovery (US treasuries and corporate
//!   bonds) on the Broker API (`FixedIncomeAssetsClient`) — a separate,
//!   Basic-Auth product from the Trading API.
//! - [`rest`]: credentials handling ([`rest::Credentials`]) shared by all
//!   clients.
//!
//! All clients are async (tokio) and return [`Result`] with the crate
//! [`Error`] type.

pub mod broker;
pub mod data;
pub mod error;
pub mod rest;
pub mod stream;
pub mod trading;

pub use error::{Error, Result};

/// Compiles the code blocks in `README.md` as doctests so the examples there
/// cannot drift out of sync with the API.
///
/// `cfg(doctest)` is set only while collecting doctests, so this item is absent
/// from normal builds and from the rendered documentation.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
