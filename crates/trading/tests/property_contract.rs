//! Property-based coverage for the deterministic trading validator boundary.
//!
//! The validator takes `now` from its caller, so these tests pin both the
//! ordinary monotonic-within-window contract and the integer-edge behavior near
//! `u32::MAX` / `u64::MAX`.

#![allow(
    clippy::doc_markdown,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    reason = "pedantic lint group acceptable inside integration test code"
)]

use std::panic::{AssertUnwindSafe, catch_unwind};

use cow_sdk_core::{Address, Amount, OrderKind};
use cow_sdk_orderbook::{OrderCreation, SigningScheme};
use cow_sdk_trading::{AmountSide, ClientRejection, OrderBoundsValidator};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

const REGRESSION_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/proptest-regressions/property_contract.txt"
);
const MAX_DELTA_SECONDS: u64 = 3_600;
const MIN_VALIDITY_MARGIN_SECONDS: u64 = 61;

fn address(hex: &str) -> Address {
    Address::new(hex).expect("fixture address must be valid")
}

fn address_strategy() -> impl Strategy<Value = Address> {
    any::<[u8; 20]>().prop_map(Address::from_bytes)
}

fn amount_strategy() -> impl Strategy<Value = Amount> {
    any::<u128>().prop_map(|value| {
        Amount::new(value.to_string()).expect("u128 string must remain a valid amount")
    })
}

fn order_kind_strategy() -> impl Strategy<Value = OrderKind> {
    any::<bool>().prop_map(|is_buy| {
        if is_buy {
            OrderKind::Buy
        } else {
            OrderKind::Sell
        }
    })
}

fn signing_scheme_strategy() -> impl Strategy<Value = SigningScheme> {
    (0u8..4).prop_map(|value| match value {
        0 => SigningScheme::Eip712,
        1 => SigningScheme::EthSign,
        2 => SigningScheme::Eip1271,
        _ => SigningScheme::PreSign,
    })
}

fn arbitrary_order() -> impl Strategy<Value = OrderCreation> {
    (
        address_strategy(),
        address_strategy(),
        amount_strategy(),
        amount_strategy(),
        any::<u32>(),
        order_kind_strategy(),
        signing_scheme_strategy(),
        any::<[u8; 8]>(),
        address_strategy(),
        prop::option::of(address_strategy()),
        any::<bool>(),
    )
        .prop_map(
            |(
                sell_token,
                buy_token,
                sell_amount,
                buy_amount,
                valid_to,
                kind,
                signing_scheme,
                signature_seed,
                from,
                receiver,
                partially_fillable,
            )| {
                let signature = format!("0x{}", alloy_primitives::hex::encode(signature_seed));
                let mut order = OrderCreation::new(
                    sell_token,
                    buy_token,
                    sell_amount,
                    buy_amount,
                    valid_to,
                    kind,
                    signing_scheme,
                    signature,
                    from,
                )
                .with_partially_fillable(partially_fillable);
                if let Some(receiver) = receiver {
                    order = order.with_receiver(receiver);
                }
                order
            },
        )
}

