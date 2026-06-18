# cow-sdk-contracts

Low-level [CoW Protocol](https://cow.fi) contract helpers: order hashing and
UID packing, EIP-712 / EIP-1271 signature codecs and on-chain verification,
fail-closed on-chain event decoding, the settlement / eth-flow / token ABI
bindings, deployment metadata, and the opt-in COW Shed account-abstraction
module.

> ⚠️ **Alpha — `0.1.0-alpha`.** Pre-release and not security-audited; the public
> API may change before `0.1.0`. It is published as a pre-release, so Cargo
> selects it only when you opt in (`cow-sdk-contracts = "0.1.0-alpha.5"`). Review
> it yourself before relying on it with real funds.

This crate owns the deterministic building blocks used by higher-level crates
such as [`cow-sdk-signing`](https://crates.io/crates/cow-sdk-signing) and
[`cow-sdk-trading`](https://crates.io/crates/cow-sdk-trading). Most end-user
code reaches these helpers through
[`cow-sdk`](https://crates.io/crates/cow-sdk); depend on this crate directly
when you are writing contract-level tooling, encoders, or verifiers that do not
need the full trading facade.

## What it provides

- **Order hashing and UID codec** — `hash_order` and `hash_order_cancellation(s)`
  EIP-712 digests, `order_eip712_type_hash`, and the 56-byte UID codec
  (`compute_order_uid`, `pack_order_uid_params`, `extract_order_uid_params`).
- **Signature codecs and recovery** — a closed-construction `RecoverableSignature`
  (ECDSA recovery, ERC-2098 round-trip, EIP-191 prehash), the `Signature` enum,
  the `SigningScheme` enum, and EIP-1271 payload encode/decode.
- **On-chain EIP-1271 verification** — `verify_eip1271_signature(_cached)` through
  an injected `Provider`, with an optional `Eip1271Cache`.
- **Fail-closed event decoding** — `OrderPlacement` / `OrderInvalidation`,
  settlement, and eth-flow log decoders that validate the topic set and field
  lengths before ABI decode and return a typed error rather than panicking on
  hostile input.
- **ABI bindings and calldata** — eth-flow create/invalidate, ERC-20, and
  wrapped-native (WETH) wrap/unwrap interactions.
- **Deployment registry** — `Registry::address` resolves Settlement,
  VaultRelayer, and EthFlow across the supported chains and prod/staging from
  committed const addresses, no RPC.
- **(feature `cow-shed`)** account-abstraction proxy derivation, EIP-712 hook
  signing, and `executeHooks` calldata via the `CowShedHooks` orchestrator.

## Install

```toml
[dependencies]
cow-sdk-contracts = "0.1.0-alpha.5"
```

## Example

```rust
use cow_sdk_contracts::{ContractId, Registry};
use cow_sdk_core::{CowEnv, SupportedChainId};

// Resolve the canonical settlement contract for a chain and environment.
let settlement = Registry::default()
    .address(ContractId::Settlement, SupportedChainId::Mainnet, CowEnv::Prod)
    .unwrap();
assert_eq!(
    settlement.to_hex_string(),
    "0x9008d19f58aabd9ed0d60971565aa8510560ab41",
);
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
- `tracing` — structured spans through the
  [`tracing`](https://docs.rs/tracing) facade.

## Primitive layer

`Address`, `Hash32`, `OrderUid`, and `AppDataHash` come from `cow-sdk-core` as
`#[repr(transparent)]` newtypes over the matching `alloy_primitives` types per
[ADR 0052](https://github.com/0xSymbiome/cow-rs/blob/main/docs/adr/0052-alloy-primitives-canonical-primitive-layer.md);
conversion at the alloy seam is zero-cost. EIP-712 domain separators and message
hashes route through `alloy_sol_types`.

## Where this fits

This crate is the deterministic, provider-agnostic building-block layer. It does
not own the user-domain `OrderData` (that is
[`cow-sdk-core`](https://crates.io/crates/cow-sdk-core); this crate hashes over
it), it carries no RPC client (verification takes a `cow_sdk_core::Provider`
parameter), it does not implement the solver `settle` path (out of scope —
order-lifecycle only), and it does no HTTP or order submission
([`cow-sdk-orderbook`](https://crates.io/crates/cow-sdk-orderbook)) or order
building ([`cow-sdk-trading`](https://crates.io/crates/cow-sdk-trading)).

## Where to next

- [Getting Started](https://github.com/0xSymbiome/cow-rs/blob/main/docs/getting-started.md)
- [Workspace README](https://github.com/0xSymbiome/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-or-later. See the workspace
[LICENSE](https://github.com/0xSymbiome/cow-rs/blob/main/LICENSE)
file for the full text.
