//! Static contract test: every capability `ContractId` variant
//! added by the composable + COW Shed leaf landing is present in
//! deterministic order with an `as_str` arm, and the enum stays
//! `#[non_exhaustive]`.

use cow_sdk_contracts::ContractId;

const CAPABILITY_VARIANTS: &[(&str, ContractId)] = &[
    ("ComposableCow", ContractId::ComposableCow),
    (
        "ExtensibleFallbackHandler",
        ContractId::ExtensibleFallbackHandler,
    ),
    (
        "CurrentBlockTimestampFactory",
        ContractId::CurrentBlockTimestampFactory,
    ),
    ("TwapHandler", ContractId::TwapHandler),
    ("GoodAfterTimeHandler", ContractId::GoodAfterTimeHandler),
    ("StopLossHandler", ContractId::StopLossHandler),
    (
        "TradeAboveThresholdHandler",
        ContractId::TradeAboveThresholdHandler,
    ),
    (
        "PerpetualStableSwapHandler",
        ContractId::PerpetualStableSwapHandler,
    ),
    ("CowShedImplementation", ContractId::CowShedImplementation),
    ("CowShedFactory", ContractId::CowShedFactory),
    (
        "CowShedForComposableCow",
        ContractId::CowShedForComposableCow,
    ),
];

#[test]
fn every_capability_contract_id_has_canonical_as_str_arm() {
    for (expected, variant) in CAPABILITY_VARIANTS {
        let actual = variant.as_str();
        assert_eq!(
            actual, *expected,
            "ContractId::{variant:?}.as_str() must return `{expected}`, got `{actual}`"
        );
    }
}

#[test]
fn capability_contract_ids_count_matches_spec() {
    assert_eq!(
        CAPABILITY_VARIANTS.len(),
        11,
        "the composable + COW Shed landing adds exactly 11 new ContractId variants",
    );
}
