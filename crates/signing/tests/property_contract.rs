mod common;

use cow_sdk_contracts::{OrderCancellations, SigningScheme, hash_order, hash_order_cancellations};
use cow_sdk_core::{
    Address, Amount, AppDataHex, OrderBalance, OrderKind, OrderUid, SupportedChainId,
    TypedDataDomain, UnsignedOrder,
};
use cow_sdk_signing::{
    domain_separator_for, generate_order_id, get_domain, order_cancellations_typed_data_payload,
    order_typed_data_payload, sign_order_cancellations_with_scheme, sign_order_with_scheme,
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
            domain.verifying_contract =
                Address::new(format!("0x{}", hex::encode(bytes))).unwrap();
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

#[test]
fn domain_separators_change_only_when_the_typed_data_domain_changes() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0x51A0_0001);
        let domain = generated_domain(&mut rng);
        let separator = domain_separator_for(&domain).unwrap();
        let same_separator = domain_separator_for(&domain).unwrap();
        let changed_separator = domain_separator_for(&different_domain(domain.clone(), seed)).unwrap();

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
        let expected_digest =
            hash_order(&get_domain(chain, None).unwrap(), &cow_sdk_contracts::Order::from(&order))
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
        assert_eq!(typed_result.signing_scheme, SigningScheme::Eip712, "seed {seed}");
        assert_eq!(message_result.signing_scheme, SigningScheme::EthSign, "seed {seed}");
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
        assert_eq!(typed_result.signing_scheme, SigningScheme::Eip712, "seed {seed}");
        assert_eq!(message_result.signing_scheme, SigningScheme::EthSign, "seed {seed}");
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
