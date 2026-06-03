# cow-sdk-browser-wallet

Browser-wallet integration for the [CoW Protocol](https://cow.fi) Rust
SDK. Exposes typed EIP-1193 provider, signer, discovery, and session
contracts for WASM consumers plus a deterministic mock transport for
tests and review flows.

This crate is the browser-runtime leaf of the `cow-rs` package family.
The public API stays Rust-native and typed; raw JavaScript payloads
remain local to the crate. Most consumers reach this crate through the
[`cow-sdk`](https://crates.io/crates/cow-sdk) facade's `browser-wallet`
feature flag; depend on it directly when building a WASM integration
that does not use the trading facade. Injected-wallet behavior is
environment-sensitive: authorization prompts, provider inventory, and
vendor-specific support are controlled by the browser runtime rather
than normalized into universal SDK guarantees.

For TypeScript applications that already use viem, ethers, wagmi, or another
JavaScript wallet stack, prefer `cow-sdk-wasm`. It exposes the SDK through
typed callbacks and keeps JavaScript wallet objects outside Rust trait storage.
This crate remains the Rust-native browser-wallet leaf.

Transaction submission through the browser wallet returns
`TransactionBroadcast` with the hash accepted by the injected provider.
Receipt observation is a separate provider lookup. When an EIP-1193 receipt is
available, this crate populates `TransactionReceipt` fields for status, block,
gas, sender, and recipient; absent optional fields remain empty, while present
malformed fields fail closed with a typed browser-wallet error.

Typed-data signing consumes `cow_sdk_core::TypedDataDomain` directly; the
cow struct emits the canonical EIP-1193 `eth_signTypedData_v4`
second-parameter wire shape through its own `Serialize` impl per
[ADR 0052](https://github.com/cowdao-grants/cow-rs/blob/main/docs/adr/0052-alloy-primitives-canonical-primitive-layer.md),
pinned by `PROP-BWL-007` against
`parity/fixtures/signing/eth_sign_typed_data_request.json`.

## Install

```toml
[dependencies]
cow-sdk-browser-wallet = "0.1"
```

Or enable the feature through the facade:

```toml
[dependencies]
cow-sdk = { version = "0.1", features = ["browser-wallet"] }
```

## Minimal example

```rust
use cow_sdk_browser_wallet::{BrowserWallet, MockEip1193Transport, Origin};

let transport = MockEip1193Transport::sepolia().with_label("example wallet");
let origin = Origin::new("test://example-wallet").expect("example origin must be valid");
let _wallet = BrowserWallet::from_trusted_transport(transport, origin)
    .expect("trusted example transport must build");
```

## Where to next

- [Browser-Wallet Example](https://github.com/cowdao-grants/cow-rs/tree/main/examples/wasm/cow-trader-dioxus)
- [cow-sdk-wasm README](https://github.com/cowdao-grants/cow-rs/blob/main/crates/wasm/README.md)
- [Architecture](https://github.com/cowdao-grants/cow-rs/blob/main/docs/architecture.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
