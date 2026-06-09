# cow-sdk-contracts

Low-level [CoW Protocol](https://cow.fi) contract helpers: order hashing and
UID packing, EIP-712 / EIP-1271 signature codecs and on-chain verification,
fail-closed on-chain event decoding, the settlement / eth-flow / token ABI
bindings, deployment metadata, and the opt-in COW Shed account-abstraction
module.

This crate owns the deterministic building blocks used by higher-level crates
such as [`cow-sdk-signing`](https://crates.io/crates/cow-sdk-signing) and
[`cow-sdk-trading`](https://crates.io/crates/cow-sdk-trading). Most end-user
code reaches these helpers through
[`cow-sdk`](https://crates.io/crates/cow-sdk); depend on this crate directly
when you are writing contract-level tooling, encoders, or verifiers that do not
need the full trading facade.

## Install

```toml
[dependencies]
cow-sdk-contracts = "0.1"
```

## Example

```rust
use cow_sdk_contracts::{ContractId, Registry};
use cow_sdk_core::{CowEnv, SupportedChainId};

// Resolve the canonical settlement contract for a chain and environment.
let settlement = Registry::default()
    .address(ContractId::Settlement, SupportedChainId::Mainnet, CowEnv::Prod)
    .unwrap();
```

From there, `hash_order` produces an order's EIP-712 digest, the `signature`
codecs recover and encode EOA and EIP-1271 signatures, and the event decoders
parse settlement and on-chain-order logs fail-closed. See the
[crate documentation](https://docs.rs/cow-sdk-contracts) for the full surface.

## Features

All off by default:

- `cow-shed` — the COW Shed account-abstraction module (`cow_shed`):
  deterministic proxy derivation, EIP-712 hook signing, and `executeHooks`
  calldata, with the `CowShedHooks` orchestrator.
- `cow-shed-gnosis` — adds the Gnosis-only `COWShedForComposableCoW` forwarder.
- `tracing` — structured spans through the
  [`tracing`](https://docs.rs/tracing) facade.

## Primitive layer

`Address`, `Hash32`, `OrderUid`, and `AppDataHash` come from `cow-sdk-core` as
`#[repr(transparent)]` newtypes over the matching `alloy_primitives` types per
[ADR 0052](https://github.com/cowdao-grants/cow-rs/blob/main/docs/adr/0052-alloy-primitives-canonical-primitive-layer.md);
conversion at the alloy seam is zero-cost. EIP-712 domain separators and message
hashes route through `alloy_sol_types`.

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
