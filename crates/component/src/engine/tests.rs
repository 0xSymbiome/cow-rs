// The UID is `digest (32 bytes) || owner (20) || validTo (4)`, so its first
// 32 bytes are exactly the order digest. Pinning that invariant exercises
// both deterministic entry points the component exports.
const ORDER_JSON: &str = r#"{
    "sellToken": "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
    "buyToken": "0x0625afb445c3b6b7b929342a04a22599fd5dbb59",
    "receiver": "0x2222222222222222222222222222222222222222",
    "sellAmount": "1000000000000000",
    "buyAmount": "1000000000000000000",
    "feeAmount": "0",
    "validTo": 2000000000,
    "appData": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "kind": "sell",
    "partiallyFillable": false,
    "sellTokenBalance": "erc20",
    "buyTokenBalance": "erc20"
}"#;
const OWNER: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";

#[test]
fn uid_embeds_the_order_digest() {
    let uid = super::compute_uid(11_155_111, OWNER, ORDER_JSON).expect("uid");
    let digest = super::compute_digest(11_155_111, ORDER_JSON).expect("digest");

    assert_eq!(uid.len(), 2 + 112, "uid is 0x + 56 bytes");
    assert_eq!(digest.len(), 2 + 64, "digest is 0x + 32 bytes");
    // The digest is the UID's leading 32 bytes.
    assert_eq!(uid[2..66], digest[2..66]);
}

#[test]
fn tx_helpers_resolve_targets_and_encode_canonical_calls() {
    use cow_sdk_contracts::{ContractId, Registry};
    use cow_sdk_core::{CowEnv, SupportedChainId, wrapped_native_token};

    let weth = wrapped_native_token(SupportedChainId::Mainnet)
        .address
        .to_hex_string();
    let settlement = Registry::default()
        .address(
            ContractId::Settlement,
            SupportedChainId::Mainnet,
            CowEnv::Prod,
        )
        .expect("settlement registered for mainnet")
        .to_hex_string();
    let eth_flow = Registry::default()
        .address(ContractId::EthFlow, SupportedChainId::Mainnet, CowEnv::Prod)
        .expect("eth-flow registered for mainnet")
        .to_hex_string();

    // wrap: target the wrapped-native token, deposit() selector, amount as value.
    let (to, data, value) = super::tx::wrap(1, "1000").expect("wrap");
    assert_eq!(to, weth);
    assert!(data.starts_with("0xd0e30db0"), "deposit()");
    assert_eq!(value, "1000");

    // unwrap: target the wrapped-native token, withdraw(uint256) selector, zero value.
    let (to, data, value) = super::tx::unwrap(1, "1000").expect("unwrap");
    assert_eq!(to, weth);
    assert!(data.starts_with("0x2e1a7d4d"), "withdraw(uint256)");
    assert_eq!(value, "0");

    // approve: target the token, approve(address,uint256) selector, zero value.
    let token = format!("0x{}", "33".repeat(20));
    let spender = format!("0x{}", "44".repeat(20));
    let (to, data, value) =
        super::tx::approve(1, &token, "5", Some(&spender), None).expect("approve");
    assert_eq!(to, token);
    assert!(data.starts_with("0x095ea7b3"), "approve(address,uint256)");
    assert_eq!(value, "0");

    // pre-sign and cancel: target the settlement contract, zero value.
    let uid = format!("0x{}", "11".repeat(56));
    let (to, _, value) = super::tx::pre_sign(1, &uid, None).expect("pre-sign");
    assert_eq!(to, settlement);
    assert_eq!(value, "0");
    let (to, _, value) = super::tx::cancel(1, &uid, None).expect("cancel");
    assert_eq!(to, settlement);
    assert_eq!(value, "0");

    // sell-native and cancel-native: target the eth-flow contract.
    let (to, _, _) = super::tx::sell_native(1, ORDER_JSON, 7, None).expect("sell-native");
    assert_eq!(to, eth_flow);
    let (to, _, value) = super::tx::cancel_native(1, ORDER_JSON, 7, None).expect("cancel-native");
    assert_eq!(to, eth_flow);
    assert_eq!(value, "0", "eth-flow cancellation sends no value");
}

