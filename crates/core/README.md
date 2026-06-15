# cow-sdk-core

Shared [CoW Protocol](https://cow.fi) core types and runtime-neutral
trait contracts.

> ⚠️ **Alpha — `0.1.0-alpha`.** Pre-release and not security-audited; the public
> API may change before `0.1.0`. It is published as a pre-release, so Cargo
> selects it only when you opt in (`cow-sdk-core = "0.1.0-alpha.1"`). Review it
> yourself before relying on it with real funds.

`cow-sdk-core` is the foundation of the
[`cow-rs`](https://github.com/0xSymbiome/cow-rs) crate family: the shared
vocabulary and runtime-neutral seams every other crate is built on. It ships
validated primitive types, environment and chain configuration, a uniform error
taxonomy, and the trait shapes used across the workspace. It performs no protocol
work of its own — it defines the types and boundaries the rest of the SDK fills
in. Most consumers reach these types through the top-level
[`cow-sdk`](https://crates.io/crates/cow-sdk) facade re-exports; depend on this
crate directly when you are building a sibling leaf crate or implementing a custom
`Signer`, `Provider`, or `HttpTransport` adapter.

The cow-named identity and numeric primitive types ship as cow-owned
`#[repr(transparent)]` newtypes over `alloy_primitives` per
[ADR 0052](https://github.com/0xSymbiome/cow-rs/blob/main/docs/adr/0052-alloy-primitives-canonical-primitive-layer.md).

## What it provides

- **Identity & numeric newtypes** — `Address`, `Hash32`, `AppDataHash`, `HexData`,
  `OrderUid`, `OrderData`, `OrderKind`, `Amount`/`Amounts`, `TokenInfo`, and the
  validity-window types, as cow-owned transparent newtypes over `alloy_primitives`.
- **The `address!` macro** — compile-time-validated address literals. The literal
  must be the lowercase wire form; a mixed-case literal fails the build, because an
  EIP-55 checksum cannot be verified in const evaluation. A malformed address never
  reaches runtime.
- **Runtime-neutral trait seams** — `Signer`, `DigestSigner`, `TypedDataSigner`,
  `Provider`, `SigningProvider`, `LogProvider`, and the object-safe `HttpTransport`,
  plus `UserRejection`, the bound that lets the SDK tell a user-declined signature
  apart from other signer failures. `async_trait` is re-exported for implementors.
- **Explicit transaction lifecycle states** — `TransactionBroadcast` is the
  signer-side broadcast acknowledgement; `TransactionReceipt` is the provider-side
  mined observation with optional status, block, gas, sender, and recipient fields.
- **Chain & endpoint configuration** — `SupportedChainId`, default API base URLs,
  canonical orderbook/subgraph host allow-lists, `NATIVE_CURRENCY_ADDRESS`,
  `wrapped_native_token`, `ProtocolOptions`, and `ExternalHostPolicy` for
  SSRF-resistant URL validation.
- **Uniform error taxonomy** — `ErrorClass`, the coarse telemetry bucket every
  crate's error maps to, alongside `CoreError`, `ValidationError`, and
  `TransportErrorClass`.
- **Secret redaction** — `Redacted<T>` and the redacted URL-map types, so
  secret-bearing configuration never leaks through `Debug`, `Display`, or `serde`.
- **Cooperative cancellation** — a re-exported `CancellationToken` plus the
  `Cancellable` / `WithCancellation` combinators for long-running SDK futures.
- **The HTTP transport seam** — the `HttpTransport` async injection point with the
  native `ReqwestTransport` default (size-capped, URL-stripping on error); the
  browser default lives in `cow-sdk-transport-wasm`.

## Install

```toml
[dependencies]
cow-sdk-core = "0.1.0-alpha.1"
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

## Feature flags

| Feature | Default | Enables |
| --- | --- | --- |
| `transport-policy` | off | Shared HTTP retry, rate-limit, `Retry-After`, jitter, and transport-error classification policy used by the orderbook, subgraph, and IPFS clients. Off by default so a consumer that needs only the primitive types does not pull the retry-timer dependencies. |
| `tracing` | off | Emits `tracing` spans and events from the transport layer. |

## Where this fits

This crate defines types and seams; it does not compute order hashes or signatures
(see [`cow-sdk-signing`](https://crates.io/crates/cow-sdk-signing)), talk to the
orderbook (see [`cow-sdk-orderbook`](https://crates.io/crates/cow-sdk-orderbook)),
build or submit trades (see [`cow-sdk-trading`](https://crates.io/crates/cow-sdk-trading)),
or decode contract events (see [`cow-sdk-contracts`](https://crates.io/crates/cow-sdk-contracts)).
The `Signer` and `Provider` seams are defined here; concrete adapters live in
[`cow-sdk-alloy-signer`](https://crates.io/crates/cow-sdk-alloy-signer),
[`cow-sdk-alloy-provider`](https://crates.io/crates/cow-sdk-alloy-provider), and
[`cow-sdk-browser-wallet`](https://crates.io/crates/cow-sdk-browser-wallet).

## Where to next

- [Getting Started](https://github.com/0xSymbiome/cow-rs/blob/main/docs/getting-started.md)
- [Integrations Guide](https://github.com/0xSymbiome/cow-rs/blob/main/docs/integrations.md)
- [Workspace README](https://github.com/0xSymbiome/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-or-later. See the workspace
[LICENSE](https://github.com/0xSymbiome/cow-rs/blob/main/LICENSE)
file for the full text.
