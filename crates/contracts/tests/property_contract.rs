use cow_sdk_contracts::{
    Eip1271SignatureData, Order, Signature, SigningScheme, TokenRegistry, TradeExecution,
    decode_order, decode_trade_flags, encode_trade, hash_order, normalize_order,
};
use cow_sdk_core::{Address, Amount, AppDataHex, OrderBalance, OrderKind, TypedDataDomain};

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
            decoded_flags.partially_fillable,
            normalized.partially_fillable,
            "seed {seed}"
        );
        assert_eq!(
            decoded_flags.sell_token_balance,
            normalized.sell_token_balance,
            "seed {seed}"
        );
        assert_eq!(
            decoded_flags.buy_token_balance,
            normalized.buy_token_balance,
            "seed {seed}"
        );
        assert_eq!(trade.executed_amount, execution.executed_amount, "seed {seed}");
        assert_eq!(
            normalize_order(&decoded_order).unwrap(),
            normalized,
            "seed {seed}"
        );
    }
}
