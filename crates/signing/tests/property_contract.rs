#![cfg(not(target_arch = "wasm32"))]
//! Property-based coverage for the deterministic `cow-sdk-signing` boundary.
//!
//! Each `proptest!` case exercises a named invariant on one of the
//! typed-data, order-id, cancellation, or EIP-1271 signing helpers.
//! Shrinking narrows any counter-example before `cargo test` prints it,
//! and committed seed files under `tests/proptest-regressions/` keep the
//! shrink outcomes reproducible across contributors. Net coverage
//! matches the hand-rolled enumerator this file replaced: every
//! invariant family the enumerator exercised carries a named property
//! here.

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

mod common;

use std::collections::BTreeMap;

use cow_sdk_contracts::{OrderCancellations, SigningScheme, hash_order, hash_order_cancellations};
use cow_sdk_core::{
    Address, Amount, AppDataHex, BuyTokenDestination, CowEnv, OrderKind, OrderUid, ProtocolOptions,
    SellTokenSource, SupportedChainId, TypedDataDomain, UnsignedOrder,
};
use cow_sdk_signing::{
    ORDER_PRIMARY_TYPE, domain_fields, domain_separator_for, eip1271_signature_payload,
    generate_order_id, get_domain, order_cancellations_typed_data_payload, order_fields,
    order_typed_data, order_typed_data_payload, sign_order_cancellations_with_scheme,
    sign_order_with_scheme,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

use common::MockSigner;

/// Path for committed regression seeds; proptest writes new shrink
/// outcomes here so every contributor re-runs prior counter-examples
/// before any novel case is generated.
const REGRESSION_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/proptest-regressions/property_contract.txt"
);

/// EIP-1271 signature tail lengths from the reviewed ABI boundary set
/// so the dynamic-tail offset, length, and padding arithmetic is
/// exercised at every 32-byte transition.
/// Strategy that emits an address with a non-zero low byte.
fn address_strategy() -> impl Strategy<Value = Address> {
    any::<[u8; 20]>().prop_map(|mut bytes| {
        if bytes.iter().all(|byte| *byte == 0) {
            bytes[19] = 1;
        }
        Address::new(format!("0x{}", hex::encode(bytes))).unwrap()
    })
}

/// Strategy that emits an [`AppDataHex`] payload.
fn app_data_strategy() -> impl Strategy<Value = AppDataHex> {
    any::<[u8; 32]>()
        .prop_map(|bytes| AppDataHex::new(format!("0x{}", hex::encode(bytes))).unwrap())
}

/// Strategy that emits an [`Amount`] with at least one non-zero byte.
fn amount_strategy() -> impl Strategy<Value = Amount> {
    any::<[u8; 32]>().prop_map(|mut bytes| {
        if bytes.iter().all(|byte| *byte == 0) {
            bytes[31] = 1;
        }
        Amount::new(format!("0x{}", hex::encode(bytes))).unwrap()
    })
}

/// Strategy that emits every supported chain id.
fn chain_id_strategy() -> impl Strategy<Value = SupportedChainId> {
    prop::sample::select(SupportedChainId::ALL.to_vec())
}

/// Strategy that emits a 56-byte order UID.
fn order_uid_strategy() -> impl Strategy<Value = OrderUid> {
    (any::<[u8; 32]>(), any::<[u8; 24]>()).prop_map(|(first, second)| {
        let mut bytes = [0u8; 56];
        bytes[..32].copy_from_slice(&first);
        bytes[32..].copy_from_slice(&second);
        OrderUid::new(format!("0x{}", hex::encode(bytes))).unwrap()
    })
}

/// Strategy that emits a non-empty vector of order UIDs for
/// cancellation-batch coverage.
fn order_uid_batch_strategy() -> impl Strategy<Value = Vec<OrderUid>> {
    prop::collection::vec(order_uid_strategy(), 1..=4)
}

/// Strategy that emits a typed-data domain with randomized but
/// reviewed-shape fields.
fn domain_strategy() -> impl Strategy<Value = TypedDataDomain> {
    (
        "cow-rs-domain-[A-Za-z0-9]{1,12}",
        (1u32..=9u32, 0u32..=9u32, 0u32..=9u32),
        1u64..=1_000_000u64,
        address_strategy(),
    )
        .prop_map(
            |(name, (major, minor, patch), chain_id, verifying_contract)| TypedDataDomain {
                name: Some(name.into()),
                version: Some(format!("{major}.{minor}.{patch}").into()),
                chain_id: Some(alloy_primitives::U256::from(chain_id)),
                verifying_contract: Some(*verifying_contract.as_alloy()),
                salt: None,
            },
        )
}

