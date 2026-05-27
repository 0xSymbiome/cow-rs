//! Contract suite pinning the `LimitTradeParametersFromQuote` newtype
//! invariant.
//!
//! Every public path that produces a `LimitTradeParametersFromQuote`
//! must satisfy: `quote_id` is non-`None` by construction. The
//! constructor concentrates the check, the `EthFlow` native-currency
//! submission entry and the `EthFlow` transaction helper bind to the
//! newtype at the public boundary, and the public `quote_id()`
//! accessor returns the inner value without an `Option`.

use cow_sdk_core::{Address, Amount, OrderKind};
use cow_sdk_trading::{LimitTradeParameters, LimitTradeParametersFromQuote, TradingError};

const SELL: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BUY: &str = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn sample_limit(quote_id: Option<i64>) -> LimitTradeParameters {
    let mut params = LimitTradeParameters::new(
        OrderKind::Sell,
        Address::new(SELL).expect("sell address literal must be valid"),
        Address::new(BUY).expect("buy address literal must be valid"),
        Amount::new("1000000000000000000").expect("sell amount literal must be valid"),
        Amount::new("3000000000000000000").expect("buy amount literal must be valid"),
    );
    if let Some(id) = quote_id {
        params = params.with_quote_id(id);
    }
    params
}

#[test]
fn try_from_limit_rejects_missing_quote_id_with_typed_diagnostic() {
    let limit = sample_limit(None);
    let error = LimitTradeParametersFromQuote::try_from_limit(limit)
        .expect_err("missing quote id must produce the typed diagnostic");
    assert!(matches!(
        error,
        TradingError::MissingQuoteId("`EthFlow` order posting")
    ));
}

#[test]
fn try_from_limit_accepts_present_quote_id_and_preserves_fields() {
    let limit = sample_limit(Some(42));
    let from_quote = LimitTradeParametersFromQuote::try_from_limit(limit.clone())
        .expect("present quote id must build the newtype");
    let inner = from_quote.as_limit();
    assert_eq!(inner.sell_token, limit.sell_token);
    assert_eq!(inner.buy_token, limit.buy_token);
    assert_eq!(inner.sell_amount, limit.sell_amount);
    assert_eq!(inner.buy_amount, limit.buy_amount);
    assert_eq!(inner.quote_id, Some(42));
}

#[test]
fn quote_id_accessor_returns_the_inner_value_without_option() {
    let limit = sample_limit(Some(7));
    let from_quote = LimitTradeParametersFromQuote::try_from_limit(limit)
        .expect("present quote id must build the newtype");
    let id: i64 = from_quote.quote_id();
    assert_eq!(id, 7);
}

#[test]
fn quote_id_accessor_supports_negative_and_boundary_values() {
    for boundary in [i64::MIN, -1, 0, 1, i64::MAX] {
        let limit = sample_limit(Some(boundary));
        let from_quote = LimitTradeParametersFromQuote::try_from_limit(limit)
            .expect("present quote id must build the newtype");
        assert_eq!(from_quote.quote_id(), boundary);
    }
}

#[test]
fn into_limit_returns_the_underlying_value_unchanged() {
    let limit = sample_limit(Some(123));
    let expected = limit.clone();
    let from_quote = LimitTradeParametersFromQuote::try_from_limit(limit)
        .expect("present quote id must build the newtype");
    let returned = from_quote.into_limit();
    assert_eq!(returned, expected);
}

#[test]
fn as_ref_returns_a_reference_to_the_underlying_value() {
    let limit = sample_limit(Some(99));
    let from_quote = LimitTradeParametersFromQuote::try_from_limit(limit.clone())
        .expect("present quote id must build the newtype");
    let as_ref: &LimitTradeParameters = from_quote.as_ref();
    assert_eq!(*as_ref, limit);
}

#[test]
fn serde_round_trips_through_the_flattened_inner_shape() {
    let limit = sample_limit(Some(1234));
    let from_quote = LimitTradeParametersFromQuote::try_from_limit(limit.clone())
        .expect("present quote id must build the newtype");
    let serialized =
        serde_json::to_string(&from_quote).expect("serialization must succeed for the newtype");
    let json: serde_json::Value =
        serde_json::from_str(&serialized).expect("serialized output must parse as json");
    assert_eq!(
        json.get("quoteId"),
        Some(&serde_json::Value::Number(1234.into())),
        "quote id must be present on the flattened wire shape"
    );

    let round_tripped: LimitTradeParametersFromQuote =
        serde_json::from_str(&serialized).expect("round trip must reconstruct the newtype");
    assert_eq!(round_tripped.quote_id(), 1234);
    assert_eq!(round_tripped.as_limit().sell_token, limit.sell_token);
}
