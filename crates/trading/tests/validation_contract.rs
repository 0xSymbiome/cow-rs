//! Contract suite pinning the typed [`OrderBoundsValidator`] surface.
//!
//! Each [`ClientRejection`] variant has at least one dedicated fixture
//! case so drift in either the reviewed protocol-invariant matrix or the
//! typed return shape surfaces through a failing test before it reaches
//! release. Every assertion is deterministic — the validator takes the
//! caller-supplied UNIX-seconds timestamp so no wall-clock skew affects
//! the pinned coverage.

#![allow(
    clippy::doc_markdown,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    reason = "pedantic lint group acceptable inside integration test code"
)]

use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, EVM_NATIVE_CURRENCY_ADDRESS, OrderData,
    OrderKind, SellTokenSource,
};
use cow_sdk_orderbook::SigningScheme;
use cow_sdk_test_utils::builders::address;
use cow_sdk_trading::{
    AmountSide, ClientRejection, LimitTradeParameters, OrderBoundsValidator, TradeParameters,
    TradingError,
};

const FROM: &str = "0x1111111111111111111111111111111111111111";
const SELL_TOKEN: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BUY_TOKEN: &str = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
// Canonical lowercase 0x-prefixed wire form (PROP-WB-004 / ADR 0052).
const WETH: &str = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
const OTHER_OWNER: &str = "0x2222222222222222222222222222222222222222";
const NOW: u64 = 1_700_000_000;
const VALID_TO: u32 = 1_700_003_600;

fn order() -> OrderData {
    OrderData::new(
        address(SELL_TOKEN),
        address(BUY_TOKEN),
        address(FROM),
        Amount::new("1000000000000000000").expect("test amount literal must be valid"),
        Amount::new("1000000").expect("test amount literal must be valid"),
        VALID_TO,
        app_data_hash(),
        Amount::ZERO,
        OrderKind::Sell,
        false,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
    )
}

fn app_data_hash() -> AppDataHash {
    AppDataHash::new("0x0000000000000000000000000000000000000000000000000000000000000000")
        .expect("app-data hash literal must be valid")
}

#[test]
fn happy_path_reaches_successful_validation() {
    let validator = OrderBoundsValidator::services_default();
    validator
        .validate(
            &order(),
            address(FROM),
            SigningScheme::Eip712,
            None,
            NOW,
            false,
        )
        .expect("happy-path order must validate");
}

#[test]
fn zero_from_rejects_as_missing_from() {
    let validator = OrderBoundsValidator::services_default();
    let error = validator
        .validate(
            &order(),
            address(ZERO_ADDRESS),
            SigningScheme::Eip712,
            None,
            NOW,
            false,
        )
        .expect_err("zero from must reject");
    assert!(matches!(error, ClientRejection::MissingFrom));
}

#[test]
fn valid_to_below_minimum_rejects_as_insufficient() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.valid_to = u32::try_from(NOW + 59).expect("valid_to must fit in u32");
    let error = validator
        .validate(
            &order,
            address(FROM),
            SigningScheme::Eip712,
            None,
            NOW,
            false,
        )
        .expect_err("sub-minimum validTo must reject");
    assert!(matches!(
        error,
        ClientRejection::ValidToInsufficient {
            min_seconds: 60,
            ..
        }
    ));
}

#[test]
fn valid_to_at_the_minimum_is_accepted() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.valid_to = u32::try_from(NOW + 60).expect("valid_to must fit in u32");
    validator
        .validate(
            &order,
            address(FROM),
            SigningScheme::Eip712,
            None,
            NOW,
            false,
        )
        .expect("at-minimum validTo must validate");
}

#[test]
fn valid_to_above_limit_rejects_as_excessive() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.valid_to = u32::try_from(NOW + 31_536_001).expect("valid_to must fit in u32");
    let error = validator
        .validate(
            &order,
            address(FROM),
            SigningScheme::Eip712,
            None,
            NOW,
            false,
        )
        .expect_err("over-maximum validTo must reject");
    assert!(matches!(
        error,
        ClientRejection::ValidToExcessive {
            max_seconds: 31_536_000,
            ..
        }
    ));
}