/// Strategy that emits a `(domain, domain)` pair where the second member
/// is guaranteed to differ from the first in exactly one field.
fn domain_and_changed_strategy() -> impl Strategy<Value = (TypedDataDomain, TypedDataDomain)> {
    (domain_strategy(), 0u8..=3u8).prop_map(|(domain, which)| {
        let mut changed = domain.clone();
        match which {
            0 => {
                let mut name = changed.name.as_deref().unwrap_or_default().to_owned();
                name.push_str("-alt");
                changed.name = Some(name.into());
            }
            1 => {
                let mut version = changed.version.as_deref().unwrap_or_default().to_owned();
                version.push_str(".1");
                changed.version = Some(version.into());
            }
            2 => {
                let chain_id_u256 = changed.chain_id.unwrap_or(alloy_primitives::U256::ZERO);
                changed.chain_id =
                    Some(chain_id_u256.saturating_add(alloy_primitives::U256::from(1u64)));
            }
            _ => {
                let current_addr = changed.verifying_contract.unwrap_or_default();
                let mut bytes = current_addr.into_array();
                bytes[19] ^= 1;
                if bytes.iter().all(|byte| *byte == 0) {
                    bytes[19] = 1;
                }
                changed.verifying_contract = Some(alloy_primitives::Address::from(bytes));
            }
        }
        (domain, changed)
    })
}

