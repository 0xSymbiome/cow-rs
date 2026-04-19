//! Property-based coverage for the deterministic `cow-sdk-contracts` boundary.
//!
//! Each `proptest!` case exercises a named invariant on one of the
//! order-hashing, UID-packing, trade-encoding, flag-codec, signature-
//! codec, or signing-scheme helpers. Shrinking narrows any counter-
//! example before `cargo test` prints it, and committed seed files under
//! `tests/proptest-regressions/` keep the shrink outcomes reproducible
//! across contributors. Net coverage matches the hand-rolled enumerator
//! this file replaced: every invariant family the enumerator exercised
//! carries a named property here.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::missing_const_for_fn,
    clippy::redundant_clone,
    clippy::redundant_closure,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    reason = "pedantic, nursery, and style lints acceptable in test helper code"
)]

use cow_sdk_contracts::{
    Eip1271SignatureData, Order, OrderFlags, OrderUidParams, Signature, SigningScheme,
    TokenRegistry, TradeExecution, TradeFlags, compute_order_uid, decode_eip1271_signature_data,
    decode_order, decode_order_flags, decode_signing_scheme, decode_trade_flags,
    encode_eip1271_signature_data, encode_order_flags, encode_signing_scheme, encode_trade,
    encode_trade_flags, extract_order_uid_params, hash_order, normalize_order,
    normalized_ecdsa_signature, pack_order_uid_params,
};
use cow_sdk_core::{
    Address, Amount, AppDataHex, OrderBalance, OrderDigest, OrderKind, TypedDataDomain,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

/// Path for committed regression seeds; proptest writes new shrink
/// outcomes here so every contributor re-runs prior counter-examples
/// before any novel case is generated.
const REGRESSION_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/proptest-regressions/property_contract.txt"
);

/// Documented trade-signature boundary lengths that exercise the ABI
/// offset, length, and tail padding of [`encode_eip1271_signature_data`].
const SIGNATURE_BOUNDARY_LENGTHS: [usize; 18] = [
    0, 1, 2, 15, 16, 31, 32, 33, 47, 48, 63, 64, 65, 95, 96, 97, 127, 128,
];

/// Applies the upstream buy-balance normalization rule used inside the
/// settlement encoder and mirrored in the compact flag codec: `Internal`
/// stays `Internal`; `Erc20`/`External` collapse to `Erc20`.
fn canonical_buy_balance(balance: OrderBalance) -> OrderBalance {
    match balance {
        OrderBalance::Internal => OrderBalance::Internal,
        OrderBalance::Erc20 | OrderBalance::External => OrderBalance::Erc20,
    }
}

/// Renders the hex encoding of `bytes` with per-nibble casing drawn from
/// `casing` so shrinking can isolate casing-sensitive failures.
fn render_mixed_case(bytes: &[u8], casing: &[bool]) -> String {
    debug_assert_eq!(bytes.len() * 2, casing.len());
    let mut out = String::with_capacity(bytes.len() * 2 + 2);
    out.push_str("0x");
    for (index, byte) in bytes.iter().enumerate() {
        let hi = byte >> 4;
        let lo = byte & 0x0F;
        out.push(nibble_char(hi, casing[index * 2]));
        out.push(nibble_char(lo, casing[index * 2 + 1]));
    }
    out
}

fn nibble_char(value: u8, uppercase: bool) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 if uppercase => (b'A' + value - 10) as char,
        10..=15 => (b'a' + value - 10) as char,
        _ => unreachable!("a nibble value always fits in four bits"),
    }
}

/// Strategy that emits an address with a non-zero low byte so downstream
/// helpers never see a canonical-zero address boundary they already
/// reject outside of the property under test.
fn address_strategy() -> impl Strategy<Value = Address> {
    any::<[u8; 20]>().prop_map(|mut bytes| {
        if bytes.iter().all(|byte| *byte == 0) {
            bytes[19] = 1;
        }
        Address::new(format!("0x{}", hex::encode(bytes))).unwrap()
    })
}

/// Strategy that emits a 32-byte order digest wrapped in [`OrderDigest`].
fn order_digest_strategy() -> impl Strategy<Value = OrderDigest> {
    any::<[u8; 32]>()
        .prop_map(|bytes| OrderDigest::new(format!("0x{}", hex::encode(bytes))).unwrap())
}

