#![no_main]

//! Fuzz target for the `OrderBoundsValidator::validate` boundary.
//!
//! **Surface:** `cow_sdk_trading::OrderBoundsValidator::validate`, plus the
//! adjacent owner/signer assertion used by the same rejection enum.
//! **Property:** `PROP-TRD-008`.
//! **Seed contract:** corpus inputs cover the happy path, every validator
//! rejection class, timestamp extremes, and WETH/native sentinel pairing.
//! **Corpus README:** `../corpus/fuzz_order_bounds_validator/README.md`.
//!
//! The fuzzer maps arbitrary bytes into an
//! `(OrderCreation, SigningScheme, Option<Address>, u64, bool)` tuple and
//! runs the tuple through the services-default validator. A small seed-class
//! byte keeps committed corpus files reproducible while the remaining bytes
//! still perturb addresses, amounts, scheme, time, and path flags.

use cow_sdk_core::{Address, Amount, EVM_NATIVE_CURRENCY_ADDRESS, OrderKind, ValidationReason};
use cow_sdk_orderbook::{OrderCreation, SigningScheme};
use cow_sdk_trading::{
    ClientRejection, OrderBoundsValidator, validation::assert_owner_matches_signer,
};
use libfuzzer_sys::{
    arbitrary::{Arbitrary, Unstructured},
    fuzz_target,
};

const DEFAULT_NOW: u64 = 1_700_000_000;
const DEFAULT_VALID_FOR: u64 = 3_600;
const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

#[derive(Debug)]
struct ValidatorInput {
    order: OrderCreation,
    scheme: SigningScheme,
    app_data_signer: Option<Address>,
    now: u64,
    is_eth_flow: bool,
    partner_fee_probe: bool,
}

impl ValidatorInput {
    fn into_tuple(self) -> (OrderCreation, SigningScheme, Option<Address>, u64, bool) {
        (
            self.order,
            self.scheme,
            self.app_data_signer,
            self.now,
            self.is_eth_flow,
        )
    }
}

impl<'a> Arbitrary<'a> for ValidatorInput {
    fn arbitrary(bytes: &mut Unstructured<'a>) -> libfuzzer_sys::arbitrary::Result<Self> {
        let seed_class = seed_class(read_u8(bytes, 0));
        let mut now = bounded_now(read_u64(bytes, DEFAULT_NOW));
        let mut scheme = signing_scheme(read_u8(bytes, 0));
        let mut is_eth_flow = read_bool(bytes, false);
        let mut order = base_order(now, scheme);

        order.sell_token = address_from_bytes(read_address_bytes(bytes, 0xaa));
        order.buy_token = address_from_bytes(read_address_bytes(bytes, 0xbb));
        order.sell_amount = amount_from_u128(read_u128(bytes, 1_000_000_000_000_000_000));
        order.buy_amount = amount_from_u128(read_u128(bytes, 1_000_000));
        order.valid_to = valid_to_after(now, DEFAULT_VALID_FOR);
        order.from = address_from_bytes(read_address_bytes(bytes, 0x11));
        order.kind = if read_bool(bytes, false) {
            OrderKind::Buy
        } else {
            OrderKind::Sell
        };

        let mut app_data_signer = if read_bool(bytes, false) {
            Some(address_from_bytes(read_address_bytes(bytes, 0x22)))
        } else {
            None
        };
        let mut partner_fee_probe = false;

        match seed_class {
            0 => {
                order = base_order(now, scheme);
                app_data_signer = None;
                is_eth_flow = false;
            }
            1 => order.from = zero_address(),
            2 => order.valid_to = valid_to_after(now, 59),
            3 => {
                now = 0;
                scheme = SigningScheme::Eip712;
                order.valid_to = u32::MAX;
            }
            4 => {
                order.sell_token = native_sentinel();
                is_eth_flow = false;
            }
            5 => order.buy_token = order.sell_token.clone(),
            6 => order.sell_amount = Amount::ZERO,
            7 => order.buy_amount = Amount::ZERO,
            8 => app_data_signer = Some(address_from_seed(0x33)),
            9 => app_data_signer = Some(address_from_seed(0x44)),
            10 => partner_fee_probe = true,
            11 => {
                order.valid_to = u32::MAX;
                now = u64::from(u32::MAX) - 1;
            }
            12 => {
                order.valid_to = u32::MAX;
                now = u64::MAX;
            }
            13 => {
                order.sell_token = weth_address();
                order.buy_token = native_sentinel();
                is_eth_flow = false;
            }
            _ => {
                order.sell_token = native_sentinel();
                is_eth_flow = true;
            }
        }

        Ok(Self {
            order,
            scheme,
            app_data_signer,
            now,
            is_eth_flow,
            partner_fee_probe,
        })
    }
}