#[test]
fn pre_sign_scheme_bypasses_the_lifetime_ceiling() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.valid_to = u32::try_from(NOW + 31_536_001).expect("valid_to must fit in u32");
    validator
        .validate(
            &order,
            address(FROM),
            SigningScheme::PreSign,
            None,
            NOW,
            false,
        )
        .expect("PreSign scheme must bypass the lifetime ceiling");
}

#[test]
fn limit_class_accepts_valid_to_above_three_hours() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.valid_to = u32::try_from(NOW + 10_801).expect("valid_to must fit in u32");
    validator
        .validate(
            &order,
            address(FROM),
            SigningScheme::Eip712,
            None,
            NOW,
            false,
        )
        .expect("limit class must admit beyond the 3h ceiling");
}

#[test]
fn native_sell_token_rejects_on_non_ethflow_path() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.sell_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
    let error = validator
        .validate(
            &order,
            address(FROM),
            SigningScheme::Eip712,
            None,
            NOW,
            false,
        )
        .expect_err("native sell token must reject");
    assert!(matches!(error, ClientRejection::InvalidNativeSellToken));
}

#[test]
fn eth_flow_path_accepts_native_sell_token_but_still_enforces_zero_amount() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.sell_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
    validator
        .validate(
            &order,
            address(FROM),
            SigningScheme::Eip1271,
            None,
            NOW,
            true,
        )
        .expect("eth-flow path must admit the native sentinel as sell token");

    order.sell_amount = Amount::ZERO;
    let error = validator
        .validate(
            &order,
            address(FROM),
            SigningScheme::Eip1271,
            None,
            NOW,
            true,
        )
        .expect_err("eth-flow path must still reject zero amounts");
    assert!(matches!(
        error,
        ClientRejection::ZeroAmount {
            side: AmountSide::Sell,
        }
    ));
}

#[test]
fn validate_same_token_matches_services_allow_sell_policy() {
    #[derive(Clone, Copy)]
    enum Outcome {
        Accept,
        Reject(&'static str),
    }

    let validator = OrderBoundsValidator::services_default().with_weth_address(address(WETH));
    let cases = [
        (
            "same-token sell",
            SELL_TOKEN,
            SELL_TOKEN,
            OrderKind::Sell,
            Outcome::Accept,
        ),
        (
            "same-token buy",
            SELL_TOKEN,
            SELL_TOKEN,
            OrderKind::Buy,
            Outcome::Reject(SELL_TOKEN),
        ),
        (
            "WETH-native sell",
            WETH,
            EVM_NATIVE_CURRENCY_ADDRESS,
            OrderKind::Sell,
            Outcome::Accept,
        ),
        (
            "WETH-native buy",
            WETH,
            EVM_NATIVE_CURRENCY_ADDRESS,
            OrderKind::Buy,
            Outcome::Reject(WETH),
        ),
    ];

    for (label, sell, buy, kind, expected) in cases {
        let mut order = order();
        order.sell_token = address(sell);
        order.buy_token = address(buy);
        order.kind = kind;
        let result = validator.validate(
            &order,
            address(FROM),
            SigningScheme::Eip712,
            None,
            NOW,
            false,
        );
        match (expected, result) {
            (Outcome::Accept, Ok(())) => {}
            (
                Outcome::Reject(expected_token),
                Err(ClientRejection::SameBuyAndSellToken { token }),
            ) => {
                assert_eq!(
                    token.to_hex_string(),
                    expected_token,
                    "{label}: rejection must surface the offending token"
                );
            }
            (_, actual) => panic!("{label}: unexpected outcome: {actual:?}"),
        }
    }
}

#[test]
fn paired_weth_native_guard_requires_configured_weth_to_engage() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.sell_token = address(WETH);
    order.buy_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
    validator
        .validate(
            &order,
            address(FROM),
            SigningScheme::Eip712,
            None,
            NOW,
            false,
        )
        .expect("without configured WETH the native-buy pair is admitted by the exact-match guard");
}

#[test]
fn zero_sell_amount_rejects_as_zero_sell_side() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.sell_amount = Amount::ZERO;
    let error = validator
        .validate(
            &order,
            address(FROM),
            SigningScheme::Eip712,
            None,
            NOW,
            false,
        )
        .expect_err("zero sell amount must reject");
    assert!(matches!(
        error,
        ClientRejection::ZeroAmount {
            side: AmountSide::Sell,
        }
    ));
}

