# ADR 0056: Settlement Event Decoding Is Fail-Closed And Provider-Free

- Status: Accepted
- Date: 2026-05-29
- Last reviewed: 2026-05-29
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: contracts, bindings, events, decoding, defense-in-depth
- Related: [ADR 0054](0054-onchain-order-event-decoding-is-fail-closed.md), [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0033](0033-minimum-viable-panic-surface.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

The `GPv2Settlement` event decoder in `cow-sdk-contracts` decodes `Trade`,
`Interaction`, `Settlement`, `OrderInvalidated`, and the inherited `GPv2Signing`
`PreSignature` logs through `alloy::sol!`-generated event types and is
fail-closed. It accepts borrowed log bytes (`alloy_primitives::LogData`) with no
`Provider` or network dependency, validates the topic set against the generated
`SIGNATURE_HASH` and the indexed arity before ABI decoding, length-checks the
56-byte order UID, and maps each decoded event into the `#[non_exhaustive]`
`SettlementEvent` enum in the crate's domain vocabulary (`Address`, `Amount`,
`OrderUid`). Every malformed input returns a typed `ContractsError`; no log,
however adversarial, can panic the decoder. The topic-set guard is the shared
`check_topics` helper, the same fail-closed guard the on-chain order decoder uses.

This decision extends the posture ratified in [ADR 0054](0054-onchain-order-event-decoding-is-fail-closed.md)
for the on-chain order events to the settlement event family.

## Why

On-chain logs are untrusted input. A decoder that reads a topic array without a
count check, or slices a dynamic field without a length check, turns a malformed
or hostile log into a panic — a denial of service for any indexer, watcher, or
wallet that feeds it chain data. Borrowing the log bytes instead of taking a
`Provider` keeps the decoder transport-agnostic and wasm-safe, so one
implementation serves native, browser, and any RPC client. Returning a typed
`SettlementEvent` in the crate's domain vocabulary keeps the public surface
strongly typed and lets the 56-byte `OrderUid` invariant be enforced at the
decode boundary rather than left to the caller.

## Must Remain True

- Public surface: `decode_settlement_log` takes `&LogData` and returns
  `Result<SettlementEvent, ContractsError>`. No decoding path takes a `Provider`
  or performs I/O. `SettlementEvent` is `#[non_exhaustive]`.
- Runtime and support: the decoder depends only on `alloy::sol!` and
  `alloy-primitives` — wasm-safe, no tokio, no network.
- Validation and review: topic count, topic-0, and order-UID length are each
  covered by a fail-closed test; a fuzz target asserts the decoder never panics
  on arbitrary `LogData`; topic-0 bytes are locked against an independent
  keccak-256 of the canonical event signature, so a binding drift breaks the
  build. `SettlementEvent` is classified `upstream-growing` in `enum-policy.yaml`.
- Shared guard: the topic-set validation reuses the single `check_topics` helper
  shared with the on-chain order decoder, so the fail-closed topic contract is
  expressed in one place.

## Alternatives Rejected

- Panic on malformed input: simpler, but turns hostile chain data into a crash
  for every consumer.
- Take a `Provider` and fetch or resolve eagerly inside the decoder: couples a
  pure codec to a network client and breaks wasm-safety.
- Reuse the settlement calldata `Trade` struct as the event shape: the calldata
  trade encodes tokens as registry indices and packs flags, whereas the `Trade`
  event carries full token addresses, executed amounts, and the order UID; one
  type cannot model both wire shapes without losing clarity.

## Links

- [ADR 0054](0054-onchain-order-event-decoding-is-fail-closed.md)
- [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md)
- [Architecture](../architecture.md)
- [Parity Matrix](../parity.md)

**Proven by:**

- [Settlement Event Log Decoding Audit](../audit/settlement-event-log-decoding-audit.md)
