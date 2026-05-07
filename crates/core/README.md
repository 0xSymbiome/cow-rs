# cow-sdk-core

Shared [CoW Protocol](https://cow.fi) core types, validated primitives,
configuration, and runtime-neutral trait contracts used across the
`cow-rs` crate family.

This is a foundational crate. Most consumers reach these types through
the top-level [`cow-sdk`](https://crates.io/crates/cow-sdk) facade
re-exports; depend on this crate directly when you are building a sibling
leaf crate or implementing a custom `Signer` or `Provider` adapter.

The core runtime traits split transaction lifecycle states explicitly:
`TransactionBroadcast` is the signer-side broadcast acknowledgement, while
`TransactionReceipt` is the provider-side mined observation shape with optional
status, block, gas, sender, and recipient fields.

## Install

```toml
[dependencies]
cow-sdk-core = "0.1"
```

## Minimal example

```rust
use cow_sdk_core::{Address, SupportedChainId, addresses_equal};

let address = Address::new("0x1111111111111111111111111111111111111111").unwrap();
assert_eq!(address.normalized_key(), address.as_str());
assert!(addresses_equal(&address, &address));
let _chain = SupportedChainId::Sepolia;
```

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Integrations Guide](https://github.com/cowdao-grants/cow-rs/blob/main/docs/integrations.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