#[test]
fn zero_buy_amount_rejects_as_zero_buy_side() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.buy_amount = Amount::ZERO;
    let error = validator
        .validate(
            &order,
            address(FROM),
            SigningScheme::Eip712,
            None,
            NOW,
            false,
        )
        .expect_err("zero buy amount must reject");
    assert!(matches!(
        error,
        ClientRejection::ZeroAmount {
            side: AmountSide::Buy,
        }
    ));
}

#[test]
fn app_data_signer_mismatch_rejects_as_appdata_from_mismatch() {
    let validator = OrderBoundsValidator::services_default();
    let declared = address(OTHER_OWNER);
    let error = validator
        .validate(
            &order(),
            address(FROM),
            SigningScheme::Eip712,
            Some(declared),
            NOW,
            false,
        )
        .expect_err("app-data signer mismatch must reject");
    assert!(matches!(error, ClientRejection::AppdataFromMismatch { .. }));
}

#[test]
fn app_data_signer_match_passes_case_insensitively() {
    let validator = OrderBoundsValidator::services_default();
    let from = Address::new("0xABCDef0000000000000000000000000000000001")
        .expect("mixed-case address must parse");
    let declared = Address::new("0xabcdef0000000000000000000000000000000001")
        .expect("lower-case address must parse");
    validator
        .validate(
            &order(),
            from,
            SigningScheme::Eip712,
            Some(declared),
            NOW,
            false,
        )
        .expect("matching signer must validate case-insensitively");
}

#[test]
fn owner_mismatch_assertion_returns_typed_rejection() {
    let expected = address(FROM);
    let recovered = address(OTHER_OWNER);
    let rejection = cow_sdk_trading::validation::assert_owner_matches_signer(&expected, &recovered)
        .expect_err("differing owner and signer must reject");
    match rejection {
        ClientRejection::OwnerMismatch {
            expected: got_expected,
            recovered: got_recovered,
        } => {
            assert_eq!(got_expected, expected);
            assert_eq!(got_recovered, recovered);
        }
        other => panic!("expected OwnerMismatch, got {other:?}"),
    }
}

#[test]
fn owner_mismatch_lifts_through_trading_error_client_rejected() {
    let expected = address(FROM);
    let recovered = address(OTHER_OWNER);
    let error: TradingError =
        cow_sdk_trading::validation::assert_owner_matches_signer(&expected, &recovered)
            .expect_err("differing owner and signer must reject")
            .into();
    assert!(matches!(
        error,
        TradingError::ClientRejected(ClientRejection::OwnerMismatch { .. })
    ));
}

#[test]
fn trade_parameters_validate_enforces_builder_subset() {
    let native = Address::new(EVM_NATIVE_CURRENCY_ADDRESS).unwrap();
    let params = TradeParameters::new(
        OrderKind::Sell,
        native,
        address(BUY_TOKEN),
        cow_sdk_core::Amount::new("1000000").unwrap(),
    );
    let error = params
        .validate()
        .expect_err("native sell token must fail builder-level validation");
    assert!(matches!(error, ClientRejection::InvalidNativeSellToken));
}

#[test]
fn trade_parameters_validate_rejects_zero_amount() {
    let params = TradeParameters::new(
        OrderKind::Sell,
        address(SELL_TOKEN),
        address(BUY_TOKEN),
        cow_sdk_core::Amount::ZERO,
    );
    let error = params
        .validate()
        .expect_err("zero amount must fail builder-level validation");
    assert!(matches!(
        error,
        ClientRejection::ZeroAmount {
            side: AmountSide::Sell,
        }
    ));
}

#[test]
fn limit_trade_parameters_validate_rejects_zero_buy_amount() {
    let params = LimitTradeParameters::new(
        OrderKind::Sell,
        address(SELL_TOKEN),
        address(BUY_TOKEN),
        cow_sdk_core::Amount::new("1000000").unwrap(),
        cow_sdk_core::Amount::ZERO,
    );
    let error = params
        .validate()
        .expect_err("zero buy amount must fail builder-level validation");
    assert!(matches!(
        error,
        ClientRejection::ZeroAmount {
            side: AmountSide::Buy,
        }
    ));
}
