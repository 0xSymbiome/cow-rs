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

use cow_sdk_core::{Address, EVM_NATIVE_CURRENCY_ADDRESS, OrderKind};
use cow_sdk_orderbook::{OrderCreation, SigningScheme};
use cow_sdk_trading::{
    AmountSide, ClientRejection, LimitTradeParameters, OrderBoundsValidator, OrderValidityBounds,
    SubmissionClass, TradeParameters, TradingError,
};

const FROM: &str = "0x1111111111111111111111111111111111111111";
const SELL_TOKEN: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BUY_TOKEN: &str = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
const OTHER_OWNER: &str = "0x2222222222222222222222222222222222222222";
const NOW: u64 = 1_700_000_000;
const VALID_TO: u32 = 1_700_003_600;

fn address(hex: &str) -> Address {
    Address::new(hex).expect("fixture address must be valid")
}

fn order() -> OrderCreation {
    OrderCreation::new(
        address(SELL_TOKEN),
        address(BUY_TOKEN),
        "1000000000000000000",
        "1000000",
        VALID_TO,
        OrderKind::Sell,
        SigningScheme::Eip712,
        "0x",
        address(FROM),
    )
}

#[test]
fn happy_path_reaches_successful_validation() {
    let validator = OrderBoundsValidator::services_default();
    validator
        .validate(&order(), SigningScheme::Eip712, None, NOW, false)
        .expect("happy-path order must validate");
}

#[test]
fn zero_from_rejects_as_missing_from() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.from = address(ZERO_ADDRESS);
    let error = validator
        .validate(&order, SigningScheme::Eip712, None, NOW, false)
        .expect_err("zero from must reject");
    assert!(matches!(error, ClientRejection::MissingFrom));
}

#[test]
fn valid_to_below_minimum_rejects_as_insufficient() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.valid_to = u32::try_from(NOW + 59).expect("valid_to must fit in u32");
    let error = validator
        .validate(&order, SigningScheme::Eip712, None, NOW, false)
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
        .validate(&order, SigningScheme::Eip712, None, NOW, false)
        .expect("at-minimum validTo must validate");
}

#[test]
fn valid_to_above_limit_rejects_as_excessive() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.valid_to = u32::try_from(NOW + 31_536_001).expect("valid_to must fit in u32");
    let error = validator
        .validate(&order, SigningScheme::Eip712, None, NOW, false)
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
        .validate(&order, SigningScheme::PreSign, None, NOW, false)
        .expect("PreSign scheme must bypass the lifetime ceiling");
}

#[test]
fn liquidity_class_bypasses_the_lifetime_ceiling() {
    let validator = OrderBoundsValidator::new(
        OrderValidityBounds::SERVICES_DEFAULT,
        SubmissionClass::Liquidity,
    );
    let mut order = order();
    order.valid_to = u32::try_from(NOW + 31_536_001).expect("valid_to must fit in u32");
    validator
        .validate(&order, SigningScheme::Eip712, None, NOW, false)
        .expect("Liquidity class must bypass the lifetime ceiling");
}

#[test]
fn market_class_rejects_valid_to_above_three_hours() {
    let validator = OrderBoundsValidator::new(
        OrderValidityBounds::SERVICES_DEFAULT,
        SubmissionClass::Market,
    );
    let mut order = order();
    order.valid_to = u32::try_from(NOW + 10_801).expect("valid_to must fit in u32");
    let error = validator
        .validate(&order, SigningScheme::Eip712, None, NOW, false)
        .expect_err("market class must enforce the 3h ceiling");
    assert!(matches!(
        error,
        ClientRejection::ValidToExcessive {
            max_seconds: 10_800,
            ..
        }
    ));
}

#[test]
fn limit_class_accepts_valid_to_above_three_hours() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.valid_to = u32::try_from(NOW + 10_801).expect("valid_to must fit in u32");
    validator
        .validate(&order, SigningScheme::Eip712, None, NOW, false)
        .expect("limit class must admit beyond the 3h ceiling");
}

#[test]
fn custom_bounds_from_builder_actually_apply_to_the_validator() {
    let tighter = OrderValidityBounds {
        min: std::time::Duration::from_secs(120),
        ..OrderValidityBounds::SERVICES_DEFAULT
    };
    let validator = OrderBoundsValidator::new(tighter, SubmissionClass::Limit);
    let mut order = order();
    order.valid_to = u32::try_from(NOW + 60).expect("valid_to must fit in u32");
    let error = validator
        .validate(&order, SigningScheme::Eip712, None, NOW, false)
        .expect_err("custom 120s minimum must reject a 60s lifetime");
    assert!(matches!(
        error,
        ClientRejection::ValidToInsufficient {
            min_seconds: 120,
            ..
        }
    ));
}

#[test]
fn native_sell_token_rejects_on_non_ethflow_path() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.sell_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
    let error = validator
        .validate(&order, SigningScheme::Eip712, None, NOW, false)
        .expect_err("native sell token must reject");
    assert!(matches!(error, ClientRejection::InvalidNativeSellToken));
}

#[test]
fn eth_flow_path_accepts_native_sell_token_but_still_enforces_zero_amount() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.sell_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
    validator
        .validate(&order, SigningScheme::Eip1271, None, NOW, true)
        .expect("eth-flow path must admit the native sentinel as sell token");

    order.sell_amount = "0".to_owned();
    let error = validator
        .validate(&order, SigningScheme::Eip1271, None, NOW, true)
        .expect_err("eth-flow path must still reject zero amounts");
    assert!(matches!(
        error,
        ClientRejection::ZeroAmount {
            side: AmountSide::Sell,
        }
    ));
}

