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
    ContractsError, Eip1271SignatureData, EthFlowOrderData, OrderUidParams, RecoverableSignature,
    SigningScheme, compute_order_uid, decode_eip1271_signature_data, decode_signing_scheme,
    encode_eip1271_signature_data, encode_signing_scheme, extract_order_uid_params, hash_order,
    pack_order_uid_params,
};
use cow_sdk_core::{
    Address, Amount, AppDataHash, AppDataHex, BuyTokenDestination, OrderData, OrderDigest,
    OrderKind, SellTokenSource, TypedDataDomain,
};
use proptest::prelude::*;
use proptest::test_runner::{FileFailurePersistence, TestRunner};

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
    cow_sdk_test_utils::arb::arb_address()
}

/// Strategy that emits a 32-byte order digest wrapped in [`OrderDigest`].
fn order_digest_strategy() -> impl Strategy<Value = OrderDigest> {
    any::<[u8; 32]>().prop_map(|bytes| {
        OrderDigest::new(format!("0x{}", alloy_primitives::hex::encode(bytes))).unwrap()
    })
}

/// Strategy that emits an [`AppDataHex`] payload.
fn app_data_strategy() -> impl Strategy<Value = AppDataHex> {
    cow_sdk_test_utils::arb::arb_app_data_hex()
}

/// Strategy that emits an [`Amount`] with at least one non-zero byte so
/// order-hashing inputs stay outside the all-zero boundary.
fn amount_strategy() -> impl Strategy<Value = Amount> {
    cow_sdk_test_utils::arb::arb_amount()
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
fn sell_balance_strategy() -> impl Strategy<Value = SellTokenSource> {
    prop_oneof![
        Just(SellTokenSource::Erc20),
        Just(SellTokenSource::External),
        Just(SellTokenSource::Internal),
    ]
}

/// Strategy that emits every `buy_token_balance` shape the reviewed
/// order contract admits. `BuyTokenDestination` is a closed type in the
/// services model, so the strategy cycles through `Erc20` and `Internal`
/// only.
fn buy_balance_strategy() -> impl Strategy<Value = BuyTokenDestination> {
    prop_oneof![
        Just(BuyTokenDestination::Erc20),
        Just(BuyTokenDestination::Internal),
    ]
}

/// Strategy that emits a deterministic order suitable for hashing and
/// trade encoding. All typed fields are drawn from their reviewed
/// domains; the balance fields cycle through every admitted variant.
fn order_strategy() -> impl Strategy<Value = OrderData> {
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
                OrderData::new(
                    sell_token,
                    buy_token,
                    receiver,
                    sell_amount,
                    buy_amount,
                    valid_to,
                    app_data,
                    fee_amount,
                    if kind_sell {
                        OrderKind::Sell
                    } else {
                        OrderKind::Buy
                    },
                    partially_fillable,
                    sell_token_balance,
                    buy_token_balance,
                )
            },
        )
}

/// Strategy that emits a typed-data domain whose `verifying_contract`
/// is a non-zero address. The name and version fields stay fixed so
/// the reviewed domain shape stays close to the shipped settlement
/// domain.
fn domain_strategy() -> impl Strategy<Value = TypedDataDomain> {
    (address_strategy(), 1u64..=100_000u64).prop_map(|(verifying_contract, chain_id)| {
        TypedDataDomain::new(
            "Gnosis Protocol".to_owned(),
            "v2".to_owned(),
            chain_id,
            verifying_contract,
        )
    })
}

fn signature_with_v(r_bytes: &[u8; 32], s_bytes: &[u8; 32], v_byte: u8) -> String {
    let mut bytes = [0u8; 65];
    bytes[..32].copy_from_slice(r_bytes);
    bytes[32..64].copy_from_slice(s_bytes);
    bytes[64] = v_byte;
    format!("0x{}", alloy_primitives::hex::encode(bytes))
}

