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

## Compatibility Matrix

| SDK release | Alloy runtime family (9 crates, all at) | Alloy Core ABI family (5 crates, all at) | Resolution invariant |
| --- | --- | --- | --- |
| `0.1.0` | `2.0.4` | `1.5.7` | `Cargo.lock` resolves each listed crate to exactly one version; the workspace test `alloy_two_family_lockfile_invariant` enforces this. |

Runtime family: `alloy-consensus`, `alloy-json-rpc`, `alloy-network`,
`alloy-provider`, `alloy-rpc-types-eth`, `alloy-signer`,
`alloy-signer-local`, `alloy-transport`, `alloy-transport-http`.

Alloy Core ABI family: `alloy-dyn-abi`, `alloy-json-abi`,
`alloy-primitives`, `alloy-sol-macro`, `alloy-sol-types`.

Future rows record exact resolved versions, not caret ranges. The crate sets
stay aligned with the workspace dependency declarations and the manifest and
lockfile invariant tests.

## Upgrade Rehearsal

When a new Alloy runtime or Alloy Core minor lands, the maintainer runs this
rehearsal on a release-candidate branch before the absorbed version reaches the
main branch.

1. Trigger the `alloy-release-candidate.yml` canary manually and confirm it
   detects the candidate refs.
2. Update the runtime family to the new pinned version:

   ```sh
   cargo update -p alloy-consensus --precise <new-runtime-version>
   cargo update -p alloy-json-rpc --precise <new-runtime-version>
   cargo update -p alloy-network --precise <new-runtime-version>
   cargo update -p alloy-provider --precise <new-runtime-version>
   cargo update -p alloy-rpc-types-eth --precise <new-runtime-version>
   cargo update -p alloy-signer --precise <new-runtime-version>
   cargo update -p alloy-signer-local --precise <new-runtime-version>
   cargo update -p alloy-transport --precise <new-runtime-version>
   cargo update -p alloy-transport-http --precise <new-runtime-version>
   ```

3. Update the Alloy Core ABI family to the new pinned version:

   ```sh
   cargo update -p alloy-dyn-abi --precise <new-core-version>
   cargo update -p alloy-json-abi --precise <new-core-version>
   cargo update -p alloy-primitives --precise <new-core-version>
   cargo update -p alloy-sol-macro --precise <new-core-version>
   cargo update -p alloy-sol-types --precise <new-core-version>
   ```

4. Update the workspace dependency version literal for each crate above so the
   manifest declarations match the resolved set.
5. Run the validation sweep:

   ```sh
   cargo fmt --all --check
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo test --workspace --all-features
   cargo test -p cow-rs-workspace-tests --test alloy_two_family_lockfile_invariant
   cargo check-alloy-provider-invariant
   cargo check-alloy-signer-invariant
   ```

6. Refresh the dependency-audit baseline as Markdown. Capture raw
   `cargo audit --json` output to a temporary evidence file, then re-author
   `parity/dependency-audit/alloy-runtime-baseline.md` with the summarized
   advisory delta. Do not redirect raw JSON into the Markdown baseline.

   ```sh
   cargo audit --json > parity/dependency-audit/.tmp/alloy-audit-<date>.json
   ```

7. Append a new Compatibility Matrix row and refresh standing audits whose
   summaries cite the prior pinned version.

## Rollback Path

If the rehearsal surfaces an incompatibility:

1. Revert the runtime family to the previously absorbed version:

   ```sh
   cargo update -p alloy-consensus --precise <previous-runtime-version>
   cargo update -p alloy-json-rpc --precise <previous-runtime-version>
   cargo update -p alloy-network --precise <previous-runtime-version>
   cargo update -p alloy-provider --precise <previous-runtime-version>
   cargo update -p alloy-rpc-types-eth --precise <previous-runtime-version>
   cargo update -p alloy-signer --precise <previous-runtime-version>
   cargo update -p alloy-signer-local --precise <previous-runtime-version>
   cargo update -p alloy-transport --precise <previous-runtime-version>
   cargo update -p alloy-transport-http --precise <previous-runtime-version>
   ```

2. Revert the Alloy Core ABI family to the previously absorbed version:

   ```sh
   cargo update -p alloy-dyn-abi --precise <previous-core-version>
   cargo update -p alloy-json-abi --precise <previous-core-version>
   cargo update -p alloy-primitives --precise <previous-core-version>
   cargo update -p alloy-sol-macro --precise <previous-core-version>
   cargo update -p alloy-sol-types --precise <previous-core-version>
   ```

3. Restore the workspace dependency version literals for every crate above to
   match the reverted resolution.
4. Document the incompatibility under a "Blocked Releases" subsection with the
   absorbed version, failure mode, and upstream issue link.
5. Keep the current Compatibility Matrix row unchanged until the upstream fix
   lands or the in-place absorption is feasible.

## Continuous Absorption Check

The scheduled canary is report-only. If it fails, the workflow opens or
reuses an `alloy-canary` tracking issue so maintainers have a durable triage
record without making routine pull-request CI depend on upstream candidate
state.

| Signal | Notification | Permission |
| --- | --- | --- |
| Alloy release-candidate failure | Open or reuse an `alloy-canary` issue through the workflow `gh api` shell step | `issues: write` |

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