#[test]
fn identical_sell_and_buy_tokens_reject_as_same_token() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.buy_token = address(SELL_TOKEN);
    let error = validator
        .validate(&order, SigningScheme::Eip712, None, NOW, false)
        .expect_err("identical sell and buy tokens must reject");
    assert!(matches!(error, ClientRejection::SameBuyAndSellToken { .. }));
}

#[test]
fn paired_weth_sell_and_native_buy_rejects_through_weth_configured_validator() {
    let validator = OrderBoundsValidator::services_default().with_weth_address(address(WETH));
    let mut order = order();
    order.sell_token = address(WETH);
    order.buy_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
    let error = validator
        .validate(&order, SigningScheme::Eip712, None, NOW, false)
        .expect_err("WETH sell paired with the native-buy sentinel must reject");
    match error {
        ClientRejection::SameBuyAndSellToken { token } => {
            assert_eq!(
                token.as_str(),
                WETH,
                "rejection must surface the WETH address"
            );
        }
        other => panic!("expected SameBuyAndSellToken, got {other:?}"),
    }
}

#[test]
fn paired_weth_native_guard_requires_configured_weth_to_engage() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.sell_token = address(WETH);
    order.buy_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
    validator
        .validate(&order, SigningScheme::Eip712, None, NOW, false)
        .expect("without configured WETH the native-buy pair is admitted by the exact-match guard");
}

#[test]
fn zero_sell_amount_rejects_as_zero_sell_side() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order();
    order.sell_amount = "0".to_owned();
    let error = validator
        .validate(&order, SigningScheme::Eip712, None, NOW, false)
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
    order.buy_amount = "0".to_owned();
    let error = validator
        .validate(&order, SigningScheme::Eip712, None, NOW, false)
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
        .validate(&order(), SigningScheme::Eip712, Some(declared), NOW, false)
        .expect_err("app-data signer mismatch must reject");
    assert!(matches!(error, ClientRejection::AppdataFromMismatch { .. }));
}

#[test]
fn app_data_signer_match_passes_case_insensitively() {
    let validator = OrderBoundsValidator::services_default();
    let mut mixed_case = order();
    mixed_case.from = Address::new("0xABCDef0000000000000000000000000000000001")
        .expect("mixed-case address must parse");
    let declared = Address::new("0xabcdef0000000000000000000000000000000001")
        .expect("lower-case address must parse");
    validator
        .validate(
            &mixed_case,
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
fn validator_is_pure_and_idempotent() {
    let validator = OrderBoundsValidator::services_default();
    let order = order();
    let first = validator.validate(&order, SigningScheme::Eip712, None, NOW, false);
    let second = validator.validate(&order, SigningScheme::Eip712, None, NOW, false);
    assert!(first.is_ok() && second.is_ok());
}

#[test]
fn trade_parameters_validate_enforces_builder_subset() {
    let native = Address::new(EVM_NATIVE_CURRENCY_ADDRESS).unwrap();
    let params = TradeParameters::new(
        OrderKind::Sell,
        native,
        18,
        address(BUY_TOKEN),
        18,
        cow_sdk_core::Amount::new("1000000").unwrap(),
    );
    let error = params
        .validate()
        .expect_err("native sell token must fail builder-level validation");
    assert!(matches!(error, ClientRejection::InvalidNativeSellToken));
}

#[test]
fn trade_parameters_validate_rejects_same_tokens() {
    let params = TradeParameters::new(
        OrderKind::Sell,
        address(SELL_TOKEN),
        18,
        address(SELL_TOKEN),
        18,
        cow_sdk_core::Amount::new("1000000").unwrap(),
    );
    let error = params
        .validate()
        .expect_err("same sell/buy token must fail builder-level validation");
    assert!(matches!(error, ClientRejection::SameBuyAndSellToken { .. }));
}

#[test]
fn trade_parameters_validate_rejects_zero_amount() {
    let params = TradeParameters::new(
        OrderKind::Sell,
        address(SELL_TOKEN),
        18,
        address(BUY_TOKEN),
        18,
        cow_sdk_core::Amount::zero(),
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
fn amount_is_zero_matches_the_typed_numeric_predicate() {
    use cow_sdk_core::Amount;
    assert!(
        Amount::zero().is_zero(),
        "Amount::zero() must report is_zero() == true"
    );
    assert!(
        !Amount::new("1").unwrap().is_zero(),
        "Amount::new(\"1\") must report is_zero() == false",
    );
    assert!(
        !Amount::new("1000000000000000000").unwrap().is_zero(),
        "a non-zero large amount must report is_zero() == false",
    );
    assert!(
        Amount::new("0").unwrap().is_zero(),
        "Amount::new(\"0\") must report is_zero() == true",
    );
}

#[test]
fn limit_trade_parameters_validate_rejects_zero_buy_amount() {
    let params = LimitTradeParameters::new(
        OrderKind::Sell,
        address(SELL_TOKEN),
        18,
        address(BUY_TOKEN),
        18,
        cow_sdk_core::Amount::new("1000000").unwrap(),
        cow_sdk_core::Amount::zero(),
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
