use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

use cow_sdk_verification_console::{
    app_data_report_json, app_data_schema_json, approval_transaction_preview_json,
    capability_report_json, cid_from_hex_json, eip1271_payload_preview_json, hex_from_cid_json,
    order_envelope_preview_json, supported_chains_json, trading_defaults_json,
};

wasm_bindgen_test_configure!(run_in_browser);

const OWNER: &str = "0x4444444444444444444444444444444444444444";
const MAINNET_WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
const MAINNET_USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

#[wasm_bindgen_test]
fn property_0_exports_are_callable_and_constants_are_present() {
    let chains = parse_json(supported_chains_json());
    let mainnet = chains
        .as_array()
        .expect("supported chains must be an array")
        .iter()
        .find(|chain| chain["chainId"] == 1)
        .expect("mainnet must be listed");
    assert_eq!(mainnet["apiPath"], "mainnet");
    assert_address_eq(&mainnet["wrappedNative"]["address"], MAINNET_WETH);

    let capability = parse_json(capability_report_json(1, "prod"));
    assert_eq!(capability["surface"], "cow-sdk");
    assert_eq!(capability["mode"], "wasm-console");
    assert_eq!(capability["chainId"], 1);
    assert_eq!(capability["sdkConstructed"], true);
    assert_address_eq(&capability["wrappedNative"]["address"], MAINNET_WETH);
    assert_address_eq(&capability["sampleOrder"]["sellToken"], MAINNET_WETH);
    assert_address_eq(&capability["sampleOrder"]["buyToken"], MAINNET_USDC);
    assert!(
        capability["sampleOrderNotes"]["buyToken"]
            .as_str()
            .expect("buy token note must be a string")
            .contains("Static USDC")
    );
    assert!(
        capability["deployment"]["settlement"]
            .as_str()
            .expect("settlement address must be a string")
            .starts_with("0x")
    );

    let defaults = parse_json(trading_defaults_json());
    assert_eq!(defaults["quoteValiditySeconds"], 1800);
    assert_eq!(defaults["defaultSlippageBps"], 50);
    assert_eq!(defaults["maxSlippageBps"], 10000);
}

#[wasm_bindgen_test]
fn app_data_and_cid_exports_round_trip_deterministically() {
    let app_data = r#"{
        "version": "1.14.0",
        "appCode": "cow-rs/wasm-console",
        "environment": "browser",
        "metadata": {
          "quote": {
            "slippageBips": 50
          }
        }
      }"#;

    let report = parse_json(app_data_report_json(app_data));
    assert_eq!(report["valid"], true);
    let cid = report["cid"].as_str().expect("cid must be a string");
    let app_data_hex = report["appDataHex"]
        .as_str()
        .expect("appDataHex must be a string");

    let from_cid = parse_json(hex_from_cid_json(cid));
    assert_eq!(from_cid["appDataHex"], app_data_hex);

    let from_hex = parse_json(cid_from_hex_json(app_data_hex));
    assert_eq!(from_hex["cid"], cid);
}

#[wasm_bindgen_test]
fn order_envelope_and_approval_exports_produce_reviewable_json() {
    let order = r#"{
        "sellToken": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        "buyToken": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "receiver": "0x4444444444444444444444444444444444444444",
        "sellAmount": "100000000000000000",
        "buyAmount": "250000000",
        "validTo": 1900000000,
        "appData": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "feeAmount": "0",
        "kind": "sell",
        "partiallyFillable": false,
        "sellTokenBalance": "erc20",
        "buyTokenBalance": "erc20"
      }"#;
    let envelope = parse_json(order_envelope_preview_json(1, order, OWNER));
    assert_eq!(envelope["primaryType"], "Order");
    assert_eq!(envelope["expectedPrimaryType"], "Order");
    assert_eq!(envelope["domain"]["chainId"], 1);
    assert!(
        envelope["orderId"]
            .as_str()
            .expect("order id must be a string")
            .starts_with("0x")
    );

    let approval = r#"{
        "tokenAddress": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        "amount": "115792089237316195423570985008687907853269984665640564039457584007913129639935"
      }"#;
    let approval = parse_json(approval_transaction_preview_json(1, "prod", approval));
    assert_eq!(approval["chainId"], 1);
    assert_eq!(approval["gas"]["defaultLimit"], 150000);
    assert_eq!(approval["transaction"]["to"], MAINNET_WETH);
}

