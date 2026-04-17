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

use cow_sdk_contracts::{
    Eip1271SignatureData, Order, OrderFlags, Signature, SigningScheme, TokenRegistry,
    TradeExecution, TradeFlags, decode_eip1271_signature_data, decode_order, decode_order_flags,
    decode_trade_flags, encode_eip1271_signature_data, encode_order_flags, encode_trade,
    encode_trade_flags, hash_order, normalize_order, normalized_ecdsa_signature,
};
use cow_sdk_core::{Address, Amount, AppDataHex, OrderBalance, OrderKind, TypedDataDomain};

const CASE_COUNT: u64 = 128;
const SEARCH_CASE_COUNT: u64 = 512;

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

    fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
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

    fn signature_data(&mut self) -> String {
        format!("0x{}", hex::encode(self.fill::<65>()))
    }

    fn bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.next_u64() as u8).collect()
    }

    fn mixed_case_hex_payload(&mut self, len: usize) -> String {
        let encoded = hex::encode(self.bytes(len));
        let mixed = encoded
            .chars()
            .enumerate()
            .map(|(index, ch)| {
                if index.is_multiple_of(2) {
                    ch.to_ascii_uppercase()
                } else {
                    ch
                }
            })
            .collect::<String>();
        format!("0x{mixed}")
    }
}

fn generated_domain(rng: &mut CaseRng) -> TypedDataDomain {
    TypedDataDomain {
        name: format!("Gnosis Protocol {}", rng.next_u32()),
        version: format!("v{}", (rng.next_u32() % 5) + 1),
        chain_id: u64::from(rng.next_u32()) + 1,
        verifying_contract: rng.non_zero_address(),
    }
}

fn generated_order(rng: &mut CaseRng) -> Order {
    let sell_token_balance = match rng.next_u64() % 3 {
        0 => OrderBalance::Erc20,
        1 => OrderBalance::External,
        _ => OrderBalance::Internal,
    };
    let buy_token_balance = if rng.next_bool() {
        OrderBalance::Internal
    } else {
        OrderBalance::Erc20
    };

    Order {
        sell_token: rng.non_zero_address(),
        buy_token: rng.non_zero_address(),
        receiver: Some(rng.non_zero_address()),
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
        sell_token_balance: Some(sell_token_balance),
        buy_token_balance: Some(buy_token_balance),
    }
}

fn semantically_equivalent_order(order: &Order, rng: &mut CaseRng) -> Order {
    let mut equivalent = order.clone();

    if order.sell_token_balance == Some(OrderBalance::Erc20) && rng.next_bool() {
        equivalent.sell_token_balance = None;
    }

    equivalent.buy_token_balance = match order.buy_token_balance {
        Some(OrderBalance::Internal) => Some(OrderBalance::Internal),
        Some(OrderBalance::Erc20) | Some(OrderBalance::External) | None => match rng.next_u64() % 3
        {
            0 => None,
            1 => Some(OrderBalance::Erc20),
            _ => Some(OrderBalance::External),
        },
    };

    equivalent
}

fn signature_for_scheme(rng: &mut CaseRng, scheme: SigningScheme) -> Signature {
    match scheme {
        SigningScheme::Eip712 | SigningScheme::EthSign => Signature::Ecdsa {
            scheme,
            data: rng.signature_data(),
        },
        SigningScheme::Eip1271 => Signature::Eip1271 {
            data: Eip1271SignatureData {
                verifier: rng.non_zero_address(),
                signature: rng.signature_data(),
            },
        },
        SigningScheme::PreSign => Signature::PreSign {
            owner: rng.non_zero_address(),
        },
        _ => panic!("unsupported generated signing scheme: {scheme:?}"),
    }
}

fn generated_sell_balance(rng: &mut CaseRng) -> OrderBalance {
    match rng.next_u64() % 3 {
        0 => OrderBalance::Erc20,
        1 => OrderBalance::External,
        _ => OrderBalance::Internal,
    }
}

fn generated_buy_balance(rng: &mut CaseRng) -> OrderBalance {
    match rng.next_u64() % 3 {
        0 => OrderBalance::Erc20,
        1 => OrderBalance::External,
        _ => OrderBalance::Internal,
    }
}

