use cow_sdk::orderbook::OrderbookError;
use cow_sdk::trading::OrderbookClient;
use cow_sdk::{ApiContext, CowEnv, OrderBookApi, OrderUid, SupportedChainId};
use cow_sdk_examples_native::support::{
    MockOrderbook, orderbook_version_response, sample_open_order, sample_order_uid,
    sample_quote_response, text_preview,
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

#[tokio::test]
async fn version_fixture_is_plain_text() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/version"))
        .respond_with(orderbook_version_response("v1.2.3"))
        .mount(&server)
        .await;

    let api = OrderBookApi::new_with_base_url(
        ApiContext {
            chain_id: SupportedChainId::Sepolia,
            env: CowEnv::Prod,
            base_urls: None,
            api_key: None,
        },
        server.uri(),
    );

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
        OrderbookError::InvalidTransform(message) => {
            assert!(message.contains(unknown_uid.as_str()));
        }
        other => panic!("expected InvalidTransform for unknown mock order uid, got {other:?}"),
    }
}