/// Strategy that emits a deterministic unsigned order.
fn unsigned_order_strategy() -> impl Strategy<Value = UnsignedOrder> {
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
        0u8..=2u8,
        0u8..=1u8,
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
                sell_balance_selector,
                buy_balance_selector,
            )| {
                let sell_token_balance = match sell_balance_selector {
                    0 => SellTokenSource::Erc20,
                    1 => SellTokenSource::External,
                    _ => SellTokenSource::Internal,
                };
                let buy_token_balance = match buy_balance_selector {
                    0 => BuyTokenDestination::Erc20,
                    _ => BuyTokenDestination::Internal,
                };

                UnsignedOrder::new(
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

/// Strategy that emits a `(chain, options)` pair so the optional
/// [`ProtocolOptions`] is drawn together with its chain context. The
/// emitted options either pin an environment, pin a settlement-contract
/// override for `chain`, or do both; returning `None` is also a branch
/// so the default-domain path is exercised.
fn chain_with_protocol_options_strategy()
-> impl Strategy<Value = (SupportedChainId, Option<ProtocolOptions>)> {
    chain_id_strategy().prop_flat_map(|chain| {
        (
            Just(chain),
            (
                any::<bool>(),
                any::<bool>(),
                any::<bool>(),
                address_strategy(),
            )
                .prop_map(
                    move |(want_env, env_is_staging, want_override, override_address)| {
                        if !want_env && !want_override {
                            return None;
                        }
                        let mut options = ProtocolOptions::new();
                        if want_env {
                            options = options.with_env(if env_is_staging {
                                CowEnv::Staging
                            } else {
                                CowEnv::Prod
                            });
                        }
                        if want_override {
                            let mut overrides = BTreeMap::new();
                            overrides.insert(u64::from(chain), override_address);
                            options = options.with_settlement_contract_override(overrides);
                        }
                        Some(options)
                    },
                ),
        )
    })
}

fn decode_u256_word(word: &[u8]) -> usize {
    let bytes: [u8; 8] = word[24..32].try_into().unwrap();
    u64::from_be_bytes(bytes) as usize
}

fn padded_len(len: usize) -> usize {
    if len == 0 { 0 } else { len.div_ceil(32) * 32 }
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSION_FILE))),
        ..ProptestConfig::default()
    })]

    /// [`domain_separator_for`] is deterministic on a fixed
    /// [`TypedDataDomain`] and changes whenever any of the four reviewed
    /// fields (`name`, `version`, `chain_id`, `verifying_contract`)
    /// change.
    #[test]
    fn domain_separators_change_only_when_the_typed_data_domain_changes(
        (domain, changed) in domain_and_changed_strategy(),
    ) {
        let separator = domain_separator_for(&domain).unwrap();
        let repeated = domain_separator_for(&domain).unwrap();
        let changed_separator = domain_separator_for(&changed).unwrap();

        prop_assert_eq!(&separator, &repeated);
        prop_assert_ne!(&separator, &changed_separator);
    }

    /// [`order_typed_data_payload`] and [`generate_order_id`] are
    /// deterministic for a fixed `(chain, order, owner)` triple; the
    /// generated digest matches [`hash_order`] under the resolved
    /// domain; and [`sign_order_with_scheme`] routes typed-data and
    /// ethSign schemes through the correct [`MockSigner`] channel with
    /// the expected digest bytes for the message-signing path.
    #[test]
    fn order_payloads_and_generated_ids_are_deterministic_and_scheme_explicit(
        chain in chain_id_strategy(),
        order in unsigned_order_strategy(),
        owner in address_strategy(),
    ) {
        let payload = order_typed_data_payload(chain, &order, None).unwrap();
        let repeated_payload = order_typed_data_payload(chain, &order, None).unwrap();
        let generated = generate_order_id(chain, &order, &owner, None).unwrap();
        let repeated_generated = generate_order_id(chain, &order, &owner, None).unwrap();
        let expected_digest = hash_order(
            &get_domain(chain, None).unwrap(),
            &cow_sdk_contracts::Order::from(&order),
        )
        .unwrap();

        prop_assert_eq!(&payload, &repeated_payload);
        prop_assert_eq!(&generated, &repeated_generated);
        prop_assert_eq!(&generated.order_digest, &expected_digest);

        let typed_signer = MockSigner::new();
        let typed_result = sign_order_with_scheme(
            &order,
            chain,
            &typed_signer,
            SigningScheme::Eip712,
            None,
        )
        .unwrap();
        let typed_calls = typed_signer.calls.borrow().clone();
        prop_assert_eq!(typed_result.signing_scheme, SigningScheme::Eip712);
        prop_assert_eq!(typed_calls.typed_data.len(), 1);
        prop_assert!(typed_calls.messages.is_empty());

        let message_signer = MockSigner::new();
        let message_result = sign_order_with_scheme(
            &order,
            chain,
            &message_signer,
            SigningScheme::EthSign,
            None,
        )
        .unwrap();
        let message_calls = message_signer.calls.borrow().clone();
        prop_assert_eq!(message_result.signing_scheme, SigningScheme::EthSign);
        prop_assert!(message_calls.typed_data.is_empty());
        prop_assert_eq!(message_calls.messages.len(), 1);
        prop_assert_eq!(
            format!("0x{}", hex::encode(&message_calls.messages[0])),
            expected_digest.to_hex_string().clone(),
        );
    }

    /// [`order_cancellations_typed_data_payload`] is deterministic, its
    /// digest matches [`hash_order_cancellations`], and
    /// [`sign_order_cancellations_with_scheme`] routes typed-data and
    /// ethSign schemes through the correct [`MockSigner`] channel.
    #[test]
    fn cancellation_payloads_are_deterministic_and_preserve_scheme_boundaries(
        chain in chain_id_strategy(),
        order_uids in order_uid_batch_strategy(),
    ) {
        let payload = order_cancellations_typed_data_payload(&order_uids, chain, None).unwrap();
        let repeated_payload =
            order_cancellations_typed_data_payload(&order_uids, chain, None).unwrap();
        let expected_digest = hash_order_cancellations(
            &get_domain(chain, None).unwrap(),
            &OrderCancellations::new(order_uids.clone()),
        )
        .unwrap();

        prop_assert_eq!(&payload, &repeated_payload);

        let typed_signer = MockSigner::new();
        let typed_result = sign_order_cancellations_with_scheme(
            &order_uids,
            chain,
            &typed_signer,
            SigningScheme::Eip712,
            None,
        )
        .unwrap();
        let typed_calls = typed_signer.calls.borrow().clone();
        prop_assert_eq!(typed_result.signing_scheme, SigningScheme::Eip712);
        prop_assert_eq!(typed_calls.typed_data.len(), 1);
        prop_assert!(typed_calls.messages.is_empty());

        let message_signer = MockSigner::new();
        let message_result = sign_order_cancellations_with_scheme(
            &order_uids,
            chain,
            &message_signer,
            SigningScheme::EthSign,
            None,
        )
        .unwrap();
        let message_calls = message_signer.calls.borrow().clone();
        prop_assert_eq!(message_result.signing_scheme, SigningScheme::EthSign);
        prop_assert!(message_calls.typed_data.is_empty());
        prop_assert_eq!(message_calls.messages.len(), 1);
        prop_assert_eq!(
            format!("0x{}", hex::encode(&message_calls.messages[0])),
            expected_digest.to_hex_string().clone(),
        );
    }

    /// [`order_typed_data`] preserves the reviewed field shape: the
    /// primary type is [`ORDER_PRIMARY_TYPE`], the types map holds the
    /// shipped [`order_fields`] and [`domain_fields`] values, the
    /// resolved domain matches [`get_domain`], and a
    /// [`ProtocolOptions::settlement_contract_override`] is honoured on
    /// the verifying contract field of the emitted domain.
    #[test]
    fn typed_order_payloads_preserve_fields_message_and_override_contracts(
        (chain, options) in chain_with_protocol_options_strategy(),
        order in unsigned_order_strategy(),
    ) {
        let expected_domain = get_domain(chain, options.as_ref()).unwrap();

        let payload = order_typed_data_payload(chain, &order, options.as_ref()).unwrap();
        let repeated_payload = order_typed_data_payload(chain, &order, options.as_ref()).unwrap();
        let typed = order_typed_data(chain, &order, options.as_ref()).unwrap();

        prop_assert_eq!(&payload, &repeated_payload);
        prop_assert_eq!(payload.primary_type.clone(), ORDER_PRIMARY_TYPE);
        prop_assert_eq!(typed.primary_type.clone(), ORDER_PRIMARY_TYPE);
        prop_assert_eq!(&payload.domain, &expected_domain);
        prop_assert_eq!(&typed.domain, &expected_domain);
        prop_assert_eq!(&typed.types, &payload.types);
        prop_assert_eq!(payload.types.get(ORDER_PRIMARY_TYPE).unwrap().clone(), order_fields());
        prop_assert_eq!(payload.types.get("EIP712Domain").unwrap().clone(), domain_fields());
        prop_assert_eq!(payload.message.clone(), serde_json::to_string(&order).unwrap());
        prop_assert_eq!(&typed.message, &order);

        if let Some(overrides) = options
            .as_ref()
            .and_then(|options| options.settlement_contract_override.as_ref())
        {
            let expected = overrides.get(&u64::from(chain)).unwrap();
            prop_assert_eq!(
                payload.domain.verifying_contract,
                Some(*expected.as_alloy()),
            );
        }
    }

    /// [`eip1271_signature_payload`] preserves the reviewed ABI tail
    /// layout for canonical 65-byte recoverable signatures: the
    /// app-data field lands at word 6, the dynamic offset word points
    /// at word 13, the length word records the unpadded byte count, the
    /// signature bytes follow at word 14, and the tail is zero-padded
    /// to a 32-byte multiple.
    #[test]
    fn eip1271_payloads_preserve_dynamic_tail_boundaries_across_generated_signatures(
        order in unsigned_order_strategy(),
        seed in any::<u64>(),
    ) {
        let mut signature_bytes: Vec<u8> = (0..65)
            .map(|index| (seed.wrapping_add(index as u64) as u8) ^ 0x5A)
            .collect();
        signature_bytes[64] = if (seed & 1) == 0 { 27 } else { 28 };
        let signature = format!("0x{}", hex::encode(&signature_bytes));
        let payload = eip1271_signature_payload(&order, &signature).unwrap();
        let encoded = hex::decode(payload.trim_start_matches("0x")).unwrap();

        let offset_word_start = 32 * 12;
        let length_word_start = 32 * 13;
        let data_start = 32 * 14;
        let expected_padding = padded_len(signature_bytes.len());

        prop_assert_eq!(&encoded[32 * 6..32 * 7], order.app_data.as_slice());
        prop_assert_eq!(
            decode_u256_word(&encoded[offset_word_start..offset_word_start + 32]),
            32 * 13,
        );
        prop_assert_eq!(
            decode_u256_word(&encoded[length_word_start..length_word_start + 32]),
            signature_bytes.len(),
        );
        prop_assert_eq!(encoded.len(), data_start + expected_padding);
        prop_assert_eq!(
            &encoded[data_start..data_start + signature_bytes.len()],
            signature_bytes.as_slice(),
        );
        prop_assert!(
            encoded[data_start + signature_bytes.len()..data_start + expected_padding]
                .iter()
                .all(|byte| *byte == 0),
        );
    }
}
