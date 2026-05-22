# ADR 0026: Bound Alloy Major Releases Behind SDK Types And A Configurable Canary Lane

- Status: Accepted (amended)
- Date: 2026-04-27
- Last reviewed: 2026-05-22
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: alloy, dependencies, provider, compatibility, ci
- Related: [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

Alloy major releases are absorbed at repository-controlled boundaries, not at
the published SDK facade. The facade re-export set stays sealed over SDK-owned
types: domain types, provider traits, generated contract bindings, and typed
request or response models rather than concrete alloy provider types.

The workspace keeps Alloy's runtime family and Alloy Core's ABI, primitive, and
`sol!` family pinned centrally. Runtime crates stay on the reviewed `2.0`
family; ABI/core crates stay on the reviewed `1.5` family. `alloy-provider` is
allowed only in `cow-sdk-alloy-provider` and `cow-sdk-alloy`.
`alloy-signer-local` is allowed only in `cow-sdk-alloy-signer` and
`cow-sdk-alloy`.

A scheduled and manually-dispatched canary workflow checks the workspace
against configurable upstream Alloy and Alloy Core refs. It reports
forward-compatibility drift without adding a pull-request trigger; promotion to
PR-blocking status requires an explicit policy change. The operational
rehearsal, release-day, rollback, and escalation procedures live in the
dedicated alloy major-release runbook.

## Why

Alloy powers contract bindings, typed ABI helpers, and browser-wallet ABI
support, so major releases can surface real migration work. Letting alloy types
leak into the stable SDK facade would turn those migrations into
consumer-facing semver breaks. Keeping the seam behind SDK-owned traits and
data types makes major-release absorption a local maintenance event unless
protocol semantics change.

## Must Remain True

- Public surface: default published crates do not force native Alloy provider
  or local-signer dependencies, facade re-exports remain SDK-owned, and public
  RPC traits continue to use SDK-owned request and response types.
- Runtime and support: alloy-powered adapters live at leaf or consumer-owned
  boundaries; the facade does not choose a chain-RPC runtime for consumers.
- Validation and review: runtime and ABI/core crates stay on the reviewed
  two-family policy, invariant gates stay blocking, source-lock records the
  reviewed pins, and the canary stays configurable by ref.
- Cost: canary failures require triage before dependency promotion, but do not
  block routine PR CI while informational.

## Compatibility Matrix

| SDK release | Alloy runtime family | Alloy Core ABI family | Resolution invariant |
| --- | --- | --- | --- |
| `0.1.0` | `2.0.4` | `1.5.7` | `Cargo.lock` resolves each listed crate to exactly one version; the workspace invariant test enforces this. |

Runtime family: `alloy-consensus`, `alloy-json-rpc`, `alloy-network`,
`alloy-provider`, `alloy-rpc-types-eth`, `alloy-signer`,
`alloy-signer-local`, `alloy-transport`, `alloy-transport-http`.

Alloy Core ABI family: `alloy-dyn-abi`, `alloy-json-abi`,
`alloy-primitives`, `alloy-sol-macro`, `alloy-sol-types`.

Future rows record exact resolved versions, not caret ranges.

## Alternatives Rejected

- Expose alloy provider types directly: this would bind the SDK semver surface
  to an external provider ecosystem.
- Freeze alloy indefinitely: this avoids migration pressure briefly but raises
  security, compatibility, and upstream-support risk.
- Hardcode a moving upstream branch in CI: mutable names are not stable release
  evidence.
- Make the canary PR-blocking immediately: major-release drift should be
  observable before it becomes a required contributor gate.

## Links

- [Alloy Major-Release Absorption Runbook](../alloy-major-release-runbook.md)
- [Architecture](../architecture.md)
- [Adapting alloy providers](../providers/adapting-alloy.md)
- [Parity scope source lock](../parity-scope.md#source-lock)
- [Verification matrix workspace gates](../verification-matrix.md#workspace-gates)
- [Alloy release-candidate workflow](../../.github/workflows/alloy-release-candidate.yml)
- [Alloy Umbrella Adapter ADR](0037-alloy-umbrella-adapter.md)

**Proven by:**

- [Contract Bindings Parity Audit](../audit/contract-bindings-parity-audit.md)
- [Browser-Wallet Alloy Dependency Audit](../audit/browser-wallet-alloy-dependency-audit.md)
- [Alloy Umbrella Adapter Audit](../audit/alloy-umbrella-adapter-audit.md)
- [Source-Lock Provenance Audit](../audit/source-lock-provenance-audit.md)
- [Workflow Security Audit](../audit/workflow-security-audit.md)

## Amendment 2026-05-22: canonical primitive layer (per ADR 0052)

The alloy-core ABI family (`alloy-primitives`, `alloy-sol-types`,
`alloy-sol-macro`, `alloy-dyn-abi`, `alloy-json-abi`, `alloy-serde`) is
in scope for direct dependency on `cow-sdk-core`, `cow-sdk-contracts`,
`cow-sdk-signing`, `cow-sdk-app-data`, and `cow-sdk-cow-shed` (plus
`cow-sdk-composable` when that crate is rooted) per
[ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md). The
alloy-runtime family (`alloy-provider`, `alloy-signer-local`,
`alloy-network`, `alloy-consensus`, `alloy-rpc-types-eth`, and the
`alloy-transport-*` family) remains confined to the native adapter
crates `cow-sdk-alloy`, `cow-sdk-alloy-provider`, and
`cow-sdk-alloy-signer`. The cow-named identity and numeric types
re-exported from `cow-sdk-core` are cow-owned `#[repr(transparent)]`
newtypes over the alloy-core primitive types per ADR 0052; the facade
does not leak raw alloy paths into public docs or wasm-bindgen exports.
The canary lane continues to cover the expanded alloy-core surface.
