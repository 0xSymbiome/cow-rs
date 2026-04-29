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

The workspace keeps alloy's ABI, primitive, and `sol!` dependency family pinned
centrally. A dedicated weekly scheduled and manually-dispatched canary workflow
checks the same workspace against a configurable upstream `ALLOY_CANARY_REF`
and falls back to a pinned SHA when no repository variable is set. The canary
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

- Public surface: default published crates do not force an `alloy-provider`
  dependency, facade re-exports remain SDK-owned, and public RPC traits
  continue to use SDK-owned request and response types.
- Runtime and support: alloy-powered adapters live at leaf or consumer-owned
  boundaries; the facade does not choose a chain-RPC runtime for consumers.
- Validation and review: `alloy-*` workspace packages stay on one reviewed
  minor line, the `alloy-provider` invariant gate stays blocking, and the
  candidate canary stays configurable by ref with a pinned SHA fallback.
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

**Proven by:**

- [Contract Bindings Parity Audit](../audit/contract-bindings-parity-audit.md)
- [Browser-Wallet Alloy Dependency Audit](../audit/browser-wallet-alloy-dependency-audit.md)
- [Source-Lock Provenance Audit](../audit/source-lock-provenance-audit.md)
- [Workflow Security Audit](../audit/workflow-security-audit.md)
