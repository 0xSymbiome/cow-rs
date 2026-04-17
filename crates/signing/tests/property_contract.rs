#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::iter_on_single_items,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::unnested_or_patterns,
    reason = "pedantic, nursery, and perf lints acceptable in test helper code"
)]

use std::collections::BTreeMap;

mod common;

use cow_sdk_contracts::{OrderCancellations, SigningScheme, hash_order, hash_order_cancellations};
use cow_sdk_core::{
    Address, Amount, AppDataHex, CowEnv, OrderBalance, OrderKind, OrderUid, ProtocolOptions,
    SupportedChainId, TypedDataDomain, UnsignedOrder,
};
use cow_sdk_signing::{
    ORDER_PRIMARY_TYPE, domain_fields, domain_separator_for, eip1271_signature_payload,
    generate_order_id, get_domain, order_cancellations_typed_data_payload, order_fields,
    order_typed_data, order_typed_data_payload, sign_order_cancellations_with_scheme,
    sign_order_with_scheme,
};

use common::MockSigner;

const CASE_COUNT: u64 = 128;

#[derive(Clone)]
struct CaseRng {
    state: u64,
}

impl CaseRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.state = value;
        value.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }

    fn fill<const N: usize>(&mut self) -> [u8; N] {
        let mut bytes = [0u8; N];
        for byte in &mut bytes {
            *byte = self.next_u64() as u8;
        }
        bytes
    }

    fn non_zero_address(&mut self) -> Address {
        let mut bytes = self.fill::<20>();
        if bytes.iter().all(|byte| *byte == 0) {
            bytes[19] = 1;
        }
        Address::new(format!("0x{}", hex::encode(bytes))).unwrap()
    }

    fn app_data(&mut self) -> AppDataHex {
        AppDataHex::new(format!("0x{}", hex::encode(self.fill::<32>()))).unwrap()
    }

    fn amount(&mut self) -> Amount {
        let mut bytes = self.fill::<32>();
        if bytes.iter().all(|byte| *byte == 0) {
            bytes[31] = 1;
        }
        Amount::new(format!("0x{}", hex::encode(bytes))).unwrap()
    }

    fn supported_chain(&mut self) -> SupportedChainId {
        SupportedChainId::ALL[(self.next_u64() as usize) % SupportedChainId::ALL.len()]
    }

    fn order_uid(&mut self) -> OrderUid {
        OrderUid::new(format!("0x{}", hex::encode(self.fill::<56>()))).unwrap()
    }

    fn bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.next_u64() as u8).collect()
    }
}

fn generated_domain(rng: &mut CaseRng) -> TypedDataDomain {
    TypedDataDomain {
        name: format!("cow-rs-domain-{}", rng.next_u32()),
        version: format!("{}.{}.{}", 1 + (rng.next_u32() % 9), 0, 0),
        chain_id: u64::from(rng.next_u32()) + 1,
        verifying_contract: rng.non_zero_address(),
    }
}

fn different_domain(mut domain: TypedDataDomain, seed: u64) -> TypedDataDomain {
    match seed % 4 {
        0 => domain.name.push_str("-alt"),
        1 => domain.version.push_str(".1"),
        2 => domain.chain_id = domain.chain_id.saturating_add(1),
        _ => {
            let current = domain.verifying_contract.as_str().trim_start_matches("0x");
            let mut bytes = hex::decode(current).unwrap();
            bytes[19] ^= 1;
            if bytes.iter().all(|byte| *byte == 0) {
                bytes[19] = 1;
            }
            domain.verifying_contract = Address::new(format!("0x{}", hex::encode(bytes))).unwrap();
        }
    }
    domain
}

fn generated_order(rng: &mut CaseRng) -> UnsignedOrder {
    UnsignedOrder {
        sell_token: rng.non_zero_address(),
        buy_token: rng.non_zero_address(),
        receiver: rng.non_zero_address(),
        sell_amount: rng.amount(),
        buy_amount: rng.amount(),
        valid_to: rng.next_u32(),
        app_data: rng.app_data(),
        fee_amount: rng.amount(),
        kind: if rng.next_bool() {
            OrderKind::Sell
        } else {
            OrderKind::Buy
        },
        partially_fillable: rng.next_bool(),
        sell_token_balance: match rng.next_u64() % 3 {
            0 => OrderBalance::Erc20,
            1 => OrderBalance::External,
            _ => OrderBalance::Internal,
        },
        buy_token_balance: if rng.next_bool() {
            OrderBalance::Internal
        } else {
            OrderBalance::Erc20
        },
    }
}

