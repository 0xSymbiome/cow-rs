# cow-sdk-signing

Deterministic [CoW Protocol](https://cow.fi) order hashing, EIP-712 typed
data payload construction, order UID generation, and EIP-1271 helper
surfaces.

This crate owns the canonical typed-data boundary
(`cow_sdk_core::TypedDataPayload`) and the explicit payload construction
paths used by the trading orchestration surface and by browser-wallet
runtime adapters. Most end-user code reaches these helpers through
[`cow-sdk`](https://crates.io/crates/cow-sdk); depend on this crate
directly when implementing custom signer integrations or offline
signing tooling.

## Install

```toml
[dependencies]
cow-sdk-signing = "0.1"
```

## Minimal example

```rust
use cow_sdk_core::{CowEnv, SupportedChainId};
use cow_sdk_signing::domain_separator_for;

let _domain = domain_separator_for(SupportedChainId::Sepolia, CowEnv::Prod);
```

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Integrations Guide](https://github.com/cowdao-grants/cow-rs/blob/main/docs/integrations.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
