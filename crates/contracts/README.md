# cow-sdk-contracts

Low-level [CoW Protocol](https://cow.fi) contract helpers for order
hashing, settlement encoding, signature codecs, fail-closed on-chain order
event decoding, wrapped-native interactions, and deployment metadata.

This crate owns the deterministic building blocks used by higher-level
crates such as [`cow-sdk-signing`](https://crates.io/crates/cow-sdk-signing)
and [`cow-sdk-trading`](https://crates.io/crates/cow-sdk-trading). Most
end-user code reaches these helpers through
[`cow-sdk`](https://crates.io/crates/cow-sdk); depend on this crate
directly when you are writing contract-level tooling, encoders, or
verifiers that do not need the full trading facade.

## Install

```toml
[dependencies]
cow-sdk-contracts = "0.1"
```

## Minimal example

```rust
use cow_sdk_contracts::{ContractId, Order, Registry, hash_order};
use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, CowEnv, OrderKind, SellTokenSource,
    SupportedChainId, TypedDataDomain, UnsignedOrder,
};

let verifying_contract = Registry::default()
    .address(ContractId::Settlement, SupportedChainId::Mainnet, CowEnv::Prod)
    .unwrap();
let domain = TypedDataDomain::new(
    "Gnosis Protocol".to_owned(),
    "v2".to_owned(),
    SupportedChainId::Mainnet.into(),
    verifying_contract,
);
let trader_address = Address::new("0x3333333333333333333333333333333333333333").unwrap();

let order = Order::from(&UnsignedOrder::new(
    Address::new("0x1111111111111111111111111111111111111111").unwrap(),
    Address::new("0x2222222222222222222222222222222222222222").unwrap(),
    trader_address,
    Amount::new("1000000000000000000").unwrap(),
    Amount::new("900000000000000000").unwrap(),
    0,
    AppDataHash::new("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
    Amount::ZERO,
    OrderKind::Sell,
    false,
    SellTokenSource::Erc20,
    BuyTokenDestination::Erc20,
));

let _digest = hash_order(&domain, &order).unwrap();
```

## Primitive Layer

The `cow_sdk_contracts` crate consumes `Address`, `Hash32`, `OrderUid`,
and `AppDataHash` from `cow_sdk_core::types::*` as cow-owned
`#[repr(transparent)]` newtypes around the corresponding `alloy_primitives`
type per [ADR 0052](https://github.com/cowdao-grants/cow-rs/blob/main/docs/adr/0052-alloy-primitives-canonical-primitive-layer.md).
The cow newtype layer preserves the Rust type-system distinction between
same-width byte primitives while keeping bit-for-bit layout compatibility
with the underlying alloy primitive; conversion at the alloy seam is
zero-cost via `.0` access or `From::from(...)`.

`alloy_sol_types::sol!`-generated structs consume cow newtype values
through the bit-compatible bridge. EIP-712 domain separators route through
`alloy_sol_types::Eip712Domain::separator`, and EIP-712 message hashes
route through `alloy_sol_types::SolStruct::eip712_signing_hash(&domain)`.

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