fn order_template() -> OrderCreation {
    OrderCreation::new(
        address("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        address("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        Amount::new("1000000000000000000").expect("fixture amount must be valid"),
        Amount::new("1000000").expect("fixture amount must be valid"),
        u32::MAX,
        OrderKind::Sell,
        SigningScheme::Eip712,
        "0x",
        address("0x1111111111111111111111111111111111111111"),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidationClass {
    Accepted,
    ValidToInsufficient,
    ValidToExcessive,
    MissingFrom,
    AppdataFromMismatch,
    SameBuyAndSellToken,
    InvalidNativeSellToken,
    ZeroAmount(AmountSide),
    OwnerMismatch,
    InvalidPartnerFee,
    Unknown,
}

fn validation_class(outcome: &Result<(), ClientRejection>) -> ValidationClass {
    match outcome {
        Ok(()) => ValidationClass::Accepted,
        Err(ClientRejection::ValidToInsufficient { .. }) => ValidationClass::ValidToInsufficient,
        Err(ClientRejection::ValidToExcessive { .. }) => ValidationClass::ValidToExcessive,
        Err(ClientRejection::MissingFrom) => ValidationClass::MissingFrom,
        Err(ClientRejection::AppdataFromMismatch { .. }) => ValidationClass::AppdataFromMismatch,
        Err(ClientRejection::SameBuyAndSellToken { .. }) => ValidationClass::SameBuyAndSellToken,
        Err(ClientRejection::InvalidNativeSellToken) => ValidationClass::InvalidNativeSellToken,
        Err(ClientRejection::ZeroAmount { side }) => ValidationClass::ZeroAmount(*side),
        Err(ClientRejection::OwnerMismatch { .. }) => ValidationClass::OwnerMismatch,
        Err(ClientRejection::InvalidPartnerFee { .. }) => ValidationClass::InvalidPartnerFee,
        Err(_) => ValidationClass::Unknown,
    }
}

#[test]
fn same_token_validation_class_is_buy_side_only() {
    let validator = OrderBoundsValidator::services_default();
    let now = 1_700_000_000;
    let mut buy_order = order_template();
    buy_order.valid_to = u32::try_from(now + 3_600).expect("valid_to must fit in u32");
    buy_order.buy_token = buy_order.sell_token;
    buy_order.kind = OrderKind::Buy;

    let buy_outcome = validator.validate(&buy_order, SigningScheme::Eip712, None, now, false);
    assert_eq!(
        validation_class(&buy_outcome),
        ValidationClass::SameBuyAndSellToken
    );

    let mut sell_order = buy_order;
    sell_order.kind = OrderKind::Sell;
    let sell_outcome = validator.validate(&sell_order, SigningScheme::Eip712, None, now, false);
    assert_eq!(validation_class(&sell_outcome), ValidationClass::Accepted);
}

fn normalize_now_inside_u32_window(now_seconds: u64) -> u64 {
    let max_now = u64::from(u32::MAX) - MAX_DELTA_SECONDS - MIN_VALIDITY_MARGIN_SECONDS;
    now_seconds % (max_now + 1)
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSION_FILE))),
        ..ProptestConfig::default()
    })]

    #[test]
    fn validator_is_monotonic_within_window_via_proptest(
        mut order in arbitrary_order(),
        scheme in signing_scheme_strategy(),
        app_data_signer in prop::option::of(address_strategy()),
        now_seconds in any::<u64>(),
        delta_seconds in 0u64..MAX_DELTA_SECONDS,
        is_eth_flow in any::<bool>(),
    ) {
        let now = normalize_now_inside_u32_window(now_seconds);
        let then = now + delta_seconds;
        order.valid_to = u32::try_from(then + MIN_VALIDITY_MARGIN_SECONDS)
            .expect("normalized validity window must fit in u32");

        let validator = OrderBoundsValidator::services_default();
        let outcome_now = validator.validate(
            &order,
            scheme,
            app_data_signer,
            now,
            is_eth_flow,
        );
        let outcome_then = validator.validate(
            &order,
            scheme,
            app_data_signer,
            then,
            is_eth_flow,
        );

        prop_assert_eq!(
            validation_class(&outcome_now),
            validation_class(&outcome_then),
            "classification must remain stable while both observations are inside the validity window"
        );
    }
}

#[test]
fn validator_handles_u32_max_validto_without_overflow() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order_template();
    order.valid_to = u32::MAX;

    for (now, expected) in [
        (0u64, ValidationClass::ValidToExcessive),
        (
            u64::from(u32::MAX) - 1,
            ValidationClass::ValidToInsufficient,
        ),
        (u64::from(u32::MAX), ValidationClass::ValidToInsufficient),
        (
            u64::from(u32::MAX) + 1,
            ValidationClass::ValidToInsufficient,
        ),
        (u64::MAX - 1, ValidationClass::ValidToInsufficient),
        (u64::MAX, ValidationClass::ValidToInsufficient),
    ] {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            validator.validate(&order, SigningScheme::Eip712, None, now, false)
        }))
        .expect("validator must not panic at timestamp extremes");

        assert_eq!(
            validation_class(&outcome),
            expected,
            "now={now} must resolve to the documented typed classification"
        );
    }
}
