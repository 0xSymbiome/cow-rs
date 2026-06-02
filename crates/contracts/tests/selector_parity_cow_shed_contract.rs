//! COW Shed selector parity contract test: assert the canonical
//! cow-shed selectors fixture pins the deployed-runtime entry-point
//! selectors and EIP-712 type hashes. Source authority is the
//! cow-sdk TypeScript ABI L1 (pinned cow-sdk SHA
//! `74393ee2923a2932584998169daca6ce3c2da60c`).

fn canonical_fixture() -> serde_json::Value {
    cow_sdk_test_utils::fixtures::manifest_fixture(
        env!("CARGO_MANIFEST_DIR"),
        "tests/fixtures/cow_shed_canonical_selectors.json",
    )
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
        "COWShedFactory must declare the 2-arg initializeProxy(address,bool) form sourced from cow-sdk TS ABI L1 authority"
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
fn type_hashes_have_no_whitespace_between_commas() {
    let fixture = canonical_fixture();
    let type_hashes = fixture["type_hashes"]
        .as_array()
        .expect("type_hashes array");
    for row in type_hashes {
        let type_string = row["type_string"].as_str().expect("type_string");
        assert!(
            !type_string.contains(", "),
            "type string `{type_string}` must contain no whitespace between commas in declaration order"
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