#[wasm_bindgen_test]
fn malformed_deterministic_input_fails_visibly() {
    let error = app_data_report_json("{").expect_err("malformed app-data JSON must fail");
    let message = error_message(error);
    assert!(message.contains("invalid appDataDoc JSON"));
}

/// Property 4 — trading defaults expose the reviewed slippage and quote-validity bounds.
#[wasm_bindgen_test]
fn trading_defaults_expose_reviewed_slippage_and_quote_validity_bounds() {
    let defaults = parse_json(trading_defaults_json());

    let default_bps = defaults["defaultSlippageBps"]
        .as_u64()
        .expect("defaultSlippageBps must be a number");
    assert!(
        (50..=1000).contains(&default_bps),
        "default slippage must stay inside the reviewed 50..=1000 bps band, got {default_bps}"
    );

    let max_bps = defaults["maxSlippageBps"]
        .as_u64()
        .expect("maxSlippageBps must be a number");
    assert!(max_bps >= default_bps, "max slippage must be >= default");
    assert_eq!(
        max_bps, 10_000,
        "max slippage stays at 100% per the reviewed contract"
    );

    let validity = defaults["quoteValiditySeconds"]
        .as_u64()
        .expect("quoteValiditySeconds must be a number");
    assert!(
        (60..=3_600).contains(&validity),
        "quote validity must sit between 1 minute and 1 hour, got {validity}s"
    );

    let ethflow_bps = defaults["ethflowFloorSlippageBps"]
        .as_u64()
        .expect("ethflowFloorSlippageBps must be a number");
    assert!(
        ethflow_bps >= default_bps,
        "ethflow floor slippage must not fall below the default slippage band"
    );
}

/// Property 5 — the EIP-1271 payload preview captures the generated payload for the provided order.
#[wasm_bindgen_test]
fn eip1271_payload_preview_captures_signature_payload_for_reviewed_order() {
    let order = reviewed_order_json();
    let sample_signature = format!("0x{}", "02".repeat(65));
    let preview = parse_json(eip1271_payload_preview_json(order, &sample_signature));

    let payload = preview["payload"]
        .as_str()
        .expect("eip1271 payload must be a string");
    assert!(
        payload.starts_with("0x"),
        "eip1271 payload must be a 0x-prefixed hex blob"
    );
    let body = payload
        .strip_prefix("0x")
        .expect("eip1271 payload must start with 0x");
    assert!(
        !body.is_empty() && body.len() % 2 == 0,
        "eip1271 payload must encode a non-empty even-length byte sequence"
    );
    assert!(
        body.chars().all(|c| c.is_ascii_hexdigit()),
        "eip1271 payload must contain only ascii hex digits"
    );
}

/// Property 6 — approval preview emits an ABI-encoded full-range max uint for the selected token.
#[wasm_bindgen_test]
fn approval_preview_emits_full_range_max_uint_calldata() {
    let approval = r#"{
        "tokenAddress": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        "amount": "115792089237316195423570985008687907853269984665640564039457584007913129639935"
      }"#;
    let preview = parse_json(approval_transaction_preview_json(1, "prod", approval));

    assert_eq!(preview["chainId"], 1);
    assert_eq!(preview["transaction"]["to"], MAINNET_WETH);

    let data = preview["transaction"]["data"]
        .as_str()
        .expect("approval transaction data must be a hex string");
    assert!(
        data.starts_with("0x"),
        "approval calldata must be 0x-prefixed"
    );
    assert!(
        data.ends_with(&"f".repeat(64)),
        "max-uint amount must encode the full unsigned uint256 word at the tail of the calldata"
    );

    let default_limit = preview["gas"]["defaultLimit"]
        .as_u64()
        .expect("default gas limit must be a number");
    assert!(
        default_limit >= 60_000,
        "gas default must cover an ERC-20 approve call, got {default_limit}"
    );
}

