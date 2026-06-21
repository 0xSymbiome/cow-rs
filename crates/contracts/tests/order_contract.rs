#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::derive_partial_eq_without_eq,
    clippy::iter_on_single_items,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unnested_or_patterns,
    reason = "pedantic, nursery, style, and perf lints acceptable in test helper code"
)]

mod common;

use cow_sdk_contracts::{
    BUY_ETH_ADDRESS, ContractId, OrderCancellations, OrderUidParams, Registry, compute_order_uid,
    extract_order_uid_params, hash_order, hash_order_cancellation, hash_order_cancellations,
    pack_order_uid_params,
};
use cow_sdk_core::{
    Address, BuyTokenDestination, CowEnv, OrderData, SupportedChainId, TypedDataDomain,
};

use common::fixture_case;

const UPSTREAM_SEPOLIA_ORDER_DIGEST: &str =
    "0xc95c0093ac625698d627b6a16b20ea16a8a735493b6f9c7b72d996de978eb823";
const UPSTREAM_SEPOLIA_ORDER_UID: &str = "0xc95c0093ac625698d627b6a16b20ea16a8a735493b6f9c7b72d996de978eb823fb3c7eb936caa12b5a884d612393969a557d4307004c4c1e";
const UPSTREAM_SEPOLIA_ORDER_OWNER: &str = "0xfb3c7eb936caa12b5a884d612393969a557d4307";

fn sample_domain() -> TypedDataDomain {
    cow_sdk_test_utils::builders::sample_domain()
}

fn sample_order() -> OrderData {
    cow_sdk_test_utils::builders::OrderBuilder::weth_dai()
        .buy_balance(BuyTokenDestination::Internal)
        .build()
}

fn upstream_signing_sample_order() -> OrderData {
    cow_sdk_test_utils::builders::OrderBuilder::default().build()
}

fn upstream_signing_domain() -> TypedDataDomain {
    TypedDataDomain::new(
        "Gnosis Protocol".to_owned(),
        "v2".to_owned(),
        u64::from(SupportedChainId::Sepolia),
        Registry::default()
            .address(
                ContractId::Settlement,
                SupportedChainId::Sepolia,
                CowEnv::Prod,
            )
            .expect("canonical settlement address is registered for sepolia"),
    )
}

#[test]
fn buy_eth_sentinel_is_the_typed_native_marker() {
    // The sentinel is a typed `Address`; its canonical lowercase wire form is the
    // protocol's `0xEeee…EEeE` native-currency marker.
    assert_eq!(
        BUY_ETH_ADDRESS.to_hex_string(),
        "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
    );
}

#[test]
fn order_hash_and_uid_helpers_are_consistent() {
    let order = sample_order();
    let domain = sample_domain();

    let order_hash = hash_order(&domain, &order);
    assert_eq!(order_hash.to_hex_string().len(), 66);
    assert_eq!(hash_order(&domain, &order), order_hash);

    let owner = Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let uid = compute_order_uid(&domain, &order, &owner);

    let uid_case = fixture_case("contracts-order-uid-length");
    assert_eq!(
        uid.to_hex_string().trim_start_matches("0x").len(),
        usize::try_from(uid_case["expected"]["hex_chars"].as_u64().unwrap())
            .expect("fixture uid length must fit in usize")
    );

    let extracted = extract_order_uid_params(&uid).unwrap();
    assert_eq!(extracted.owner, owner);
    assert_eq!(extracted.valid_to, order.valid_to);
    assert_eq!(extracted.order_digest, order_hash);

    let roundtrip = pack_order_uid_params(&OrderUidParams::new(order_hash, owner, order.valid_to));
    assert_eq!(roundtrip, uid);

    let cancellation = hash_order_cancellation(&domain, &uid);
    let batch = hash_order_cancellations(&domain, &OrderCancellations::new(vec![uid, roundtrip]));
    assert_eq!(cancellation.to_hex_string().len(), 66);
    assert_eq!(batch.to_hex_string().len(), 66);
    assert_ne!(cancellation, batch);
}

#[test]
fn canonical_unsigned_order_path_matches_upstream_signing_fixture_digest_and_uid() {
    let unsigned = upstream_signing_sample_order();
    let domain = upstream_signing_domain();
    let owner = Address::new(UPSTREAM_SEPOLIA_ORDER_OWNER).unwrap();

    let digest = hash_order(&domain, &unsigned);
    assert_eq!(digest.to_hex_string(), UPSTREAM_SEPOLIA_ORDER_DIGEST);

    let uid = compute_order_uid(&domain, &unsigned, &owner);
    assert_eq!(uid.to_hex_string(), UPSTREAM_SEPOLIA_ORDER_UID);

    let unpacked = extract_order_uid_params(&uid).unwrap();
    assert_eq!(unpacked.owner, owner);
    assert_eq!(unpacked.valid_to, unsigned.valid_to);
    assert_eq!(
        unpacked.order_digest.to_hex_string(),
        UPSTREAM_SEPOLIA_ORDER_DIGEST
    );
}