fn generated_cancellations(rng: &mut CaseRng) -> Vec<OrderUid> {
    let len = 1 + (rng.next_u64() % 4) as usize;
    (0..len).map(|_| rng.order_uid()).collect()
}

fn protocol_options_for_chain(
    rng: &mut CaseRng,
    chain: SupportedChainId,
) -> Option<ProtocolOptions> {
    let env = if rng.next_bool() {
        Some(if rng.next_bool() {
            CowEnv::Prod
        } else {
            CowEnv::Staging
        })
    } else {
        None
    };

    let settlement_contract_override = if rng.next_bool() {
        let mut overrides = BTreeMap::new();
        overrides.insert(u64::from(chain), rng.non_zero_address());
        Some(overrides)
    } else {
        None
    };

    if env.is_none() && settlement_contract_override.is_none() {
        None
    } else {
        let mut options = ProtocolOptions::new();
        if let Some(env) = env {
            options = options.with_env(env);
        }
        if let Some(overrides) = settlement_contract_override {
            options = options.with_settlement_contract_override(overrides);
        }
        Some(options)
    }
}

fn decode_u256_word(word: &[u8]) -> usize {
    let bytes: [u8; 8] = word[24..32].try_into().unwrap();
    u64::from_be_bytes(bytes) as usize
}

fn padded_len(len: usize) -> usize {
    if len == 0 {
        0
    } else {
        ((len - 1) / 32 + 1) * 32
    }
}

#[test]
fn domain_separators_change_only_when_the_typed_data_domain_changes() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0x51A0_0001);
        let domain = generated_domain(&mut rng);
        let separator = domain_separator_for(&domain).unwrap();
        let same_separator = domain_separator_for(&domain).unwrap();
        let changed_separator =
            domain_separator_for(&different_domain(domain.clone(), seed)).unwrap();

        assert_eq!(separator, same_separator, "seed {seed}");
        assert_ne!(separator, changed_separator, "seed {seed}");
    }
}

#[test]
fn order_payloads_and_generated_ids_are_deterministic_and_scheme_explicit() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0x51A0_0002);
        let chain = rng.supported_chain();
        let order = generated_order(&mut rng);
        let owner = rng.non_zero_address();

        let payload = order_typed_data_payload(chain, &order, None).unwrap();
        let repeated_payload = order_typed_data_payload(chain, &order, None).unwrap();
        let generated = generate_order_id(chain, &order, &owner, None).unwrap();
        let repeated_generated = generate_order_id(chain, &order, &owner, None).unwrap();
        let expected_digest = hash_order(
            &get_domain(chain, None).unwrap(),
            &cow_sdk_contracts::Order::from(&order),
        )
        .unwrap();

        let typed_signer = MockSigner::new();
        let typed_result =
            sign_order_with_scheme(&order, chain, &typed_signer, SigningScheme::Eip712, None)
                .unwrap();
        let typed_calls = typed_signer.calls.borrow().clone();

        let message_signer = MockSigner::new();
        let message_result =
            sign_order_with_scheme(&order, chain, &message_signer, SigningScheme::EthSign, None)
                .unwrap();
        let message_calls = message_signer.calls.borrow().clone();

        assert_eq!(payload, repeated_payload, "seed {seed}");
        assert_eq!(generated, repeated_generated, "seed {seed}");
        assert_eq!(generated.order_digest, expected_digest, "seed {seed}");
        assert_eq!(
            typed_result.signing_scheme,
            SigningScheme::Eip712,
            "seed {seed}"
        );
        assert_eq!(
            message_result.signing_scheme,
            SigningScheme::EthSign,
            "seed {seed}"
        );
        assert_eq!(typed_calls.typed_data.len(), 1, "seed {seed}");
        assert!(typed_calls.messages.is_empty(), "seed {seed}");
        assert_eq!(message_calls.messages.len(), 1, "seed {seed}");
        assert!(message_calls.typed_data.is_empty(), "seed {seed}");
        assert_eq!(
            format!("0x{}", hex::encode(&message_calls.messages[0])),
            expected_digest.as_str(),
            "seed {seed}"
        );
    }
}