fn canonical_buy_balance(balance: OrderBalance) -> OrderBalance {
    match balance {
        OrderBalance::Internal => OrderBalance::Internal,
        OrderBalance::Erc20 | OrderBalance::External => OrderBalance::Erc20,
    }
}

fn search_signature_len(case: u64, rng: &mut CaseRng) -> usize {
    const BOUNDARY_LENGTHS: [usize; 18] = [
        0, 1, 2, 15, 16, 31, 32, 33, 47, 48, 63, 64, 65, 95, 96, 97, 127, 128,
    ];

    match case % 4 {
        0 => BOUNDARY_LENGTHS[((case / 4) as usize) % BOUNDARY_LENGTHS.len()],
        1 => 1 + (rng.next_u64() % 96) as usize,
        2 => 64 + (rng.next_u64() % 193) as usize,
        _ => (rng.next_u64() % 321) as usize,
    }
}

#[test]
fn order_hashing_is_deterministic_for_equivalent_normalized_inputs() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0xC011_AA01);
        let domain = generated_domain(&mut rng);
        let order = generated_order(&mut rng);
        let equivalent = semantically_equivalent_order(&order, &mut rng);

        let normalized = normalize_order(&order).unwrap();
        let equivalent_normalized = normalize_order(&equivalent).unwrap();
        let hash = hash_order(&domain, &order).unwrap();
        let equivalent_hash = hash_order(&domain, &equivalent).unwrap();

        assert_eq!(normalized, equivalent_normalized, "seed {seed}");
        assert_eq!(hash, equivalent_hash, "seed {seed}");
        assert_eq!(hash_order(&domain, &order).unwrap(), hash, "seed {seed}");
    }
}

#[test]
fn encoded_trades_preserve_the_normalized_order_boundary() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0xC011_AA02);
        let order = generated_order(&mut rng);
        let normalized = normalize_order(&order).unwrap();
        let execution = TradeExecution {
            executed_amount: rng.amount(),
        };
        let scheme = match rng.next_u64() % 4 {
            0 => SigningScheme::Eip712,
            1 => SigningScheme::EthSign,
            2 => SigningScheme::Eip1271,
            _ => SigningScheme::PreSign,
        };
        let signature = signature_for_scheme(&mut rng, scheme);
        let mut tokens = TokenRegistry::new();

        let trade = encode_trade(&mut tokens, &normalized, &signature, &execution).unwrap();
        let decoded_flags = decode_trade_flags(trade.flags).unwrap();
        let decoded_order = decode_order(&trade, &tokens.addresses()).unwrap();

        assert_eq!(decoded_flags.kind, normalized.kind, "seed {seed}");
        assert_eq!(
            decoded_flags.partially_fillable, normalized.partially_fillable,
            "seed {seed}"
        );
        assert_eq!(
            decoded_flags.sell_token_balance, normalized.sell_token_balance,
            "seed {seed}"
        );
        assert_eq!(
            decoded_flags.buy_token_balance, normalized.buy_token_balance,
            "seed {seed}"
        );
        assert_eq!(
            trade.executed_amount, execution.executed_amount,
            "seed {seed}"
        );
        assert_eq!(
            normalize_order(&decoded_order).unwrap(),
            normalized,
            "seed {seed}"
        );
    }
}

