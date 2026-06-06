# Composable Watch-Tower Boundary Audit

Status: Current
Last reviewed: 2026-06-06
Owning surface: the reserved `cow-sdk-composable` crate and the deferred watch-tower boundary governed by ADR 0048
Refresh trigger: Refresh when the composable crate gains a crate body and joins the workspace `members`, when ADR 0048's DOES / DOES NOT lists are amended, or when the negative-edge crate-graph invariants begin mechanical enforcement.
Related docs:
- [ADR 0048](../adr/0048-composable-conditional-order-framework.md)
- [Principles](../principles.md) (Off-Chain Orchestration Boundary)
- [Composable Contract Bindings Audit](composable-contract-bindings-audit.md)

## Scope

This audit covers:

- the current state of the reserved `cow-sdk-composable` crate, namely a
  manifest-only placeholder that is not a workspace member, carries no crate
  body, and exposes no public API;
- the watch-tower boundary that ADR 0048 prescribes for that crate's future
  body, recorded here as deferred.

It does not cover the composable-cow Solidity mirrors, deployment registry rows,
or parity fixtures, which exist today under the contracts crate and parity tree
and are governed by the
[Composable Contract Bindings Audit](composable-contract-bindings-audit.md).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Crate state | `cow-sdk-composable` is a reserved, manifest-only placeholder: an empty `[dependencies]` set, no `src/` body, and no entry in the workspace `members` list | Conforms |
| Published surface | The crate exposes no public API and is not part of the published `cow-sdk` facade surface, so depending on it has no effect today | Conforms |
| Watch-tower boundary | The DOES / DOES NOT boundary that will bound the future crate body is recorded in ADR 0048 and is deferred until the crate lands | Deferred |
| Crate-graph invariants | The negative-edge invariants (`cow-sdk-composable ⇏ cow-sdk-trading`, `cow-sdk-composable ⇏ alloy-provider`) hold vacuously while the crate carries no dependencies; mechanical enforcement begins when the crate joins the workspace | Deferred |

## Current Contract

### Crate state

`cow-sdk-composable` is a reserved manifest. Its `Cargo.toml` declares package
identity and an empty `[dependencies]` set, it has no `src/` body, and it is not
listed in the workspace `members`. It therefore ships no code and exposes no
public API. The crate README states the same posture for consumers.

### Deferred watch-tower boundary

The boundary that will bound the crate body when it lands is recorded in
ADR 0048. When the crate gains a body, it is bounded to deterministic encoders,
decoders, selector constants, the `PollResult` classification enum, and
single-call provider operations (`poll_async`, `event_scan_async`,
`local_poll_async`), and it never embeds service loops, persistence adapters,
notification systems, automatic order posting, global retry cadence, chain event
indexing beyond a single-call scan, production watch-tower state machines, or any
background task. That boundary is governed by ADR 0048; this audit becomes a
conformance record when the crate lands.

### Crate-graph invariants

While the reserved crate carries no dependencies, the negative-edge invariants
`cow-sdk-composable ⇏ cow-sdk-trading` and `cow-sdk-composable ⇏ alloy-provider`
hold vacuously, and the reverse-edge guard
`cow-sdk-orderbook ⇏ cow-sdk-composable` holds because no crate depends on the
reserved leaf. Mechanical enforcement through `cargo metadata` and the dependency
validator begins when the crate joins the workspace with a real dependency set.

## Evidence

Primary implementation points:

- `crates/composable/Cargo.toml` (reserved manifest, empty dependency set)
- `crates/composable/README.md` (placeholder posture for consumers)
- `docs/adr/0048-composable-conditional-order-framework.md` (the governing boundary and its deferral)

Validation surface:

```text
cargo metadata --format-version 1
```
