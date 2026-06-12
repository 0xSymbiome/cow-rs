#![cfg(feature = "cow-shed")]

//! COW Shed selector parity contract: every selector row in
//! `parity/fixtures/cow_shed/canonical_selectors.json` is re-derived from its
//! canonical signature with an independent keccak implementation
//! (`sha3::Keccak256`, not alloy's) and asserted equal to the macro-emitted
//! `SolCall::SELECTOR` constant of the bound function — the fixture, the
//! keccak preimage, and the `sol!` bindings must agree three ways. The record
//! anchors to the DEPLOYED v1.0.x runtime (cow-shed tag v1.0.1, cross-checked
//! against the deployed-runtime factory ABI the TS arbiter ships); the
//! ENS-purged 1-arg `initializeProxy(address)` exists only in the unsupported
//! v2.x source generations. The canonical EIP-712 type-hash values are pinned
//! by `cow_shed/execute_hooks_digest.json`.

use alloy_sol_types::SolCall;
use cow_sdk_contracts::cow_shed::bindings::{COWShed, COWShedFactory};
use sha3::{Digest, Keccak256};

fn canonical_fixture() -> serde_json::Value {
    cow_sdk_test_utils::fixtures::fixture("cow_shed/canonical_selectors")
}

/// Independent selector derivation: `sha3::Keccak256`, not alloy's keccak.
fn independent_selector(signature: &str) -> [u8; 4] {
    let digest = Keccak256::digest(signature.as_bytes());
    [digest[0], digest[1], digest[2], digest[3]]
}

fn parse_selector(value: &str) -> [u8; 4] {
    let bytes =
        alloy_primitives::hex::decode(value.trim_start_matches("0x")).expect("selector hex parses");
    let mut out = [0_u8; 4];
    out.copy_from_slice(&bytes);
    out
}

/// Maps a fixture row to the macro-emitted selector constant of the bound
/// function. A fixture row without a binding (or vice versa) is drift and
/// fails loudly.
fn bound_selector(name: &str) -> [u8; 4] {
    match name {
        "executeHooks((address,uint256,bytes,bool,bool)[],bytes32,uint256,address,bytes)" => {
            COWShedFactory::executeHooksCall::SELECTOR
        }
        "initializeProxy(address,bool)" => COWShedFactory::initializeProxyCall::SELECTOR,
        "proxyOf(address)" => COWShedFactory::proxyOfCall::SELECTOR,
        "ownerOf(address)" => COWShedFactory::ownerOfCall::SELECTOR,
        "implementation()" => COWShedFactory::implementationCall::SELECTOR,
        "executeHooks((address,uint256,bytes,bool,bool)[],bytes32,uint256,bytes)" => {
            COWShed::executeHooksCall::SELECTOR
        }
        "trustedExecuteHooks((address,uint256,bytes,bool,bool)[])" => {
            COWShed::trustedExecuteHooksCall::SELECTOR
        }
        "claimWithResolver(address)" => COWShed::claimWithResolverCall::SELECTOR,
        "updateTrustedExecutor(address)" => COWShed::updateTrustedExecutorCall::SELECTOR,
        "updateImplementation(address)" => COWShed::updateImplementationCall::SELECTOR,
        "revokeNonce(bytes32)" => COWShed::revokeNonceCall::SELECTOR,
        "nonces(bytes32)" => COWShed::noncesCall::SELECTOR,
        "domainSeparator()" => COWShed::domainSeparatorCall::SELECTOR,
        "trustedExecutor()" => COWShed::trustedExecutorCall::SELECTOR,
        "VERSION()" => COWShed::VERSIONCall::SELECTOR,
        "initialize(address,bool)" => COWShed::initializeCall::SELECTOR,
        other => {
            panic!("fixture row `{other}` has no bound SolCall — fixture and bindings drifted")
        }
    }
}

#[test]
fn every_selector_row_is_keccak_derived_and_binding_backed() {
    let fixture = canonical_fixture();
    for (group, expected_rows) in [("factory_methods", 5), ("shed_methods", 11)] {
        let rows = fixture[group].as_array().expect("selector group array");
        assert_eq!(
            rows.len(),
            expected_rows,
            "{group} row count pins the bound surface"
        );
        for row in rows {
            let name = row["name"].as_str().expect("row name");
            let pinned = parse_selector(row["selector"].as_str().expect("row selector"));
            assert_eq!(
                independent_selector(name),
                pinned,
                "independent keccak diverges from the fixture for `{name}`"
            );
            assert_eq!(
                bound_selector(name),
                pinned,
                "SolCall::SELECTOR diverges from the fixture for `{name}`"
            );
        }
    }
}

#[test]
fn initialize_proxy_is_two_arg_form() {
    let fixture = canonical_fixture();
    let methods = fixture["factory_methods"]
        .as_array()
        .expect("factory_methods array");
    let two_arg_form = methods
        .iter()
        .find(|row| row["name"].as_str() == Some("initializeProxy(address,bool)"));
    assert!(
        two_arg_form.is_some(),
        "COWShedFactory must declare the 2-arg initializeProxy(address,bool) form per the deployed v1.0.x runtime"
    );
    let one_arg_form = methods
        .iter()
        .find(|row| row["name"].as_str() == Some("initializeProxy(address)"));
    assert!(
        one_arg_form.is_none(),
        "COWShedFactory must NOT declare the ENS-purged 1-arg initializeProxy(address) form; it exists only in the unsupported v2.x generations"
    );
}

#[test]
fn type_strings_have_no_whitespace_between_commas() {
    let fixture = canonical_fixture();
    let type_strings = fixture["type_strings"]
        .as_object()
        .expect("type_strings map");
    assert!(
        !type_strings.is_empty(),
        "type_strings must carry the COW Shed EIP-712 type strings"
    );
    for (name, type_string) in type_strings {
        let type_string = type_string.as_str().expect("type_string is a string");
        assert!(
            !type_string.contains(", "),
            "type string `{name}` must contain no whitespace between commas in declaration order: {type_string}"
        );
    }
}

#[test]
fn eoa_signature_byte_order_is_r_then_s_then_v() {
    let fixture = canonical_fixture();
    let order = fixture["signature_byte_order"]["order"]
        .as_str()
        .expect("signature_byte_order.order");
    assert_eq!(
        order, "r || s || v",
        "COW Shed signed-hook signature must use r-then-s-then-v byte order"
    );
}
