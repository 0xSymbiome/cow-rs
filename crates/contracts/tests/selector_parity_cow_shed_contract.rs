//! COW Shed selector parity contract test: assert the canonical
//! cow-shed selectors fixture pins the deployed-runtime entry-point
//! selectors and EIP-712 type strings. The authority is the deployed
//! `COWShedFactory` v1.0.1 runtime interface (verifiable on-chain; each
//! selector is keccak256 of the deployed signature). The deployed
//! 2-arg `initializeProxy(address,bool)` diverges from the cow-shed
//! source-HEAD 1-arg form, so the record is anchored to the deployed
//! runtime rather than any source checkout. The canonical EIP-712
//! type-hash values are pinned by `cow_shed/execute_hooks_digest.json`.

fn canonical_fixture() -> serde_json::Value {
    cow_sdk_test_utils::fixtures::fixture("cow_shed/canonical_selectors")
}

fn lookup_factory_selector(fixture: &serde_json::Value, name: &str) -> String {
    cow_sdk_test_utils::fixtures::row_by_name(fixture, "factory_methods", name)["selector"]
        .as_str()
        .expect("selector must be a string")
        .to_string()
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
        "COWShedFactory must declare the 2-arg initializeProxy(address,bool) form per the deployed v1.0.1 runtime (it diverges from the source-HEAD 1-arg form)"
    );
    let one_arg_form = methods
        .iter()
        .find(|row| row["name"].as_str() == Some("initializeProxy(address)"));
    assert!(
        one_arg_form.is_none(),
        "COWShedFactory must NOT declare the 1-arg source-HEAD initializeProxy(address) form; the deployed bytecode targets the 2-arg selector"
    );
}

#[test]
fn execute_hooks_selector_pinned() {
    let fixture = canonical_fixture();
    let selector = lookup_factory_selector(
        &fixture,
        "executeHooks((address,uint256,bytes,bool,bool)[],bytes32,uint256,address,bytes)",
    );
    assert_eq!(
        selector, "0x46d2f7a9",
        "executeHooks selector must match the deployed-runtime entry point"
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
fn forwarder_is_valid_signature_selector_pinned() {
    let fixture = canonical_fixture();
    let selector = cow_sdk_test_utils::fixtures::row_by_name(
        &fixture,
        "forwarder_methods",
        "isValidSignature(bytes32,bytes)",
    )["selector"]
        .as_str()
        .expect("selector must be a string")
        .to_string();
    assert_eq!(
        selector, "0x1626ba7e",
        "ERC1271Forwarder must expose the canonical ERC-1271 isValidSignature selector"
    );
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
