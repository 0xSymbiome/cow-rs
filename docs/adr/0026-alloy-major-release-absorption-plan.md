# ADR 0026: Bound Alloy Major Releases Behind SDK Types And A Configurable Canary Lane

- Status: Accepted
- Date: 2026-04-27
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: alloy, dependencies, provider, compatibility, ci
- Related: [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md)

## Decision

Alloy major releases are absorbed at repository-controlled boundaries, not at
the published SDK facade. The facade re-export set stays sealed over SDK-owned
types: it exposes `cow-sdk-core` domain types, provider traits, generated
contract bindings, and typed request or response models rather than concrete
alloy provider types.

The workspace keeps Alloy's runtime family and Alloy Core's ABI, primitive, and
`sol!` family pinned centrally. Runtime crates stay on the reviewed `2.0`
family; ABI/core crates stay on the reviewed `1.5` family. `alloy-provider` is
allowed only in `cow-sdk-alloy-provider` and `cow-sdk-alloy`.
`alloy-signer-local` is allowed only in `cow-sdk-alloy-signer` and
`cow-sdk-alloy`.

A dedicated weekly scheduled and manually-dispatched canary workflow checks the
same workspace against configurable upstream Alloy and Alloy Core refs and
falls back to pinned SHAs when no repository variables are set. The canary
reports forward-compatibility drift without adding a pull-request trigger;
promotion to PR-blocking status requires an explicit policy change.

## Why

Alloy powers contract bindings, typed ABI helpers, and browser-wallet ABI
support, so major releases can surface real migration work. Letting alloy types
leak into the stable SDK facade would turn those migrations into consumer-facing
semver breaks. Keeping the seam behind SDK-owned traits and data types makes
major-release absorption a local maintenance event unless protocol semantics
change.

## Must Remain True

- Public surface: default published crates do not force native Alloy provider
  or local-signer dependencies, facade re-exports remain SDK-owned, and public
  RPC traits continue to use SDK-owned request and response types.
- Runtime and support: alloy-powered adapters live at leaf or consumer-owned
  boundaries; the facade does not choose a chain-RPC runtime for consumers.
- Validation and review: Alloy runtime crates and Alloy ABI/core crates stay on
  their reviewed two-family policy, the Alloy provider and signer-local
  invariant gates stay blocking, source-lock records the reviewed upstream pins
  for `alloy-rs/alloy` v2.0.4 and `alloy-rs/core` v1.5.7, and the candidate
  canary stays configurable by ref with pinned SHA fallbacks.
- Cost: a canary failure requires triage before the next dependency upgrade,
  but it does not block routine PR CI while it remains informational.

## Alternatives Rejected

- Expose alloy provider types directly: this would bind the SDK semver surface
  to an external provider ecosystem.
- Freeze alloy indefinitely: this avoids migration pressure briefly but raises
  security, compatibility, and upstream-support risk.
- Hardcode a moving upstream branch in CI: mutable names are not stable release
  evidence and make reproducing drift harder.
- Make the canary PR-blocking immediately: major-release drift should be
  observable before it becomes a required contributor gate.

## Links

- [Architecture](../architecture.md)
- [Adapting alloy providers](../providers/adapting-alloy.md)
- [Parity scope source lock](../parity-scope.md#source-lock)
- [Parity scope surface boundaries](../parity-scope.md#surface-boundaries)
- [Verification matrix workspace gates](../verification-matrix.md#workspace-gates)
- [Alloy release-candidate workflow](../../.github/workflows/alloy-release-candidate.yml)
- [Alloy Umbrella Adapter ADR](0037-alloy-umbrella-adapter.md)

**Proven by:**

- [Contract Bindings Parity Audit](../audit/contract-bindings-parity-audit.md)
- [Browser-Wallet Alloy Dependency Audit](../audit/browser-wallet-alloy-dependency-audit.md)
- [Alloy Umbrella Adapter Audit](../audit/alloy-umbrella-adapter-audit.md)
- [Source-Lock Provenance Audit](../audit/source-lock-provenance-audit.md)
- [Workflow Security Audit](../audit/workflow-security-audit.md)