#[test]
fn compact_flag_codecs_roundtrip_across_generated_variants() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0xC011_AA03);
        let order_flags = OrderFlags {
            kind: if rng.next_bool() {
                OrderKind::Sell
            } else {
                OrderKind::Buy
            },
            partially_fillable: rng.next_bool(),
            sell_token_balance: generated_sell_balance(&mut rng),
            buy_token_balance: generated_buy_balance(&mut rng),
        };

        let encoded_order = encode_order_flags(&order_flags).unwrap();
        let decoded_order = decode_order_flags(encoded_order).unwrap();
        assert_eq!(
            decoded_order,
            OrderFlags {
                buy_token_balance: canonical_buy_balance(order_flags.buy_token_balance),
                ..order_flags.clone()
            },
            "seed {seed}"
        );
        assert_eq!(
            encode_order_flags(&decoded_order).unwrap(),
            encoded_order,
            "seed {seed}"
        );

        let trade_flags = TradeFlags {
            kind: order_flags.kind,
            partially_fillable: order_flags.partially_fillable,
            sell_token_balance: order_flags.sell_token_balance,
            buy_token_balance: order_flags.buy_token_balance,
            signing_scheme: match rng.next_u64() % 4 {
                0 => SigningScheme::Eip712,
                1 => SigningScheme::EthSign,
                2 => SigningScheme::Eip1271,
                _ => SigningScheme::PreSign,
            },
        };

        let encoded_trade = encode_trade_flags(&trade_flags).unwrap();
        let decoded_trade = decode_trade_flags(encoded_trade).unwrap();
        assert_eq!(encoded_trade & 0b1000_0000, 0, "seed {seed}");
        assert_eq!(
            decoded_trade,
            TradeFlags {
                buy_token_balance: canonical_buy_balance(trade_flags.buy_token_balance),
                ..trade_flags.clone()
            },
            "seed {seed}"
        );
        assert_eq!(
            encode_trade_flags(&decoded_trade).unwrap(),
            encoded_trade,
            "seed {seed}"
        );
    }
}

#[test]
fn signature_codecs_preserve_verifier_and_payload_bytes() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed ^ 0xC011_AA04);
        let byte_len = (rng.next_u64() % 97) as usize;
        let signature = rng.mixed_case_hex_payload(byte_len);
        let verifier = rng.non_zero_address();

        let normalized = normalized_ecdsa_signature(&signature).unwrap();
        assert_eq!(normalized, normalized.to_ascii_lowercase(), "seed {seed}");
        assert_eq!(
            hex::decode(normalized.trim_start_matches("0x")).unwrap(),
            hex::decode(signature.trim_start_matches("0x")).unwrap(),
            "seed {seed}"
        );

        let encoded = encode_eip1271_signature_data(&Eip1271SignatureData {
            verifier: verifier.clone(),
            signature: signature.clone(),
        })
        .unwrap();
        let decoded = decode_eip1271_signature_data(&encoded).unwrap();

        assert_eq!(decoded.verifier, verifier, "seed {seed}");
        assert_eq!(decoded.signature, normalized, "seed {seed}");
        assert_eq!(encoded.len(), 2 + ((20 + byte_len) * 2), "seed {seed}");
    }
}

#[test]
fn abi_layout_narrow_search_profile_preserves_eip1271_payload_boundaries() {
    for case in 0..SEARCH_CASE_COUNT {
        let mut rng = CaseRng::new(case ^ 0xC011_AA05);
        let byte_len = search_signature_len(case, &mut rng);
        let signature = rng.mixed_case_hex_payload(byte_len);
        let verifier = rng.non_zero_address();
        let encoded = encode_eip1271_signature_data(&Eip1271SignatureData {
            verifier: verifier.clone(),
            signature: signature.clone(),
        })
        .unwrap();
        let decoded = decode_eip1271_signature_data(&encoded).unwrap();
        let encoded_bytes = hex::decode(encoded.trim_start_matches("0x")).unwrap();
        let verifier_bytes = hex::decode(verifier.as_str().trim_start_matches("0x")).unwrap();
        let signature_bytes = hex::decode(signature.trim_start_matches("0x")).unwrap();

        assert_eq!(encoded_bytes.len(), 20 + byte_len, "case {case}");
        assert_eq!(
            &encoded_bytes[..20],
            verifier_bytes.as_slice(),
            "case {case}"
        );
        assert_eq!(
            &encoded_bytes[20..],
            signature_bytes.as_slice(),
            "case {case}"
        );
        assert_eq!(decoded.verifier, verifier, "case {case}");
        assert_eq!(
            decoded.signature,
            normalized_ecdsa_signature(&signature).unwrap(),
            "case {case}"
        );
        assert_eq!(
            hex::decode(decoded.signature.trim_start_matches("0x")).unwrap(),
            signature_bytes,
            "case {case}"
        );
    }
}
