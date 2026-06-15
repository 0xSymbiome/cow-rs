//! Property-based coverage for the deterministic trading validator boundary.
//!
//! The validator takes `now` from its caller, so these tests pin both the
//! stability of the classification while an order stays in the future and the
//! integer-edge behavior of the not-expired check near `u32::MAX` / `u64::MAX`.

#![allow(
    clippy::doc_markdown,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    reason = "pedantic lint group acceptable inside integration test code"
)]

use std::panic::{AssertUnwindSafe, catch_unwind};

use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderData, OrderKind, SellTokenSource,
};
use cow_sdk_test_utils::builders::address;
use cow_sdk_trading::{AmountSide, ClientRejection, OrderBoundsValidator};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

const REGRESSION_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/proptest-regressions/property_contract.txt"
);
const MAX_DELTA_SECONDS: u64 = 3_600;
const MIN_VALIDITY_MARGIN_SECONDS: u64 = 61;

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

fn arbitrary_order() -> impl Strategy<Value = (OrderData, Address)> {
    (
        address_strategy(),
        address_strategy(),
        amount_strategy(),
        amount_strategy(),
        any::<u32>(),
        order_kind_strategy(),
        address_strategy(),
        address_strategy(),
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
                from,
                receiver,
                partially_fillable,
            )| {
                let order = OrderData::new(
                    sell_token,
                    buy_token,
                    receiver,
                    sell_amount,
                    buy_amount,
                    valid_to,
                    app_data_hash(),
                    Amount::ZERO,
                    kind,
                    partially_fillable,
                    SellTokenSource::Erc20,
                    BuyTokenDestination::Erc20,
                );
                (order, from)
            },
        )
}

fn order_template() -> OrderData {
    OrderData::new(
        address("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        address("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        template_from(),
        Amount::new("1000000000000000000").expect("fixture amount must be valid"),
        Amount::new("1000000").expect("fixture amount must be valid"),
        u32::MAX,
        app_data_hash(),
        Amount::ZERO,
        OrderKind::Sell,
        false,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
    )
}

fn template_from() -> Address {
    address("0x1111111111111111111111111111111111111111")
}

fn app_data_hash() -> AppDataHash {
    AppDataHash::new("0x0000000000000000000000000000000000000000000000000000000000000000")
        .expect("app-data hash literal must be valid")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidationClass {
    Accepted,
    ValidToInPast,
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
        Err(ClientRejection::ValidToInPast { .. }) => ValidationClass::ValidToInPast,
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
    fn validator_classification_is_stable_while_order_stays_in_the_future(
        (mut order, from) in arbitrary_order(),
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
        let outcome_now = validator.validate(&order, from, app_data_signer, now, is_eth_flow);
        let outcome_then = validator.validate(&order, from, app_data_signer, then, is_eth_flow);

        prop_assert_eq!(
            validation_class(&outcome_now),
            validation_class(&outcome_then),
            "classification must stay stable while the order is not expired at either observation"
        );
    }
}

#[test]
fn validator_handles_u32_max_validto_without_overflow() {
    let validator = OrderBoundsValidator::services_default();
    let mut order = order_template();
    order.valid_to = u32::MAX;

    for (now, expected) in [
        (0u64, ValidationClass::Accepted),
        (u64::from(u32::MAX) - 1, ValidationClass::Accepted),
        (u64::from(u32::MAX), ValidationClass::ValidToInPast),
        (u64::from(u32::MAX) + 1, ValidationClass::ValidToInPast),
        (u64::MAX - 1, ValidationClass::ValidToInPast),
        (u64::MAX, ValidationClass::ValidToInPast),
    ] {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            validator.validate(&order, template_from(), None, now, false)
        }))
        .expect("validator must not panic at timestamp extremes");

        assert_eq!(
            validation_class(&outcome),
            expected,
            "now={now} must resolve to the documented typed classification"
        );
    }
}
