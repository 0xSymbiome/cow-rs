# cow-sdk-core

Shared [CoW Protocol](https://cow.fi) core types and runtime-neutral
trait contracts.

The crate ships validated primitive types, environment and chain
configuration, and the trait shapes used across the `cow-rs` crate
family. Most consumers reach these types through the top-level
[`cow-sdk`](https://crates.io/crates/cow-sdk) facade re-exports;
depend on this crate directly when you are building a sibling leaf
crate or implementing a custom `Signer` or `Provider` adapter.

The cow-named identity and numeric primitive types ship as cow-owned
`#[repr(transparent)]` newtypes over `alloy_primitives` per
[ADR 0052](https://github.com/0xSymbiome/cow-rs/blob/main/docs/adr/0052-alloy-primitives-canonical-primitive-layer.md).

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
use cow_sdk_core::{Amount, SupportedChainId, wrapped_native_token};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// `Amount` is the typed atomic-quantity boundary. `from_units` builds a
// whole token amount from a number (no string, no zero-counting); reach for
// `parse_units` when the amount is fractional or arrives as text. Both scale
// by `10^decimals` with exact integer arithmetic, and `format_units` is the
// inverse for display.
let one_weth = Amount::from_units(1, 18)?;
assert_eq!(one_weth.to_string(), "1000000000000000000");
assert_eq!(one_weth.format_units(18), "1.000000000000000000");

// Fractional or user-supplied amounts arrive as a decimal string.
let one_and_a_half_weth = Amount::parse_units("1.5", 18)?;
assert_eq!(one_and_a_half_weth.to_string(), "1500000000000000000");

// A chain id drives real configuration: the API path segment used in
// orderbook base URLs, and the wrapped-native token metadata.
let chain = SupportedChainId::Mainnet;
assert_eq!(chain.api_path(), "mainnet");

let weth = wrapped_native_token(chain);
assert_eq!(weth.symbol, "WETH");
assert_eq!(weth.decimals, 18);
# Ok(())
# }
```

## Where to next

- [Getting Started](https://github.com/0xSymbiome/cow-rs/blob/main/docs/getting-started.md)
- [Integrations Guide](https://github.com/0xSymbiome/cow-rs/blob/main/docs/integrations.md)
- [Workspace README](https://github.com/0xSymbiome/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-or-later. See the workspace
[LICENSE](https://github.com/0xSymbiome/cow-rs/blob/main/LICENSE)
file for the full text.
