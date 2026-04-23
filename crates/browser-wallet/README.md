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
use cow_sdk_browser_wallet::MockEip1193Transport;

let _transport = MockEip1193Transport::sepolia().with_label("example wallet");
```

## Where to next

- [Browser-Wallet Example](https://github.com/cowdao-grants/cow-rs/tree/main/examples/wasm/browser-wallet-console)
- [Architecture](https://github.com/cowdao-grants/cow-rs/blob/main/docs/architecture.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
