# cow-sdk-contracts

Low-level [CoW Protocol](https://cow.fi) contract helpers for order
hashing, settlement encoding, signature codecs, and deployment metadata.

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
    Amount::zero(),
    OrderKind::Sell,
    false,
    SellTokenSource::Erc20,
    BuyTokenDestination::Erc20,
));

let _digest = hash_order(&domain, &order).unwrap();
```

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