/// Strategy that emits an [`AppDataHex`] payload.
fn app_data_strategy() -> impl Strategy<Value = AppDataHex> {
    any::<[u8; 32]>()
        .prop_map(|bytes| AppDataHex::new(format!("0x{}", hex::encode(bytes))).unwrap())
}

/// Strategy that emits an [`Amount`] with at least one non-zero byte so
/// order-hashing inputs stay outside the all-zero boundary.
fn amount_strategy() -> impl Strategy<Value = Amount> {
    any::<[u8; 32]>().prop_map(|mut bytes| {
        if bytes.iter().all(|byte| *byte == 0) {
            bytes[31] = 1;
        }
        Amount::new(format!("0x{}", hex::encode(bytes))).unwrap()
    })
}

/// Strategy that emits every supported [`SigningScheme`] variant.
fn signing_scheme_strategy() -> impl Strategy<Value = SigningScheme> {
    prop_oneof![
        Just(SigningScheme::Eip712),
        Just(SigningScheme::EthSign),
        Just(SigningScheme::Eip1271),
        Just(SigningScheme::PreSign),
    ]
}

/// Strategy that emits every `sell_token_balance` shape the reviewed
/// order contract admits.
fn sell_balance_strategy() -> impl Strategy<Value = OrderBalance> {
    prop_oneof![
        Just(OrderBalance::Erc20),
        Just(OrderBalance::External),
        Just(OrderBalance::Internal),
    ]
}

/// Strategy that emits every `buy_token_balance` shape the reviewed
/// order contract admits. The codec collapses `External` to `Erc20` on
/// encode; this strategy emits all three so the normalization path is
/// exercised.
fn buy_balance_strategy() -> impl Strategy<Value = OrderBalance> {
    prop_oneof![
        Just(OrderBalance::Erc20),
        Just(OrderBalance::External),
        Just(OrderBalance::Internal),
    ]
}

/// Strategy that emits a deterministic order suitable for hashing and
/// trade encoding. All typed fields are drawn from their reviewed
/// domains; the balance fields cycle through every admitted variant.
fn order_strategy() -> impl Strategy<Value = Order> {
    (
        address_strategy(),
        address_strategy(),
        address_strategy(),
        amount_strategy(),
        amount_strategy(),
        any::<u32>(),
        app_data_strategy(),
        amount_strategy(),
        any::<bool>(),
        any::<bool>(),
        sell_balance_strategy(),
        buy_balance_strategy(),
    )
        .prop_map(
            |(
                sell_token,
                buy_token,
                receiver,
                sell_amount,
                buy_amount,
                valid_to,
                app_data,
                fee_amount,
                kind_sell,
                partially_fillable,
                sell_token_balance,
                buy_token_balance,
            )| {
                Order {
                    sell_token,
                    buy_token,
                    receiver: Some(receiver),
                    sell_amount,
                    buy_amount,
                    valid_to,
                    app_data,
                    fee_amount,
                    kind: if kind_sell {
                        OrderKind::Sell
                    } else {
                        OrderKind::Buy
                    },
                    partially_fillable,
                    sell_token_balance: Some(sell_token_balance),
                    buy_token_balance: Some(buy_token_balance),
                }
            },
        )
}

/// Strategy that emits an order plus an equivalent order under the
/// reviewed balance-normalization rule. `sell_token_balance == Erc20`
/// may become `None` on the equivalent side; `buy_token_balance` is
/// reshuffled among the collapsing group
/// `{None, Some(Erc20), Some(External)}` while `Some(Internal)` stays
/// pinned. The two orders must produce byte-identical
/// [`normalize_order`] output and therefore the same [`hash_order`]
/// digest under a shared typed-data domain.
fn equivalent_order_pair_strategy() -> impl Strategy<Value = (Order, Order)> {
    order_strategy().prop_flat_map(|order| {
        let sell_is_erc20 = order.sell_token_balance == Some(OrderBalance::Erc20);
        let buy_is_internal = order.buy_token_balance == Some(OrderBalance::Internal);
        (
            any::<bool>(),
            prop_oneof![
                Just(None),
                Just(Some(OrderBalance::Erc20)),
                Just(Some(OrderBalance::External)),
            ],
        )
            .prop_map(move |(erase_sell, buy_selector)| {
                let mut equivalent = order.clone();
                if sell_is_erc20 && erase_sell {
                    equivalent.sell_token_balance = None;
                }
                if !buy_is_internal {
                    equivalent.buy_token_balance = buy_selector;
                }
                (order.clone(), equivalent)
            })
    })
}