#[test]
fn order_signing_payloads_are_canonical() {
    // The typed-data envelope names the Order primary type and carries the
    // domain, types, and message a wallet signs.
    let typed = super::signing::order_typed_data(11_155_111, ORDER_JSON).expect("order typed data");
    let value: serde_json::Value = serde_json::from_str(&typed).expect("typed data is json");
    assert_eq!(value["primaryType"], "Order");
    assert!(value["domain"].is_object());
    assert!(value["types"]["Order"].is_array());
    assert!(value["message"]["sellToken"].is_string());

    // generate-order-id agrees with the standalone uid / digest entry points.
    let (uid, digest) =
        super::signing::generate_order_id(11_155_111, OWNER, ORDER_JSON).expect("order id");
    assert_eq!(
        uid,
        super::compute_uid(11_155_111, OWNER, ORDER_JSON).expect("uid")
    );
    assert_eq!(
        digest,
        super::compute_digest(11_155_111, ORDER_JSON).expect("digest")
    );
    assert_eq!(uid[2..66], digest[2..66]);

    // The EIP-1271 wrapper turns a 65-byte ECDSA signature into a longer
    // verifier-prefixed `abi.encode(order, signature)` payload.
    let signature = format!("0x{}{}1b", "11".repeat(32), "22".repeat(32));
    let payload = super::signing::eip1271_signature_payload(ORDER_JSON, &signature)
        .expect("eip-1271 payload");
    assert!(payload.starts_with("0x"));
    assert!(
        payload.len() > signature.len(),
        "the wrapped payload is longer than the raw signature",
    );

    // The cancellation envelope names the OrderCancellations primary type.
    let cancel =
        super::signing::cancellations_typed_data(11_155_111, &[format!("0x{}", "11".repeat(56))])
            .expect("cancellation typed data");
    let cancel_value: serde_json::Value =
        serde_json::from_str(&cancel).expect("cancellation is json");
    assert_eq!(cancel_value["primaryType"], "OrderCancellations");
}

#[test]
fn twap_encoding_targets_composablecow_and_classifies_schedule() {
    use cow_sdk_contracts::composable::{
        COMPOSABLE_COW, TwapDurationOfPart, TwapStartTime, TwapTiming,
    };

    let composable_cow = COMPOSABLE_COW.to_hex_string();
    let sell = "0xfff9976782d46cc05630d1f6ebab18b2324d6b14";
    let buy = "0x0625afb445c3b6b7b929342a04a22599fd5dbb59";
    let app_data = format!("0x{}", "00".repeat(32));
    let salt = format!("0x{}", "11".repeat(32));

    // A 4-part TWAP starting at a fixed epoch, each part valid for its whole interval.
    let twap = super::composable::build_twap(
        sell,
        buy,
        None,
        "4000000000000000",
        "400000000000000000",
        4,
        3600,
        TwapStartTime::AtEpoch(1_000_000),
        TwapDurationOfPart::Auto,
        &app_data,
    )
    .expect("twap builds");

    // create: targets ComposableCoW, zero value, non-empty calldata.
    let (to, data, value) = super::composable::create_transaction(&twap, &salt).expect("create tx");
    assert_eq!(to, composable_cow);
    assert!(
        data.starts_with("0x") && data.len() > 2,
        "non-empty calldata"
    );
    assert_eq!(value, "0");

    // order-id is a deterministic 0x + 32 bytes; remove targets ComposableCoW.
    let id = super::composable::order_id(&twap, &salt).expect("order id");
    assert_eq!(id.len(), 2 + 64);
    let (to, _, value) = super::composable::remove_transaction(&id).expect("remove tx");
    assert_eq!(to, composable_cow);
    assert_eq!(value, "0");

    // timing classifies the schedule from its resolved start (the at-epoch t0).
    // The schedule ends at start + n * t = 1_000_000 + 4 * 3600.
    let start = 1_000_000;
    assert!(matches!(
        super::composable::timing_at(&twap, start, start - 1).expect("timing"),
        TwapTiming::NotStarted { .. }
    ));
    assert!(matches!(
        super::composable::timing_at(&twap, start, start + 3600 + 10).expect("timing"),
        TwapTiming::Active { part: 1, .. }
    ));
    assert!(matches!(
        super::composable::timing_at(&twap, start, start + 4 * 3600).expect("timing"),
        TwapTiming::Expired
    ));
}

#[test]
fn event_decoding_is_wired_and_fails_closed() {
    let zero_topic = format!("0x{}", "00".repeat(32));
    // A log whose topic-0 matches no known event is rejected, never panicked
    // on — the borrowed-bytes path and the fail-closed decoder are wired.
    assert!(super::events::settlement(std::slice::from_ref(&zero_topic), "0x").is_err());
    assert!(super::events::eth_flow(&[zero_topic], "0x").is_err());
    // Malformed topics and data are rejected before decoding.
    assert!(super::events::settlement(&["not-hex".to_owned()], "0x").is_err());
    assert!(super::events::settlement(&[], "zz").is_err());
}