fuzz_target!(|input: ValidatorInput| {
    let partner_fee_probe = input.partner_fee_probe;
    let (order, scheme, app_data_signer, now, is_eth_flow) = input.into_tuple();
    let validator = OrderBoundsValidator::services_default().with_weth_address(weth_address());

    let validation = validator.validate(&order, scheme, app_data_signer.clone(), now, is_eth_flow);
    assert_well_defined(&validation);

    if let Some(recovered) = app_data_signer.as_ref() {
        let owner_check = assert_owner_matches_signer(&order.from, recovered);
        assert_well_defined(&owner_check);
    }

    if partner_fee_probe {
        let partner_fee = Err(ClientRejection::InvalidPartnerFee {
            field: "bps",
            reason: ValidationReason::OutOfRange {
                details: "partner fee basis points must fit u16 bounds",
            },
        });
        assert_well_defined(&partner_fee);
    }
});

fn assert_well_defined(outcome: &Result<(), ClientRejection>) {
    match outcome {
        Ok(()) => {}
        Err(ClientRejection::ValidToInsufficient { .. })
        | Err(ClientRejection::ValidToExcessive { .. })
        | Err(ClientRejection::MissingFrom)
        | Err(ClientRejection::AppdataFromMismatch { .. })
        | Err(ClientRejection::SameBuyAndSellToken { .. })
        | Err(ClientRejection::InvalidNativeSellToken)
        | Err(ClientRejection::ZeroAmount { .. })
        | Err(ClientRejection::OwnerMismatch { .. })
        | Err(ClientRejection::InvalidPartnerFee { .. }) => {}
        Err(other) => panic!(
            "OrderBoundsValidator returned an unenumerated ClientRejection variant; \
             extend the typed match in fuzz_order_bounds_validator before accepting \
             new variants on the public surface: {other:?}"
        ),
    }
    if let Err(rejection) = outcome {
        let display = rejection.to_string();
        assert!(
            !display.is_empty(),
            "typed rejection display must stay non-empty"
        );
        assert!(
            !display.contains('\n') && !display.contains('\0'),
            "typed rejection display must not carry raw newline or null bytes: {display}"
        );
    }
}

fn seed_class(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'e' => 10 + (value - b'a'),
        b'A'..=b'E' => 10 + (value - b'A'),
        _ => value % 15,
    }
}

fn read_u8(bytes: &mut Unstructured<'_>, default: u8) -> u8 {
    u8::arbitrary(bytes).unwrap_or(default)
}

fn read_bool(bytes: &mut Unstructured<'_>, default: bool) -> bool {
    bool::arbitrary(bytes).unwrap_or(default)
}

fn read_u64(bytes: &mut Unstructured<'_>, default: u64) -> u64 {
    u64::arbitrary(bytes).unwrap_or(default)
}

fn read_u128(bytes: &mut Unstructured<'_>, default: u128) -> u128 {
    u128::arbitrary(bytes).unwrap_or(default)
}

fn read_address_bytes(bytes: &mut Unstructured<'_>, fallback: u8) -> [u8; 20] {
    <[u8; 20]>::arbitrary(bytes).unwrap_or([fallback; 20])
}

fn bounded_now(now: u64) -> u64 {
    let max_now = u64::from(u32::MAX) - DEFAULT_VALID_FOR;
    now % (max_now + 1)
}

fn valid_to_after(now: u64, valid_for: u64) -> u32 {
    let now = now.min(u64::from(u32::MAX).saturating_sub(valid_for));
    u32::try_from(now + valid_for).unwrap_or(u32::MAX)
}

fn base_order(now: u64, scheme: SigningScheme) -> OrderCreation {
    OrderCreation::new(
        address_from_seed(0xaa),
        address_from_seed(0xbb),
        amount_from_u128(1_000_000_000_000_000_000),
        amount_from_u128(1_000_000),
        valid_to_after(now, DEFAULT_VALID_FOR),
        OrderKind::Sell,
        scheme,
        "0x",
        address_from_seed(0x11),
    )
}

fn signing_scheme(value: u8) -> SigningScheme {
    match value % 4 {
        0 => SigningScheme::Eip712,
        1 => SigningScheme::EthSign,
        2 => SigningScheme::Eip1271,
        _ => SigningScheme::PreSign,
    }
}

fn amount_from_u128(value: u128) -> Amount {
    Amount::new(value.to_string()).expect("u128 string must remain a valid amount")
}

fn address_from_seed(seed: u8) -> Address {
    address_from_bytes([seed; 20])
}

fn address_from_bytes(bytes: [u8; 20]) -> Address {
    Address::from_bytes(bytes)
}

fn zero_address() -> Address {
    address_from_bytes([0u8; 20])
}

fn native_sentinel() -> Address {
    Address::new(EVM_NATIVE_CURRENCY_ADDRESS)
        .expect("native sentinel literal must remain a valid address")
}

fn weth_address() -> Address {
    Address::new(WETH).expect("WETH literal must remain a valid address")
}
