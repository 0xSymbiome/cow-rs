//! Correctness smoke tests for the shared test helpers — they prove the
//! helpers are right, not merely that they compile.

use cow_sdk_core::OrderKind;
use cow_sdk_test_utils::builders::{self, OrderBuilder};
use cow_sdk_test_utils::{consts, eip712, fixtures};

#[test]
fn keccak_oracle_matches_alloy() {
    // The independent sha3-based oracle must agree with alloy's keccak256.
    assert_eq!(
        eip712::keccak256(b"cow"),
        alloy_primitives::keccak256(b"cow").0
    );
    assert_eq!(
        eip712::keccak_word("sell"),
        alloy_primitives::keccak256(b"sell").0
    );
    assert_eq!(
        eip712::keccak_word("erc20"),
        alloy_primitives::keccak256(b"erc20").0
    );
}

#[test]
fn word_encoders_are_correct() {
    let word = eip712::encode_address_word("0x1111111111111111111111111111111111111111");
    assert!(
        word[..12].iter().all(|b| *b == 0),
        "address word is left-padded"
    );
    assert!(
        word[12..].iter().all(|b| *b == 0x11),
        "address occupies the low 20 bytes"
    );

    assert_eq!(eip712::encode_bool_word(true)[31], 1);
    assert_eq!(eip712::encode_bool_word(false)[31], 0);
    assert_eq!(eip712::encode_u32_word(1)[31], 1);
    assert_eq!(eip712::encode_usize_word(1)[31], 1);

    // dual-radix: decimal and 0x-hex of the same value must encode identically.
    assert_eq!(
        eip712::encode_u256_word("255"),
        eip712::encode_u256_word("0xff")
    );
}

#[test]
fn consts_are_canonical() {
    assert_eq!(
        consts::ADDR_A,
        alloy_primitives::Address::from([0x11u8; 20])
    );
    assert_eq!(
        consts::ADDR_D,
        alloy_primitives::Address::from([0x44u8; 20])
    );
    assert!(consts::CID_1.starts_with("f01551b20"));
    assert!(consts::APP_DATA_HEX_1.starts_with("0x337aa6e6"));
    // The CID body is the app-data hash with the multicodec header prepended.
    assert!(consts::CID_1.ends_with(consts::APP_DATA_HEX_1.trim_start_matches("0x")));
}

#[test]
fn fixture_runtime_read_resolves_and_loads() {
    // Proves workspace_root() resolution + the runtime read end-to-end against
    // a real committed fixture.
    let signing = fixtures::fixture("signing");
    assert!(
        signing.is_object(),
        "parity/fixtures/signing.json must load as an object"
    );
}

#[test]
fn order_builder_default_builds_valid_order() {
    // The default (upstream signing vector) must deserialize into a valid OrderData.
    let order = OrderBuilder::default().build();
    assert_eq!(order.kind, OrderKind::Sell);
    assert!(order.partially_fillable);
}

#[test]
fn order_builder_overrides_apply() {
    let order = OrderBuilder::default()
        .kind(OrderKind::Buy)
        .partially_fillable(false)
        .receiver("0x1111111111111111111111111111111111111111")
        .build();
    assert_eq!(order.kind, OrderKind::Buy);
    assert!(!order.partially_fillable);
}

#[test]
fn order_builder_weth_dai_preset_uses_weth_and_dai() {
    let order = OrderBuilder::weth_dai().build();
    assert_eq!(
        order.sell_token.to_hex_string(),
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
    );
    assert_eq!(
        order.buy_token.to_hex_string(),
        "0x6b175474e89094c44da98b954eedeac495271d0f"
    );
    assert_eq!(order.kind, OrderKind::Sell);
}

#[test]
fn sample_domain_and_signature_construct() {
    let _domain = builders::sample_domain();
    let sig = builders::sample_signature_hex(0xaa);
    assert_eq!(sig.len(), 2 + 130, "0x + 65 bytes of hex");
    assert!(sig.ends_with("1b"));
}

#[tokio::test]
async fn recording_signer_logs_calls_and_returns_canned() {
    use cow_sdk_core::Signer;
    use cow_sdk_test_utils::mocks::{RecordingSigner, canned_tx_hash};

    let signer = RecordingSigner::new();
    assert_eq!(
        signer.get_address().await.unwrap().to_hex_string(),
        "0x4444444444444444444444444444444444444444"
    );

    let sig = signer.sign_message(b"hello cow").await.unwrap();
    assert!(sig.ends_with("1b"));
    assert_eq!(signer.calls.borrow().messages, vec![b"hello cow".to_vec()]);

    // The canned broadcast hash has one stable definition.
    assert_eq!(canned_tx_hash(), canned_tx_hash());
}

#[tokio::test]
async fn stub_http_transport_succeeds_with_empty_body() {
    use cow_sdk_core::HttpTransport;
    use cow_sdk_test_utils::mocks::StubHttpTransport;

    let transport = StubHttpTransport;
    assert_eq!(transport.get("/x", &[], None).await.unwrap(), "");
    assert_eq!(transport.post("/x", "body", &[], None).await.unwrap(), "");
}

#[tokio::test]
async fn recording_http_transport_records_requests_and_replays_responses() {
    use cow_sdk_core::HttpTransport;
    use cow_sdk_test_utils::mocks::{Canned, RecordingHttpTransport};

    let transport = RecordingHttpTransport::new([
        Canned::Ok("first".to_owned()),
        Canned::Ok("second".to_owned()),
    ]);
    assert_eq!(
        transport
            .get("/a", &[], Some(std::time::Duration::from_secs(1)))
            .await
            .unwrap(),
        "first"
    );
    assert_eq!(
        transport.post("/b", "body", &[], None).await.unwrap(),
        "second"
    );

    let observed = transport.observed();
    assert_eq!(observed.len(), 2);
    assert_eq!(observed[0].method, "GET");
    assert!(observed[0].has_timeout);
    assert_eq!(observed[1].method, "POST");
    assert_eq!(observed[1].body, "body");
    assert!(!observed[1].has_timeout);
}
