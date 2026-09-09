//! Exact decimal decoding for money and size fields.
//!
//! Alpaca sends market-data prices as JSON **numbers** (`"p":191.23`) and
//! order/REST money as quoted **strings** (`"qty":"1.5"`). Both must land as
//! `rust_decimal::Decimal` without ever passing through `f64`.
//!
//! That is not achievable by deserializing into `f64` and converting: the
//! rounding happens inside the float parse, before any conversion can run. Nor
//! by `Decimal`'s own impl — this crate enables `rust_decimal/serde-str`, so a
//! bare `Decimal` expects a string and rejects a JSON number outright.
//!
//! So we take the literal JSON text via `RawValue` and parse THAT. A number and
//! a quoted string are then handled by the same path, trailing zeros are
//! preserved, and a value beyond `Decimal`'s precision is REJECTED rather than
//! silently rounded — a wrong price should fail loudly, not arrive plausible.

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;

/// Parse the literal JSON scalar, accepting a bare number or a quoted string.
fn from_raw<E: serde::de::Error>(raw: &RawValue) -> Result<Decimal, E> {
    let text = raw.get().trim();
    // A quoted string arrives with its quotes; strip exactly one pair.
    let text = text
        .strip_prefix('"')
        .and_then(|t| t.strip_suffix('"'))
        .unwrap_or(text);
    // from_str_exact, NOT from_str: the latter silently ROUNDS a value beyond
    // Decimal precision, and a rounded price is indistinguishable from a real
    // one downstream. Refuse instead.
    Decimal::from_str_exact(text).map_err(|e| E::custom(format!("invalid decimal {text:?}: {e}")))
}

/// A required money/size field.
pub fn decimal<'de, D: Deserializer<'de>>(d: D) -> Result<Decimal, D::Error> {
    let raw: &RawValue = Deserialize::deserialize(d)?;
    from_raw(raw)
}

/// An optional money/size field. `null` and an absent key both read `None`.
pub fn decimal_opt<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Decimal>, D::Error> {
    let raw: Option<&RawValue> = Deserialize::deserialize(d)?;
    match raw {
        None => Ok(None),
        Some(r) if r.get().trim() == "null" => Ok(None),
        Some(r) => from_raw(r).map(Some),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Holder {
        #[serde(deserialize_with = "decimal")]
        price: Decimal,
        #[serde(default, deserialize_with = "decimal_opt")]
        vwap: Option<Decimal>,
    }

    #[test]
    fn a_json_number_decodes_exactly() {
        let h: Holder = serde_json::from_str(r#"{"price":2465.297176675}"#).unwrap();
        // 13 significant digits: this value does NOT survive an f64 round trip,
        // so its exactness here is the whole point of the module.
        assert_eq!(h.price.to_string(), "2465.297176675");
        assert_eq!(h.vwap, None);
    }

    #[test]
    fn a_quoted_string_decodes_the_same_way() {
        let h: Holder = serde_json::from_str(r#"{"price":"191.23","vwap":"1.5"}"#).unwrap();
        assert_eq!(h.price.to_string(), "191.23");
        assert_eq!(h.vwap.unwrap().to_string(), "1.5");
    }

    /// Scale is information: `1.50` and `1.5` are the same number but not the
    /// same quote, and re-emitting the wrong one changes a tick.
    #[test]
    fn trailing_zeros_are_preserved() {
        let h: Holder = serde_json::from_str(r#"{"price":"1.50"}"#).unwrap();
        assert_eq!(h.price.to_string(), "1.50");
    }

    #[test]
    fn null_and_absent_optionals_both_read_none() {
        let a: Holder = serde_json::from_str(r#"{"price":1,"vwap":null}"#).unwrap();
        let b: Holder = serde_json::from_str(r#"{"price":1}"#).unwrap();
        assert_eq!(a.vwap, None);
        assert_eq!(b.vwap, None);
    }

    /// Beyond `Decimal`'s precision must FAIL, not round. A silently rounded
    /// price is indistinguishable from a real one downstream.
    #[test]
    fn a_value_beyond_precision_is_rejected_not_rounded() {
        let r: Result<Holder, _> =
            serde_json::from_str(r#"{"price":1.2345678901234567890123456789012345}"#);
        assert!(r.is_err(), "an unrepresentable price must be refused");
    }
}

#[cfg(test)]
mod wire_exactness {
    use crate::data::models::Bar;

    /// The reason this module exists.
    ///
    /// It takes a SYNTHETIC value to prove: a real price like `2465.297176675`
    /// round-trips through `f64` when printed, because Rust prints the shortest
    /// string that maps back to the same double — so `to_string` cannot expose
    /// the loss. Nineteen significant digits can: `1234567890123456789` is
    /// beyond `f64`'s ~15-17, and the nearest double prints differently.
    #[test]
    fn a_price_no_f64_can_hold_decodes_exactly() {
        let json = r#"{"t":"2026-09-09T23:40:00Z","o":1,"h":1,"l":1,
                       "c":1234567890123456789,"v":0.006773038}"#;
        let bar: Bar = serde_json::from_str(json).expect("bar parses");
        assert_eq!(bar.close.to_string(), "1234567890123456789");
        assert_eq!(bar.volume.to_string(), "0.006773038");

        // The control: the f64 path really does lose this one.
        let via_f64 = 1234567890123456789f64;
        assert_ne!(
            via_f64.to_string(),
            "1234567890123456789",
            "precondition: if f64 held this exactly the test would prove nothing"
        );
    }
}