#[test]
fn cancellation_payloads_are_deterministic_and_preserve_scheme_boundaries() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0x51A0_0003);
        let chain = rng.supported_chain();
        let order_uids = generated_cancellations(&mut rng);

        let payload = order_cancellations_typed_data_payload(&order_uids, chain, None).unwrap();
        let repeated_payload =
            order_cancellations_typed_data_payload(&order_uids, chain, None).unwrap();
        let expected_digest = hash_order_cancellations(
            &get_domain(chain, None).unwrap(),
            &OrderCancellations {
                order_uids: order_uids.clone(),
            },
        )
        .unwrap();

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

        assert_eq!(payload, repeated_payload, "seed {seed}");
        assert_eq!(
            typed_result.signing_scheme,
            SigningScheme::Eip712,
            "seed {seed}"
        );
        assert_eq!(
            message_result.signing_scheme,
            SigningScheme::EthSign,
            "seed {seed}"
        );
        assert_eq!(typed_calls.typed_data.len(), 1, "seed {seed}");
        assert!(typed_calls.messages.is_empty(), "seed {seed}");
        assert_eq!(message_calls.messages.len(), 1, "seed {seed}");
        assert!(message_calls.typed_data.is_empty(), "seed {seed}");
        assert_eq!(
            format!("0x{}", hex::encode(&message_calls.messages[0])),
            expected_digest.as_str(),
            "seed {seed}"
        );
    }
}

#[test]
fn typed_order_payloads_preserve_fields_message_and_override_contracts() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0x51A0_0004);
        let chain = rng.supported_chain();
        let order = generated_order(&mut rng);
        let options = protocol_options_for_chain(&mut rng, chain);
        let expected_domain = get_domain(chain, options.as_ref()).unwrap();

        let payload = order_typed_data_payload(chain, &order, options.as_ref()).unwrap();
        let repeated_payload = order_typed_data_payload(chain, &order, options.as_ref()).unwrap();
        let typed = order_typed_data(chain, &order, options.as_ref()).unwrap();

        assert_eq!(payload, repeated_payload, "seed {seed}");
        assert_eq!(payload.primary_type, ORDER_PRIMARY_TYPE, "seed {seed}");
        assert_eq!(typed.primary_type, ORDER_PRIMARY_TYPE, "seed {seed}");
        assert_eq!(payload.domain, expected_domain, "seed {seed}");
        assert_eq!(typed.domain, expected_domain, "seed {seed}");
        assert_eq!(typed.types, payload.types, "seed {seed}");
        assert_eq!(
            payload.types.get(ORDER_PRIMARY_TYPE).unwrap(),
            &order_fields(),
            "seed {seed}"
        );
        assert_eq!(
            payload.types.get("EIP712Domain").unwrap(),
            &domain_fields(),
            "seed {seed}"
        );
        assert_eq!(
            payload.message,
            serde_json::to_string(&order).unwrap(),
            "seed {seed}"
        );
        assert_eq!(typed.message, order, "seed {seed}");

        if let Some(overrides) = options
            .as_ref()
            .and_then(|options| options.settlement_contract_override.as_ref())
        {
            assert_eq!(
                payload.domain.verifying_contract,
                overrides.get(&u64::from(chain)).unwrap().clone(),
                "seed {seed}"
            );
        }
    }
}

#[test]
fn eip1271_payloads_preserve_dynamic_tail_boundaries_across_generated_signatures() {
    const SIGNATURE_LENGTHS: [usize; 8] = [0, 1, 31, 32, 33, 64, 65, 96];

    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0x51A0_0005);
        let order = generated_order(&mut rng);
        let signature_len = SIGNATURE_LENGTHS[(rng.next_u64() as usize) % SIGNATURE_LENGTHS.len()];
        let signature_bytes = rng.bytes(signature_len);
        let signature = format!("0x{}", hex::encode(&signature_bytes));
        let payload = eip1271_signature_payload(&order, &signature).unwrap();
        let encoded = hex::decode(payload.trim_start_matches("0x")).unwrap();

        let offset_word_start = 32 * 12;
        let length_word_start = 32 * 13;
        let data_start = 32 * 14;
        let expected_padding = padded_len(signature_bytes.len());

        assert_eq!(
            &encoded[32 * 6..32 * 7],
            &hex::decode(order.app_data.as_str().trim_start_matches("0x")).unwrap(),
            "seed {seed}"
        );
        assert_eq!(
            decode_u256_word(&encoded[offset_word_start..offset_word_start + 32]),
            32 * 13,
            "seed {seed}"
        );
        assert_eq!(
            decode_u256_word(&encoded[length_word_start..length_word_start + 32]),
            signature_bytes.len(),
            "seed {seed}"
        );
        assert_eq!(encoded.len(), data_start + expected_padding, "seed {seed}");
        assert_eq!(
            &encoded[data_start..data_start + signature_bytes.len()],
            &hex::decode(signature.trim_start_matches("0x")).unwrap(),
            "seed {seed}"
        );
        assert!(
            encoded[data_start + signature_bytes.len()..data_start + expected_padding]
                .iter()
                .all(|byte| *byte == 0),
            "seed {seed}"
        );
    }
}