/// Strategy that emits a typed-data domain whose `verifying_contract`
/// is a non-zero address. The name and version fields stay fixed so
/// the reviewed domain shape stays close to the shipped settlement
/// domain.
fn domain_strategy() -> impl Strategy<Value = TypedDataDomain> {
    (address_strategy(), 1u64..=100_000u64).prop_map(|(verifying_contract, chain_id)| {
        TypedDataDomain {
            name: "Gnosis Protocol".to_owned(),
            version: "v2".to_owned(),
            chain_id,
            verifying_contract,
        }
    })
}

/// Strategy that emits a `(scheme, signature)` pair where the payload
/// matches the reviewed shape for the scheme: ECDSA schemes carry a
/// 65-byte body, EIP-1271 carries a verifier address plus a 65-byte
/// body, and pre-sign carries just the owner address.
fn scheme_and_signature_strategy() -> impl Strategy<Value = (SigningScheme, Signature)> {
    signing_scheme_strategy().prop_flat_map(|scheme| match scheme {
        SigningScheme::Eip712 | SigningScheme::EthSign => any::<[u8; 65]>()
            .prop_map(move |bytes| {
                (
                    scheme,
                    Signature::Ecdsa {
                        scheme,
                        data: format!("0x{}", hex::encode(bytes)),
                    },
                )
            })
            .boxed(),
        SigningScheme::Eip1271 => (address_strategy(), any::<[u8; 65]>())
            .prop_map(move |(verifier, bytes)| {
                (
                    scheme,
                    Signature::Eip1271 {
                        data: Eip1271SignatureData {
                            verifier,
                            signature: format!("0x{}", hex::encode(bytes)),
                        },
                    },
                )
            })
            .boxed(),
        SigningScheme::PreSign => address_strategy()
            .prop_map(move |owner| (scheme, Signature::PreSign { owner }))
            .boxed(),
        _ => unreachable!("signing_scheme_strategy emits only the four reviewed variants"),
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSION_FILE))),
        ..ProptestConfig::default()
    })]

    /// [`pack_order_uid_params`] and [`extract_order_uid_params`] are
    /// strict inverses: packing a `(digest, owner, valid_to)` triple and
    /// extracting the UID returns the same components byte-for-byte.
    #[test]
    fn order_uid_pack_extract_roundtrip(
        digest in order_digest_strategy(),
        owner in address_strategy(),
        valid_to in any::<u32>(),
    ) {
        let params = OrderUidParams {
            order_digest: digest.clone(),
            owner: owner.clone(),
            valid_to,
        };
        let uid = pack_order_uid_params(&params).unwrap();
        let extracted = extract_order_uid_params(&uid).unwrap();

        prop_assert_eq!(extracted.order_digest.as_str(), digest.as_str());
        prop_assert_eq!(extracted.owner.as_str(), owner.as_str());
        prop_assert_eq!(extracted.valid_to, valid_to);
    }

    /// [`compute_order_uid`] is deterministic: the same `(domain, order,
    /// owner)` always produces the same UID, and the UID extracts back to
    /// the same owner and `valid_to` the order carried.
    #[test]
    fn compute_order_uid_is_deterministic_and_extracts_its_inputs(
        domain in domain_strategy(),
        order in order_strategy(),
        owner in address_strategy(),
    ) {
        let first = compute_order_uid(&domain, &order, &owner).unwrap();
        let second = compute_order_uid(&domain, &order, &owner).unwrap();
        prop_assert_eq!(first.as_str(), second.as_str());

        let extracted = extract_order_uid_params(&first).unwrap();
        prop_assert_eq!(extracted.owner.as_str(), owner.as_str());
        prop_assert_eq!(extracted.valid_to, order.valid_to);
    }

    /// [`hash_order`] is deterministic under a fixed `(domain, order)`
    /// input; [`normalize_order`] is deterministic; and semantically
    /// equivalent orders (same fields modulo the reviewed balance-
    /// normalization rule) produce the same normalized form and the same
    /// digest under a shared domain.
    #[test]
    fn order_hashing_is_deterministic_for_equivalent_normalized_inputs(
        domain in domain_strategy(),
        (order, equivalent) in equivalent_order_pair_strategy(),
    ) {
        let normalized = normalize_order(&order).unwrap();
        let equivalent_normalized = normalize_order(&equivalent).unwrap();
        prop_assert_eq!(&normalized, &equivalent_normalized);

        let hash = hash_order(&domain, &order).unwrap();
        let equivalent_hash = hash_order(&domain, &equivalent).unwrap();
        prop_assert_eq!(hash.as_str(), equivalent_hash.as_str());

        let repeat = hash_order(&domain, &order).unwrap();
        prop_assert_eq!(repeat.as_str(), hash.as_str());
    }

    /// [`hash_order`] is invariant across address case variants on every
    /// address-shaped field. Lowercase, uppercase, and mixed-case
    /// renderings of the same sell/buy/receiver bytes hash to the same
    /// order digest.
    #[test]
    fn order_hashing_is_stable_across_address_case_variants(
        domain in domain_strategy(),
        order in order_strategy(),
    ) {
        let uppercase = |address: &Address| -> Address {
            let upper = format!(
                "0x{}",
                address
                    .as_str()
                    .trim_start_matches("0x")
                    .to_ascii_uppercase()
            );
            Address::new(upper).unwrap()
        };

        let mut uppercase_order = order.clone();
        uppercase_order.sell_token = uppercase(&order.sell_token);
        uppercase_order.buy_token = uppercase(&order.buy_token);
        uppercase_order.receiver = order.receiver.as_ref().map(uppercase);

        prop_assert_eq!(&order.sell_token, &uppercase_order.sell_token);

        let hash_original = hash_order(&domain, &order).unwrap();
        let hash_upper = hash_order(&domain, &uppercase_order).unwrap();
        prop_assert_eq!(hash_original.as_str(), hash_upper.as_str());
    }

    /// [`encode_trade`] / [`decode_order`] / [`decode_trade_flags`]
    /// preserve the normalized-order boundary: encoding a normalized
    /// order with any supported signing-scheme signature and decoding
    /// through the token registry reproduces the normalized kind,
    /// partial-fill flag, sell balance, buy balance, executed amount,
    /// and normalized order shape.
    #[test]
    fn encoded_trades_preserve_the_normalized_order_boundary(
        order in order_strategy(),
        executed_amount in amount_strategy(),
        (_scheme, signature) in scheme_and_signature_strategy(),
    ) {
        let normalized = normalize_order(&order).unwrap();
        let execution = TradeExecution { executed_amount };
        let mut tokens = TokenRegistry::new();

        let trade = encode_trade(&mut tokens, &normalized, &signature, &execution).unwrap();
        let decoded_flags = decode_trade_flags(trade.flags).unwrap();
        let decoded_order = decode_order(&trade, &tokens.addresses()).unwrap();

        prop_assert_eq!(decoded_flags.kind, normalized.kind);
        prop_assert_eq!(decoded_flags.partially_fillable, normalized.partially_fillable);
        prop_assert_eq!(decoded_flags.sell_token_balance, normalized.sell_token_balance);
        prop_assert_eq!(decoded_flags.buy_token_balance, normalized.buy_token_balance);
        prop_assert_eq!(&trade.executed_amount, &execution.executed_amount);
        prop_assert_eq!(&normalize_order(&decoded_order).unwrap(), &normalized);
    }

    /// [`encode_order_flags`] / [`decode_order_flags`] and
    /// [`encode_trade_flags`] / [`decode_trade_flags`] are strict
    /// inverses on every admitted variant after the reviewed buy-balance
    /// normalization rule; the encoded trade flag reserves the high bit
    /// so the sign bit stays zero.
    #[test]
    fn compact_flag_codecs_roundtrip_across_generated_variants(
        kind_sell in any::<bool>(),
        partially_fillable in any::<bool>(),
        sell_balance in sell_balance_strategy(),
        buy_balance in buy_balance_strategy(),
        scheme in signing_scheme_strategy(),
    ) {
        let kind = if kind_sell { OrderKind::Sell } else { OrderKind::Buy };
        let order_flags = OrderFlags {
            kind,
            partially_fillable,
            sell_token_balance: sell_balance,
            buy_token_balance: buy_balance,
        };
        let encoded_order = encode_order_flags(&order_flags).unwrap();
        let decoded_order = decode_order_flags(encoded_order).unwrap();
        let expected_order = OrderFlags {
            buy_token_balance: canonical_buy_balance(order_flags.buy_token_balance),
            ..order_flags.clone()
        };
        prop_assert_eq!(&decoded_order, &expected_order);
        prop_assert_eq!(encode_order_flags(&decoded_order).unwrap(), encoded_order);

        let trade_flags = TradeFlags {
            kind,
            partially_fillable,
            sell_token_balance: sell_balance,
            buy_token_balance: buy_balance,
            signing_scheme: scheme,
        };
        let encoded_trade = encode_trade_flags(&trade_flags).unwrap();
        let decoded_trade = decode_trade_flags(encoded_trade).unwrap();
        let expected_trade = TradeFlags {
            buy_token_balance: canonical_buy_balance(trade_flags.buy_token_balance),
            ..trade_flags.clone()
        };
        prop_assert_eq!(encoded_trade & 0b1000_0000, 0);
        prop_assert_eq!(&decoded_trade, &expected_trade);
        prop_assert_eq!(encode_trade_flags(&decoded_trade).unwrap(), encoded_trade);
    }

    /// [`encode_eip1271_signature_data`] and
    /// [`decode_eip1271_signature_data`] preserve the verifier address
    /// and payload bytes across any signature body drawn from the
    /// documented boundary lengths; the encoded form is lowercase and
    /// exactly `2 + (20 + byte_len) * 2` characters long.
    /// [`normalized_ecdsa_signature`] collapses mixed-case hex payloads
    /// onto the canonical lowercase form with the same underlying bytes.
    #[test]
    fn signature_codecs_preserve_verifier_and_payload_bytes(
        verifier in address_strategy(),
        len_index in 0usize..SIGNATURE_BOUNDARY_LENGTHS.len(),
        seed in any::<u64>(),
    ) {
        let byte_len = SIGNATURE_BOUNDARY_LENGTHS[len_index];
        let payload_bytes: Vec<u8> =
            (0..byte_len).map(|index| (seed.wrapping_add(index as u64) as u8) ^ 0xA5).collect();
        let casing: Vec<bool> = (0..byte_len * 2)
            .map(|bit| ((seed >> (bit % 64)) & 1) == 1)
            .collect();
        let signature = render_mixed_case(&payload_bytes, &casing);

        let normalized = normalized_ecdsa_signature(&signature).unwrap();
        prop_assert_eq!(normalized.clone(), normalized.to_ascii_lowercase());
        prop_assert_eq!(
            hex::decode(normalized.trim_start_matches("0x")).unwrap(),
            payload_bytes.clone(),
        );

        let encoded = encode_eip1271_signature_data(&Eip1271SignatureData {
            verifier: verifier.clone(),
            signature: signature.clone(),
        })
        .unwrap();
        let decoded = decode_eip1271_signature_data(&encoded).unwrap();

        prop_assert_eq!(&decoded.verifier, &verifier);
        prop_assert_eq!(decoded.signature, normalized);
        prop_assert_eq!(encoded.len(), 2 + ((20 + byte_len) * 2));

        let encoded_bytes = hex::decode(encoded.trim_start_matches("0x")).unwrap();
        let verifier_bytes = hex::decode(verifier.as_str().trim_start_matches("0x")).unwrap();
        prop_assert_eq!(encoded_bytes.len(), 20 + byte_len);
        prop_assert_eq!(&encoded_bytes[..20], verifier_bytes.as_slice());
        prop_assert_eq!(&encoded_bytes[20..], payload_bytes.as_slice());
    }

    /// [`encode_signing_scheme`] and [`decode_signing_scheme`] are strict
    /// inverses on every supported variant. [`SigningScheme::as_u8`]
    /// stays in lockstep with the encoded form.
    #[test]
    fn signing_scheme_encode_decode_roundtrip(scheme in signing_scheme_strategy()) {
        let encoded = encode_signing_scheme(scheme);
        prop_assert_eq!(encoded, scheme.as_u8());

        let decoded = decode_signing_scheme(encoded).unwrap();
        prop_assert_eq!(decoded, scheme);
    }

    /// [`decode_signing_scheme`] fails closed on every byte outside the
    /// supported set `{0, 1, 2, 3}`.
    #[test]
    fn signing_scheme_decode_rejects_unknown_bytes(byte in 4u8..=u8::MAX) {
        prop_assert!(decode_signing_scheme(byte).is_err());
    }
}
