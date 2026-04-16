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
use cow_sdk_contracts::hash_order;
use cow_sdk_core::{Address, Amount, AppDataHex, OrderBalance, OrderKind, UnsignedOrder};

let order = UnsignedOrder {
    sell_token: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
    buy_token: Address::new("0x2222222222222222222222222222222222222222").unwrap(),
    receiver: Some(Address::new("0x3333333333333333333333333333333333333333").unwrap()),
    sell_amount: Amount::new("1000000000000000000").unwrap(),
    buy_amount: Amount::new("900000000000000000").unwrap(),
    valid_to: 0,
    app_data: AppDataHex::new("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
    fee_amount: Amount::zero(),
    kind: OrderKind::Sell,
    partially_fillable: false,
    sell_token_balance: OrderBalance::Erc20,
    buy_token_balance: OrderBalance::Erc20,
};

let _digest = hash_order(&order);
```

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
