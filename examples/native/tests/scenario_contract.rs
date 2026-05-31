use cow_sdk::orderbook::{ApiContext, ExternalHostPolicy, OrderbookError};
use cow_sdk::prelude::{CowEnv, OrderbookApi, OrderUid, SupportedChainId};
use cow_sdk::trading::OrderbookClient;
use cow_sdk_examples_native::support::{
    MESSAGE_SIGNATURE, MockOrderbook, TYPED_SIGNATURE, orderbook_version_response,
    sample_open_order, sample_order_uid, sample_quote_response, text_preview,
};
use wiremock::{
    Mock, MockServer,
    matchers::{method, path},
};

#[test]
fn text_preview_is_length_safe() {
    assert_eq!(text_preview("0x1234", 18), "0x1234");
    assert_eq!(
        text_preview("abcdefghijklmnopqrst", 18),
        "abcdefghijklmnopqr"
    );
    assert_eq!(text_preview("abc", 0), "");
    assert_eq!(text_preview("ééé", 2), "éé");
}

#[test]
fn signature_fixtures_have_recoverable_ecdsa_shape() {
    for (name, signature) in [
        ("typed signature", TYPED_SIGNATURE),
        ("message signature", MESSAGE_SIGNATURE),
    ] {
        let without_prefix = signature
            .strip_prefix("0x")
            .expect("signature fixture must keep a hex prefix");
        assert_eq!(
            without_prefix.len(),
            130,
            "{name} fixture must be 65 bytes of hex"
        );
    }
}

#[tokio::test]
async fn version_fixture_is_plain_text() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/version"))
        .respond_with(orderbook_version_response("v1.2.3"))
        .mount(&server)
        .await;

    let api = OrderbookApi::builder_from_context(ApiContext::new(
        SupportedChainId::Sepolia,
        CowEnv::Prod,
    ))
    .with_external_host_policy(ExternalHostPolicy::Test)
    .base_url(server.uri())
    .build()
    .expect("test orderbook client with local mock endpoint must build");

    let version = api
        .get_version()
        .await
        .expect("version fixture should load");

    assert_eq!(version, "v1.2.3");
    assert!(!version.contains('"'));
}

#[tokio::test]
async fn mock_order_lookup_is_uid_keyed() {
    let orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sample_quote_response());
    orderbook.push_order(sample_open_order());

    let order = orderbook
        .get_order(&sample_order_uid())
        .await
        .expect("sample order should be found by its uid");

    assert_eq!(order.uid, sample_order_uid());

    let unknown_uid = OrderUid::new(format!("0x{}", "0".repeat(112)))
        .expect("zero order uid fixture should be valid");
    let error = orderbook
        .get_order(&unknown_uid)
        .await
        .expect_err("mock lookup must reject a mismatched order uid");

    match error {
        OrderbookError::InvalidTransform { field, reason: _ } => {
            assert_eq!(field, "orderUid");
        }
        other => panic!("expected InvalidTransform for unknown mock order uid, got {other:?}"),
    }
}

#[test]
fn subgraph_examples_are_declared_and_documented() {
    let manifest = include_str!("../Cargo.toml");
    let readme = include_str!("../README.md");

    for example_name in [
        "subgraph_query_roundtrip",
        "subgraph_custom_query_roundtrip",
        "subgraph_live_query",
    ] {
        assert!(
            manifest.contains(example_name),
            "missing example declaration for {example_name}"
        );
        assert!(
            readme.contains(example_name),
            "missing example README entry for {example_name}"
        );
    }

    assert!(readme.contains("`cow-sdk-subgraph`"));
    assert!(readme.contains("root facade"));
    assert!(readme.contains("THE_GRAPH_API_KEY"));
}

#[test]
fn mandatory_trading_examples_are_declared_and_documented() {
    let manifest = include_str!("../Cargo.toml");
    let native_readme = include_str!("../README.md");
    let examples_readme = include_str!("../../README.md");

    for example_name in [
        "ethflow_transaction_simulation",
        "onchain_order_actions_simulation",
    ] {
        assert!(
            manifest.contains(example_name),
            "missing example declaration for {example_name}"
        );
        assert!(
            native_readme.contains(example_name),
            "missing native README entry for {example_name}"
        );
        assert!(
            examples_readme.contains(example_name),
            "missing examples index entry for {example_name}"
        );
    }

    assert!(native_readme.contains("native-sell / EthFlow"));
    assert!(native_readme.contains("pre-sign"));
    assert!(native_readme.contains("on-chain cancellation"));
}

#[test]
fn mandatory_trading_examples_reference_reviewed_sdk_surfaces() {
    let ethflow = include_str!("../scenarios/ethflow_transaction_simulation.rs");
    let onchain = include_str!("../scenarios/onchain_order_actions_simulation.rs");

    assert!(ethflow.contains("get_eth_flow_transaction"));
    assert!(ethflow.contains("post_sell_native_currency_order"));
    assert!(onchain.contains("get_pre_sign_transaction"));
    assert!(onchain.contains("on_chain_cancel_order"));
    assert!(onchain.contains("onchain_cancellation_transaction"));
}

#[test]
fn transaction_lifecycle_example_is_declared_and_documented() {
    let manifest = include_str!("../Cargo.toml");
    let native_readme = include_str!("../README.md");
    let examples_readme = include_str!("../../README.md");
    let scenario = include_str!("../scenarios/transaction_lifecycle.rs");

    assert!(manifest.contains("transaction_lifecycle"));
    assert!(native_readme.contains("transaction_lifecycle"));
    assert!(examples_readme.contains("transaction_lifecycle"));
    assert!(scenario.contains("receiptRequestsDuringBroadcast"));
    assert!(scenario.contains("send_transaction"));
    assert!(scenario.contains("submit_and_wait_for_receipt"));
    assert!(scenario.contains("Shape A"));
    assert!(scenario.contains("Shape B"));
}