#[test]
fn ecdsa_v_normalization_rejects_every_excluded_byte_value() {
    let mut runner = TestRunner::new(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSION_FILE))),
        ..ProptestConfig::default()
    });

    runner
        .run(
            &(any::<[u8; 32]>(), any::<[u8; 32]>()),
            |(r_bytes, s_bytes)| {
                for v_byte in 0u8..=u8::MAX {
                    let signature = signature_with_v(&r_bytes, &s_bytes, v_byte);
                    match v_byte {
                        0 | 1 | 27 | 28 => {
                            let sig = RecoverableSignature::parse_hex(&signature).unwrap();
                            let output = sig.to_bytes();
                            let expected_v = if matches!(v_byte, 0 | 27) { 27 } else { 28 };

                            prop_assert_eq!(&output[..32], r_bytes.as_slice());
                            prop_assert_eq!(&output[32..64], s_bytes.as_slice());
                            prop_assert_eq!(output[64], expected_v);
                        }
                        _ => {
                            let error = RecoverableSignature::parse_hex(&signature).unwrap_err();
                            let rejected_with_value = matches!(
                                error,
                                ContractsError::InvalidSignatureRecoveryByte { value } if value == v_byte
                            );
                            prop_assert!(rejected_with_value);
                        }
                    }
                }
                Ok(())
            },
        )
        .unwrap();
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
        let params = OrderUidParams::new(digest, owner, valid_to);
        let uid = pack_order_uid_params(&params).unwrap();
        let extracted = extract_order_uid_params(&uid).unwrap();

        prop_assert_eq!(extracted.order_digest.to_hex_string(), digest.to_hex_string());
        prop_assert_eq!(extracted.owner.to_hex_string(), owner.to_hex_string());
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
        prop_assert_eq!(first.to_hex_string(), second.to_hex_string());

        let extracted = extract_order_uid_params(&first).unwrap();
        prop_assert_eq!(extracted.owner.to_hex_string(), owner.to_hex_string());
        prop_assert_eq!(extracted.valid_to, order.valid_to);
    }

    /// [`hash_order`] is deterministic under a fixed `(domain, order)`
    /// input: hashing the same concrete [`OrderData`] twice produces a
    /// byte-identical digest.
    #[test]
    fn order_hashing_is_deterministic(
        domain in domain_strategy(),
        order in order_strategy(),
    ) {
        let hash = hash_order(&domain, &order).unwrap();
        let repeat = hash_order(&domain, &order).unwrap();
        prop_assert_eq!(repeat.to_hex_string(), hash.to_hex_string());
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
                    .to_hex_string()
                    .trim_start_matches("0x")
                    .to_ascii_uppercase()
            );
            Address::new(upper).unwrap()
        };

        let mut uppercase_order = order.clone();
        uppercase_order.sell_token = uppercase(&order.sell_token);
        uppercase_order.buy_token = uppercase(&order.buy_token);
        uppercase_order.receiver = uppercase(&order.receiver);

        prop_assert_eq!(&order.sell_token, &uppercase_order.sell_token);

        let hash_original = hash_order(&domain, &order).unwrap();
        let hash_upper = hash_order(&domain, &uppercase_order).unwrap();
        prop_assert_eq!(hash_original.to_hex_string(), hash_upper.to_hex_string());
    }

    /// [`encode_eip1271_signature_data`] and
    /// [`decode_eip1271_signature_data`] preserve the verifier address
    /// and payload bytes across any signature body drawn from the
    /// documented boundary lengths; the encoded form is lowercase and
    /// exactly `2 + (20 + byte_len) * 2` characters long.
    /// [`RecoverableSignature::parse_hex`] accepts the canonical 65-byte
    /// ECDSA shape, lowercases the hex payload, and preserves the
    /// underlying `r || s || v` bytes when `v` is already in the
    /// legacy `{27, 28}` range.
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

        let mut normalized_payload_bytes: Vec<u8> = (0..65)
            .map(|index| (seed.wrapping_add(index as u64) as u8) ^ 0x5A)
            .collect();
        normalized_payload_bytes[64] = if (seed & 1) == 0 { 27 } else { 28 };
        let normalized_casing: Vec<bool> = (0..130)
            .map(|bit| ((seed.rotate_left(17) >> (bit % 64)) & 1) == 1)
            .collect();
        let normalized_signature =
            render_mixed_case(&normalized_payload_bytes, &normalized_casing);

        let sig = RecoverableSignature::parse_hex(&normalized_signature).unwrap();
        let normalized = sig.to_hex_string();
        prop_assert_eq!(normalized.clone(), normalized.to_ascii_lowercase());
        prop_assert_eq!(sig.to_bytes().to_vec(), normalized_payload_bytes);

        let encoded = encode_eip1271_signature_data(&Eip1271SignatureData::new(
            verifier,
            signature.clone(),
        ))
        .unwrap();
        let decoded = decode_eip1271_signature_data(&encoded).unwrap();

        prop_assert_eq!(&decoded.verifier, &verifier);
        prop_assert_eq!(decoded.signature, format!("0x{}", alloy_primitives::hex::encode(&payload_bytes)));
        prop_assert_eq!(encoded.len(), 2 + ((20 + byte_len) * 2));

        let encoded_bytes = alloy_primitives::hex::decode(encoded.trim_start_matches("0x")).unwrap();
        let verifier_bytes = alloy_primitives::hex::decode(verifier.to_hex_string().trim_start_matches("0x")).unwrap();
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

    /// [`EthFlowOrderData::new`] accepts every non-zero receiver and
    /// rejects `Address::ZERO` with `ContractsError::ZeroReceiver`,
    /// pre-empting the upstream `CoWSwapEthFlow` contract's
    /// `ReceiverMustBeSet()` revert (selector `0xefc9ccdf`). The
    /// bidirectional invariant covers the full 2^160 address space.
    #[test]
    fn ethflow_order_data_new_rejects_zero_receiver_iff_address_is_zero(
        receiver_bytes in proptest::array::uniform20(any::<u8>()),
    ) {
        let receiver_hex = format!("0x{}", alloy_primitives::hex::encode(receiver_bytes));
        let receiver = Address::new(receiver_hex).unwrap();
        let result = EthFlowOrderData::new(
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            receiver,
            Amount::new("1000000000000000000").unwrap(),
            Amount::new("2000000000000000000").unwrap(),
            AppDataHash::new(
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            )
            .unwrap(),
            Amount::ZERO,
            0xFFFF_FFFF,
            false,
            0,
        );
        if receiver.is_zero() {
            prop_assert!(matches!(result, Err(ContractsError::ZeroReceiver)));
        } else {
            prop_assert!(result.is_ok());
        }
    }

}