/// Property 7 — order envelope preview emits a conforming EIP-712 typed-data structure.
#[wasm_bindgen_test]
fn order_envelope_preview_emits_conforming_eip712_typed_data() {
    let envelope = parse_json(order_envelope_preview_json(1, reviewed_order_json(), OWNER));

    assert_eq!(envelope["primaryType"], "Order");

    let domain = &envelope["domain"];
    assert_eq!(domain["chainId"], 1);
    assert!(
        domain["verifyingContract"]
            .as_str()
            .is_some_and(|contract| contract.starts_with("0x")),
        "domain must carry the settlement verifying contract as 0x-prefixed hex"
    );
    assert!(
        domain["name"].is_string(),
        "typed-data domain must include a protocol name"
    );
    assert!(
        domain["version"].is_string(),
        "typed-data domain must include a version"
    );

    let types = &envelope["types"];
    assert!(
        types["Order"].is_array(),
        "Order type definition must be a typed-data field array"
    );

    let message = &envelope["message"];
    assert_eq!(message["sellToken"], MAINNET_WETH);
    assert_eq!(message["buyToken"], MAINNET_USDC);
}

/// Property 8 — app-data schema inspection surfaces the reviewed schema fields for a canonical document.
#[wasm_bindgen_test]
fn app_data_schema_surfaces_reviewed_schema_fields() {
    let doc = r#"{
        "version": "1.14.0",
        "appCode": "cow-rs/wasm-console",
        "environment": "browser",
        "metadata": {
          "quote": { "slippageBips": 50 }
        }
      }"#;
    let schema = parse_json(app_data_schema_json(doc));

    assert!(
        schema.is_object(),
        "app-data schema inspection must return a reviewable JSON object"
    );
    let fields = schema
        .as_object()
        .expect("app-data schema inspection must expose fields");
    assert!(
        !fields.is_empty(),
        "app-data schema inspection must not return an empty object"
    );
}

/// Property 9 — capability report stays coherent across supported chain and env pairings.
#[wasm_bindgen_test]
fn capability_report_holds_across_supported_chains_and_envs() {
    for (chain_id, env) in [(1_u32, "prod"), (11_155_111_u32, "staging")] {
        let report = parse_json(capability_report_json(chain_id, env));
        assert_eq!(report["surface"], "cow-sdk");
        assert_eq!(report["mode"], "wasm-console");
        assert_eq!(report["chainId"], chain_id);
        assert_eq!(report["sdkConstructed"], true);
        assert!(
            report["wrappedNative"]["address"]
                .as_str()
                .is_some_and(|address| address.starts_with("0x")),
            "wrapped native address must be 0x-prefixed for chain {chain_id}"
        );
        assert!(
            report["deployment"]["settlement"]
                .as_str()
                .is_some_and(|address| address.starts_with("0x")),
            "settlement address must be 0x-prefixed for chain {chain_id}"
        );
    }
}

fn reviewed_order_json() -> &'static str {
    r#"{
        "sellToken": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        "buyToken": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "receiver": "0x4444444444444444444444444444444444444444",
        "sellAmount": "100000000000000000",
        "buyAmount": "250000000",
        "validTo": 1900000000,
        "appData": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "feeAmount": "0",
        "kind": "sell",
        "partiallyFillable": false,
        "sellTokenBalance": "erc20",
        "buyTokenBalance": "erc20"
      }"#
}

fn parse_json(result: Result<String, JsValue>) -> Value {
    serde_json::from_str(&result.expect("export must return JSON"))
        .expect("export must return valid JSON")
}

fn error_message(error: JsValue) -> String {
    error
        .as_string()
        .expect("console errors must be string values")
}

/// Asserts that a JSON address value matches the expected hex literal without
/// enforcing a specific ASCII case. Protocol-constant tables are emitted as
/// lowercase byte arrays, so consumers must treat addresses as
/// case-insensitive hex blobs.
fn assert_address_eq(actual: &Value, expected: &str) {
    let actual = actual
        .as_str()
        .expect("address value must be a JSON string");
    assert!(
        actual.eq_ignore_ascii_case(expected),
        "address `{actual}` does not match expected `{expected}` (case-insensitive)"
    );
}
