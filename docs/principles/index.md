# Principles

These principles define the public engineering posture of `cow-rs`. Each is a standing rule
anchored to one primary ADR (its decision record) and proven — or honestly marked unenforced — by
a named gate or test. Read a principle by its **Invariant** first; the body then gives the failure
it prevents, how to comply, and where it is enforced.

## The principles

| # | Principle | Shape | Invariant | Primary ADR | Enforced by |
|---|-----------|-------|-----------|-------------|-------------|
| 1 | [Deterministic Protocol Transforms](deterministic-protocol-transforms.md) | rule | Same canonical input → same bytes; domains stay type-distinct | [0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md) | parity fixtures + proptests + fences |
| 2 | [Explicit Runtime Boundaries](explicit-runtime-boundaries.md) | structure | Pure transform crates do no hidden HTTP/RPC/GraphQL I/O | [0010](../adr/0010-runtime-neutral-async-and-transport-posture.md) | rest-transport fence (partial) |
| 3 | [Thin Facade, Real Crate Boundaries](thin-facade-real-crate-boundaries.md) | structure | `cow-sdk` re-exports; leaf crates own behavior | [0001](../adr/0001-multi-crate-sdk-family-with-thin-facade.md) | documentation-only |
| 4 | [Instance-Scoped Configuration](instance-scoped-configuration.md) | rule | Policy is per-instance; no process-global mutable state | [0006](../adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md) | documentation-only |
| 5 | [Strong Typed Public Surfaces](strong-typed-public-surfaces.md) | classify | Domain types for protocol meanings; strings only at wire boundaries | [0011](../adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md) | check-enum-policy + deny-unknown-fields |
| 6 | [Additive Optional Ecosystems](additive-optional-ecosystems.md) | structure | Optional capability via leaf crates / feature gates only | [0001](../adr/0001-multi-crate-sdk-family-with-thin-facade.md) | wasm flavour reachability + check-wasm-invariant |
| 7 | [Sole Construction Seam](sole-construction-seam.md) | structure | Clients build only via typestate `builder()`; misuse is a compile error | [0013](../adr/0013-http-transport-injection-and-typestate-builders.md) | trybuild compile-fail |
| 8 | [Chain-RPC Runtime Neutrality](chain-rpc-runtime-neutrality.md) | structure | Default path is provider-neutral via the core trait seam | [0024](../adr/0024-asyncprovider-asyncsigningprovider-capability-split.md) | alloy-provider / -signer invariants |
| 9 | [Canonical Contract Bindings](canonical-contract-bindings.md) | pipeline | Inline `sol!`, pinned and proven byte-for-byte; no hand-rolled encoders | [0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md) | parity_contract + family-pins + fences |
| 10 | [Evidence-Backed Public Claims](evidence-backed-public-claims.md) | pipeline | Claims backed by repo-visible evidence + reproducible provenance | [0026](../adr/0026-alloy-major-release-absorption-plan.md) | property-citations + docs-agree + audit-index |
| 11 | [Forward-Compatible Public Surfaces](forward-compatible-public-surfaces.md) | classify | Every public type absorbs additive growth, or is deliberately frozen | [0031](../adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md) | check-enum-policy + clippy lints |
| 12 | [Credential Redaction by Construction](credential-redaction-by-construction.md) | classify | Credentials wrapped in `Redacted`; only sanitized identity renders | [0025](../adr/0025-workspace-url-redaction-convention.md) | error_redaction_contract + check-wasm-invariant |
| 13 | [Cooperative Cancellation Coverage](cooperative-cancellation-coverage.md) | rule | Every long-running async method composes with `cancel_with` | [0010](../adr/0010-runtime-neutral-async-and-transport-posture.md) | cancellation_coverage_validator |
| 14 | [Minimum-Viable Panic Surface](minimum-viable-panic-surface.md) | classify | No unwrap/panic outside allowlisted, documented static invariants | [0033](../adr/0033-minimum-viable-panic-surface.md) | check-panic-allowlist |
| 15 | [Layered Operation Surface](layered-operation-surface.md) | structure | Free fns / bound methods / builders; thin delegation, one import path | [0069](../adr/0069-layered-trading-operation-surface-and-signing-free-transport.md) | documentation-only |
| 16 | [Off-Chain Orchestration Boundary](off-chain-orchestration-boundary.md) | structure | Composable ships off-by-default; pure encoders only, no watcher loops | [0057](../adr/0057-log-provider-capability-trait.md) | composable feature gate (partial) |

## Shapes

A principle's **shape** says what you do with it — and what visual, if any, it carries:

- **rule** — a flat invariant; no diagram.
- **structure** — a layering or boundary; carries a `**Shape**` diagram.
- **classify** — you sort your case into a posture; carries a `**Decision**` tree.
- **pipeline** — an ordered sequence to satisfy; carries a `**Pipeline**` diagram.

## Authoring contract

- Every principle is one `docs/principles/<slug>.md` file with OKF frontmatter (`type: Principle`,
  `title`, `description`, `tags`, `timestamp`, `anchored_by`, `shape`) and the sections
  **Invariant**, **Why**, **Enforced by**, **Anchored by** — plus the shape's diagram.
- The principle→ADR edge is owned by [`.github/config/principle-adr-map.yaml`](../../.github/config/principle-adr-map.yaml).
  A file's `anchored_by` frontmatter and its `**Anchored by**` line must list the same ADRs as the
  map, primary first.
- The `check-principles` gate (`cargo check-principles`, part of `cargo check-policies`) enforces
  the 1:1 file↔map correspondence, the frontmatter, the linkage agreement, the required sections,
  the shape-gated diagram, and that inbound references deep-link `<slug>.md` rather than a stale
  `index.md#anchor`.
- To add or change a principle: edit the map, author or update the `<slug>.md` to this skeleton,
  then run `cargo check-principles`.
