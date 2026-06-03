# ADR 0009: WASM Verification Consoles — Hybrid Extensibility And Two-Tier Proof

- Status: Superseded
- Date: 2026-04-17
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: examples, wasm, browser-wallet, proof-posture, extensibility
- Related: [ADR 0004](0004-feature-gated-browser-wallet-sidecar.md), [ADR 0007](0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md), [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md)
- Superseded by: [ADR 0065](0065-canonical-browser-wallet-example.md)

> Superseded by [ADR 0065](0065-canonical-browser-wallet-example.md): the
> verification-console genre is retired in favour of a single canonical
> browser-wallet trade example. The decision below is retained as design
> history.

## Decision

WASM artifacts shipped under `examples/wasm/` are named verification
consoles. They follow one naming shape, one ship checklist, one two-tier
proof posture, and one hybrid extensibility rule.

## Why

Browser-runtime examples are the first and often only place reviewers click
to confirm that the SDK behaves as claimed. If those surfaces drift into
pedagogical playgrounds, marketing pages, or ad-hoc dashboards, the public
proof story weakens. If the two surfaces that already ship diverge in shape,
reviewers must learn each one separately. If new capabilities land as panels
inside an existing console when they should have been their own console, or
as new consoles when they should have been panels, the review surface
collapses into kitchen drawers. The verification-console genre, a fixed
shape, and an explicit workflow-criterion gate keep the surface honest as it
grows.

## Must Remain True

- Public surface: WASM artifacts under `examples/wasm/` are verification
  consoles, not playgrounds. The naming convention is folder
  `<capability>-console/`, Cargo package `cow-sdk-<capability>-console`,
  Playwright lane `e2e/<capability>/`, and hosted path
  `<capability>-console/`. When the literal substitution would repeat, the
  inner `sdk-` drops at the package level only.
- Ship checklist: every console carries a one-sentence user-outcome
  subheading, a primary walkthrough entry that drives a deterministic cycle
  end-to-end, a persistent mode/chain/wallet indicator, a visible
  hosted-build link when the page is not on the Pages host, and a README on
  the fixed template shape.
- Proof posture: the deterministic lane (host-side cargo tests,
  `wasm-bindgen-test`, Playwright with mocked EIP-1193 and mocked CoW-API
  fixtures) holds every console on every commit. The environment-sensitive
  lane (manual QA against real wallet extensions, optional static
  browser-live smoke) stays explicitly gated and never substitutes for
  deterministic proof.
- Extensibility: a capability that introduces a new user workflow lands as a
  new console crate; a capability that is a deterministic SDK addition
  extends the sdk-verification console as new panels. When in doubt, default
  to a panel.
- Cost: new consoles require more deliberate design, explicit public docs, a
  dedicated Playwright lane, and a matching `wasm-bindgen-test` lane.

## Alternatives Rejected

- Collapse the mock and injected panes into a single blended view: easier to
  read at first, but it erases the two-tier proof boundary reviewers rely on
  to diagnose failures.
- Let future capabilities accrete as panels inside the first-shipped console:
  shorter in the short term, but it collapses distinct workflows into one
  surface and undermines the verification-console genre.
- Rename the genre to "playground" or "demo": matches the language other
  ecosystems use informally, but it would invite marketing copy and user-code
  editing surfaces that are out of scope for the reviewed contract.

## Links

- [Architecture](../architecture.md)
- [Examples](../examples.md)
- [Browser-Runtime Proof Posture](../browser-runtime-proof-posture.md)
- [ADR 0004](0004-feature-gated-browser-wallet-sidecar.md)
- [ADR 0007](0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md)
- [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md)
- [ADR 0065](0065-canonical-browser-wallet-example.md) (supersedes this ADR)
