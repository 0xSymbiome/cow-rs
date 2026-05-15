# Composable Watch-Tower Boundary Audit

Status: Current
Last reviewed: 2026-05-15
Owning surface: composable helper crate public API and crate-graph posture
Refresh trigger: Refresh when composable helper APIs add orchestration behavior, when the negative-edge invariants change, or when ADR 0048's DOES / DOES-NOT lists are amended.
Related docs:
- [ADR 0048](../adr/0048-composable-conditional-order-framework.md)
- [Principles](../principles.md) (Off-Chain Orchestration Boundary)
- [Composable Contract Bindings Audit](composable-contract-bindings-audit.md)

## Scope

This audit covers:

- the reachable public surface of the composable helper crate under the
  facade-level `composable` feature;
- the negative-edge invariants `cow-sdk-composable ⇏ cow-sdk-trading` and
  `cow-sdk-composable ⇏ alloy-provider`;
- the absence of forbidden patterns enumerated in ADR 0048's DOES NOT list
  (service loops, persistence adapters, notification systems, automatic
  order posting, global retry cadence, chain event indexing beyond
  `event_scan_async`, production watch-tower state machines, and any
  `tokio::spawn` or `wasm_bindgen_futures::spawn_local` site).

It does not cover the typed encoders, decoders, or selector parity surfaces;
those are governed by the [Composable Contract Bindings Audit](composable-contract-bindings-audit.md).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Reachable surface | Every item in ADR 0048's DOES list is reachable through the facade-level `composable` feature, and every item in the DOES NOT list is absent at every feature combination | Conforms |
| Crate-graph posture | `cow-sdk-composable ⇏ cow-sdk-trading` and `cow-sdk-composable ⇏ alloy-provider` (default features) hold under `cargo metadata` | Conforms |
| Reverse-edge guard | `cow-sdk-orderbook ⇏ cow-sdk-composable` holds, preventing the orderbook crate from depending on the composable leaf | Conforms |
| No-spawn invariant | Grep for `tokio::spawn`, `tokio::time::interval`, `tokio::time::sleep`, `wasm_bindgen_futures::spawn_local`, `start()`, and `run_forever()` returns zero hits inside the composable crate body | Conforms |

## Current Contract

### DOES surface

The composable helper crate exposes typed encoders, decoders, custom-error
selector constants, the `#[non_exhaustive]` `PollResult` classification
enum, the single-call `ComposableCowApi::poll_async` over an injected
`AsyncProvider`, `event_scan_async` as a single-call provider operation
over a caller-bounded block range, the local `local_poll_async` simulator
that replays a `PollResult` from a captured input tuple without any RPC,
and a reference watcher example crate that lives outside the published
library surface.

### DOES NOT surface

The composable helper crate does not expose service loops, persistence
adapters, notification systems, automatic order posting, global retry
cadence policy, chain event indexing beyond `event_scan_async`, production
watch-tower state machines, or any `tokio::spawn` or
`wasm_bindgen_futures::spawn_local` call site. Any future change that adds
one of these items to the crate body is a regression of ADR 0048 and must
be rejected at review.

### Crate-graph invariants

The composable helper crate depends only on `cow-sdk-core`,
`cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-orderbook`, and
`cow-sdk-pure-helpers`. The negative-edge invariants
`cow-sdk-composable ⇏ cow-sdk-trading` and
`cow-sdk-composable ⇏ alloy-provider` hold under `cargo metadata`. An
optional `composable-with-cow-shed` feature lifts a non-default dependency
on `cow-sdk-cow-shed` for the narrow Gnosis-only forwarder flow.

The reverse-edge guard `cow-sdk-orderbook ⇏ cow-sdk-composable` ensures
the orderbook crate does not depend on the composable leaf, preserving the
additive-leaf ordering.

## Evidence

Primary implementation points:

- `docs/adr/0048-composable-conditional-order-framework.md`
- `docs/principles.md` (Off-Chain Orchestration Boundary)
- `crates/composable/` (reserved leaf crate)
- `scripts/parity-maintainer/src/main.rs` (negative-edge validator entry
  point)

Primary regression coverage:

- `cargo metadata --format-version 1` proves the negative-edge invariants
  hold under the default feature set
- `cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- check-deps --negative-edge cow-sdk-composable::cow-sdk-trading`

Validation surface:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- check-deps --negative-edge cow-sdk-composable::cow-sdk-trading
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- check-deps --negative-edge cow-sdk-composable::alloy-provider
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- check-deps --negative-edge cow-sdk-orderbook::cow-sdk-composable
```
