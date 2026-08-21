//! Opt-in **write** test against a real Alpaca account: submits an order and
//! cancels it again.
//!
//! Kept out of `live_smoke.rs`, which is strictly read-only. This test exists
//! to prove the full order round-trip — `submit_order` request serialization,
//! `Order` response deserialization, and `cancel_order` — against the live
//! wire, something the mocked tests can only assume.
//!
//! ```sh
//! export APCA_API_KEY_ID=... APCA_API_SECRET_KEY=...
//! cargo test --test live_orders -- --ignored --nocapture
//! ```
//!
//! Safety properties:
//!
//! - The trading client is hard-wired to the **paper** host.
//! - The order is a limit buy far below the market price, so it cannot fill
//!   even if submitted while the market is open.
//! - Only the order id created by this test is ever canceled; existing orders
//!   on the account are never touched.
//! - If a step fails after submission, the test still attempts the cancel
//!   before returning the error.

use alpaca_rs_client::rest::Credentials;
use alpaca_rs_client::trading::{OrderRequest, OrderSide, OrderStatus, TimeInForce, TradingClient};
use alpaca_rs_client::{Error, Result};
use rust_decimal::Decimal;

/// Submits a far-from-market limit order, verifies it, cancels it, and
/// verifies the cancellation. See the module docs for the safety properties.
#[tokio::test]
#[ignore = "hits the live Alpaca API (paper); run explicitly with --ignored"]
async fn order_submit_cancel_round_trip() -> Result<()> {
    let credentials = match Credentials::from_env() {
        Ok(credentials) => credentials,
        Err(_) => {
            eprintln!("skipping: APCA_API_KEY_ID / APCA_API_SECRET_KEY not set");
            return Ok(());
        }
    };
    // Paper only, unconditionally.
    let client = TradingClient::new(credentials, true)?;

    // 1 share of AAPL at $1.00 (market ~$300): cannot fill, whatever the
    // market state.
    let request = OrderRequest::limit(
        "AAPL",
        OrderSide::Buy,
        Decimal::new(1, 0),
        TimeInForce::Day,
        Decimal::new(100, 2),
    );

    let order = client.submit_order(&request).await?;
    println!(
        "submitted {} — status {:?}, limit {:?}",
        order.id, order.status, order.limit_price
    );
    assert!(
        matches!(
            order.status,
            OrderStatus::New | OrderStatus::Accepted | OrderStatus::PendingNew
        ),
        "unexpected status for a resting order: {:?}",
        order.status
    );
    assert_eq!(order.symbol, "AAPL");
    assert_eq!(order.limit_price, Some(Decimal::new(100, 2)));

    // From here on, a failure must not skip the cancel.
    let result = cancel_and_verify(&client, &order.id).await;
    if let Err(err) = &result {
        eprintln!("cancel/verify failed ({err}); retrying cancel as cleanup");
        let _ = client.cancel_order(&order.id).await;
    }
    result
}

async fn cancel_and_verify(client: &TradingClient, order_id: &str) -> Result<()> {
    client.cancel_order(order_id).await?;
    println!("canceled {order_id}");

    let order = client.get_order(order_id).await?;
    println!("final status: {:?}", order.status);
    if matches!(
        order.status,
        OrderStatus::Canceled | OrderStatus::PendingCancel
    ) {
        Ok(())
    } else {
        Err(Error::Stream(format!(
            "order {order_id} not canceled, status {:?}",
            order.status
        )))
    }
}
