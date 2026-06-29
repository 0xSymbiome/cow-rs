---
type: Runbook
title: "Release Checklist"
description: "Use this checklist before tagging or publishing a release that changes the public cow-rs surface."
timestamp: 2026-06-28T00:00:00Z
---

# Release Checklist

Use this checklist before tagging or publishing a release that changes the
public `cow-rs` surface.

## 1. Native Quality Gates

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets --all-features --message-format short -- -W clippy::pedantic -W clippy::cargo -A clippy::multiple_crate_versions
cargo test --workspace
cargo nextest run --workspace --all-features --config-file .github/config/nextest.toml
cargo run-deterministic-examples --locked
typos --config .github/config/typos.toml
cargo deny check --config .github/config/deny.toml
cargo audit --deny unsound --deny unmaintained --ignore RUSTSEC-2024-0388 --ignore RUSTSEC-2024-0436 --ignore RUSTSEC-2026-0173
cargo check-alloy-provider-invariant
cargo check-alloy-signer-invariant
cargo test -p cow-rs-workspace-tests --test alloy_two_family_lockfile_invariant
cargo test -p cow-sdk-alloy --test send_transaction_does_not_wait_for_confirmation
cargo test -p cow-rs-workspace-tests --test transaction_lifecycle_cross_adapter_invariant
cargo tree --invert alloy-provider -p cow-sdk-core -p cow-sdk-contracts -p cow-sdk-signing -p cow-sdk-orderbook -p cow-sdk-subgraph -p cow-sdk-app-data -p cow-sdk-trading -p cow-sdk-alloy-provider -p cow-sdk-alloy-signer -p cow-sdk-alloy -p cow-sdk -p cow-sdk-js -p cow-sdk-test
```

The native Alloy dependency gates enforce explicit allow-lists:
`alloy-provider` is allowed only in `cow-sdk-alloy-provider` and
`cow-sdk-alloy`, while `alloy-signer-local` is allowed only in
`cow-sdk-alloy-signer` and `cow-sdk-alloy`. CI normalises the raw Cargo tree
output via `cargo check-alloy-provider-invariant` and
`cargo check-alloy-signer-invariant`; contributors should use the wrappers
rather than reading raw Cargo tree output directly.

This command is guarded for drift by `cargo docs-agree`;
any mismatch against `docs/verification.md`, `CONTRIBUTING.md`, or
`PROPERTIES.md` fails the "Verify release-gate commands agree across docs
and CI" step in the `_quality-gate.yml` "Repository policies" job.

- The `_quality-gate.yml` lane enforces both the `alloy-*` workspace-pin
  same-minor invariant and the inner-workspace WASM pin diff against the
  root workspace pins.
- The `_quality-gate.yml` nextest lane runs the standard workspace test
  runner on Ubuntu, macOS, and Windows with `fail-fast: false`, so routine
  host coverage is centralized in the shared quality gate.
- The `cow-sdk-js` import fences in `cargo check-source-fences` reject forbidden
  `cow-sdk-js` source imports for native-only Alloy crates, `reqwest`, Tokio
  runtime entrypoints, Tokio macros, and the `cow-sdk-core` reqwest re-exports.
- The compiler enforces the IpfsFetch await contract: the `fetch_doc_from_*`
  futures are `#[must_use]`, so an un-awaited call fails `unused_must_use = deny`,
  and an `IpfsFetchTransport::get` that does not return the trait's future does
  not compile.
- The lockfile invariant enforces single-version resolution for the reviewed
  Alloy runtime crates at `2.0.4` and Alloy Core ABI crates at `1.5.7`. Alloy
  runtime and ABI crates ship on independent release cadences, so both family
  sets are checked explicitly before release.
- The transaction lifecycle checks enforce that signer-backed submission
  returns a broadcast acknowledgement without receipt polling and that adapter
  receipt lookups populate the modeled mined fields consistently across the
  native Alloy paths.

`cargo audit` is the blocking RustSec gate for published advisories. It keeps
vulnerabilities, unsound advisories, and unmaintained advisories blocking while
deriving its reviewed ignore arguments from `.github/config/deny.toml`.
`cargo deny` also runs with yanked advisory policy set to deny, so yanked
published-upstream cases must stay explicit in the public audit evidence until
a published replacement exists.

This command is guarded for drift by `cargo docs-agree`;
any mismatch against `docs/verification.md` or the advisory tolerance
register in `.github/config/deny.toml` fails the "Verify release-gate
commands agree across docs and CI" step in the `_quality-gate.yml`
"Repository policies" job.

## 2. Documentation And Public API Gates

```text
cargo test --workspace --doc
cargo test --doc -p cow-sdk-orderbook
cargo test --doc -p cow-sdk-trading
cargo test --doc -p cow-sdk-contracts
cargo test --all-features --workspace --doc
cargo doc --workspace --all-features --no-deps
RUSTFLAGS="-Dmissing-docs -Dmissing-debug-implementations -Dunreachable-pub -Dunnameable-types" cargo check --workspace --all-features
```

Nightly docs.rs-style lane:

```text
DOCS_RS=1 RUSTDOCFLAGS="--cfg docsrs -D warnings -Zunstable-options --generate-link-to-definition --show-type-layout --enable-index-page" cargo +nightly doc --workspace --all-features --no-deps
```

If the release diff materially touches a surface covered by `docs/audit/`,
confirm that the affected audit is still `Current`. If the reviewed surface
changed, refresh or supersede the audit in the same change set before tagging.

## 3. Compatibility And Host Coverage

```text
cargo +1.94.0 check --workspace --all-features
cargo +1.94.0 test --workspace
cargo hack check --workspace --feature-powerset --depth 1
```

`cargo hack check --workspace --feature-powerset --depth 1` is a
build-correctness check for the feature lattice. Public API stability across
feature combinations is guarded separately by the facade public API snapshots.

Expected workflow coverage:

- `ci.yml` for routine native validation and the compatibility floor
- shared quality-gate nextest matrix for Ubuntu, macOS, and Windows host
  coverage
- `crate-checks.yml` for maintenance-depth crate isolation
- `codeql.yml` for semantic security analysis

## 4. Depth Reporting

Coverage and mutation reports are produced on demand for follow-up work and
do not define threshold-based branch protection. Run them locally with the
commands below; mutation runs can be scoped narrowly to the surfaces under
review.

Coverage:

```text
cargo +nightly llvm-cov --workspace --all-features --doctests --json --summary-only --output-path target/coverage-summary.json --ignore-filename-regex "(^|/)(tests|examples|e2e)(/|$)|crates/subgraph/src/query_documents/"
cargo +nightly llvm-cov --workspace --all-features --doctests --lcov --output-path target/coverage-lcov.info --ignore-filename-regex "(^|/)(tests|examples|e2e)(/|$)|crates/subgraph/src/query_documents/"
```

Mutation:

```text
cargo mutants -p cow-sdk-contracts -p cow-sdk-signing -p cow-sdk-app-data --output target/mutants-report
cargo mutants -p cow-sdk-orderbook -p cow-sdk-trading --file crates/orderbook/src/request.rs --file crates/orderbook/src/transform.rs --file crates/trading/src/order.rs --file crates/trading/src/slippage.rs --annotations none --no-times --re "decoded_body|execute_with|sanitize_protocol_fee_bps|partner_fee_bps|calculate_unique_order_id|adjust_buy_amount" --output target/mutants-report-orderbook-trading
cargo mutants -p cow-sdk-subgraph --file crates/subgraph/src/api.rs --file crates/subgraph/src/types.rs --annotations none --no-times --re "query|with_config_override|base_url_for|deserialize_string_or_number|deserialize_optional_string_or_number|deserialize_u64_from_string_or_number|value_to_string" --output target/mutants-report-subgraph
```

Nightly retry soak:

```text
cargo test -p cow-sdk-orderbook --test request_contract retry_timeout_soak_exercises_deterministic_waveforms -- --ignored --exact
```

## 5. Repo-Local Parity And Publication Proof

`cow-sdk` `0.1.0-alpha.8` is published on crates.io and
`@symbiome-forge/cow-sdk-wasm` `0.1.0-alpha.8` on npm. The earlier
`0.0.1-reserved.0` entries were name-reservation publishes; they do not
satisfy the functional release contract described in this checklist.

Validate the committed parity contract from the current checkout:

```text
cargo parity-validate --source-lock parity/source-lock.yaml
```

Before creating the release tag, re-affirm the source-lock pins:

- run `cargo parity-validate --source-lock parity/source-lock.yaml` and
  confirm the command reports no fixture or provenance diffs
- confirm the tag commit's `parity/source-lock.yaml` matches the upstream
  pins recorded in
  [Source-Lock Provenance Audit](../audit/source-lock-provenance-audit.md), or
  confirm any pin movement landed through a reviewed bump with rationale
- confirm the
  [per-chain provenance](../audit/deployment-registry-audit.md#per-chain-provenance)
  section is still within its 90-day review window

Then run the published package-family dry-run in release order:

```text
cargo fetch --locked
cargo build --frozen --workspace --all-features
cargo package -p cow-sdk-core --allow-dirty
cargo package -p cow-sdk-contracts --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'" --config "patch.crates-io.cow-sdk-subgraph.path='crates/subgraph'"
cargo package -p cow-sdk-app-data --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo package -p cow-sdk-orderbook --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo package -p cow-sdk-signing --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'"
cargo package -p cow-sdk-subgraph --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo package -p cow-sdk-trading --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-signing.path='crates/signing'" --config "patch.crates-io.cow-sdk-app-data.path='crates/app-data'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'"
cargo package -p cow-sdk-alloy-provider --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo package -p cow-sdk-alloy-signer --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'"
cargo package -p cow-sdk-alloy --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-alloy-provider.path='crates/alloy-provider'" --config "patch.crates-io.cow-sdk-alloy-signer.path='crates/alloy-signer'"
cargo package -p cow-sdk-test --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-signing.path='crates/signing'" --config "patch.crates-io.cow-sdk-app-data.path='crates/app-data'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'" --config "patch.crates-io.cow-sdk-trading.path='crates/trading'"
cargo package -p cow-sdk --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-signing.path='crates/signing'" --config "patch.crates-io.cow-sdk-app-data.path='crates/app-data'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'" --config "patch.crates-io.cow-sdk-trading.path='crates/trading'" --config "patch.crates-io.cow-sdk-alloy-provider.path='crates/alloy-provider'" --config "patch.crates-io.cow-sdk-alloy-signer.path='crates/alloy-signer'" --config "patch.crates-io.cow-sdk-alloy.path='crates/alloy'"
```

Then run the registry-validation dry-run in the same order:

```text
cargo publish --dry-run -p cow-sdk-core --allow-dirty
cargo publish --dry-run -p cow-sdk-contracts --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'" --config "patch.crates-io.cow-sdk-subgraph.path='crates/subgraph'"
cargo publish --dry-run -p cow-sdk-app-data --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo publish --dry-run -p cow-sdk-orderbook --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo publish --dry-run -p cow-sdk-signing --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'"
cargo publish --dry-run -p cow-sdk-subgraph --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo publish --dry-run -p cow-sdk-trading --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-signing.path='crates/signing'" --config "patch.crates-io.cow-sdk-app-data.path='crates/app-data'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'"
cargo publish --dry-run -p cow-sdk-alloy-provider --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo publish --dry-run -p cow-sdk-alloy-signer --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'"
cargo publish --dry-run -p cow-sdk-alloy --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-alloy-provider.path='crates/alloy-provider'" --config "patch.crates-io.cow-sdk-alloy-signer.path='crates/alloy-signer'"
cargo publish --dry-run -p cow-sdk-test --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-signing.path='crates/signing'" --config "patch.crates-io.cow-sdk-app-data.path='crates/app-data'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'" --config "patch.crates-io.cow-sdk-trading.path='crates/trading'"
cargo publish --dry-run -p cow-sdk --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-signing.path='crates/signing'" --config "patch.crates-io.cow-sdk-app-data.path='crates/app-data'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'" --config "patch.crates-io.cow-sdk-trading.path='crates/trading'" --config "patch.crates-io.cow-sdk-alloy-provider.path='crates/alloy-provider'" --config "patch.crates-io.cow-sdk-alloy-signer.path='crates/alloy-signer'" --config "patch.crates-io.cow-sdk-alloy.path='crates/alloy'"
```

## 6. Manual Publish Sequence

Each release publishes the crates of the `cow-sdk` family in dependency
order so every step depends only on a version already indexed by the
registry. The first release (`0.1.0-alpha.1`) has shipped; use this
sequence for every subsequent release.

### Prerequisites

- Sections 1 through 5 above are green on the release branch.
- The dry-run matrix in section 5 completes cleanly for every crate
  against the release commit.
- Workspace and per-crate versions agree with the release tag.
- The release engineer is authenticated against crates.io with publish
  rights for every crate listed below.

### Publish order

Publish one crate at a time in the order below:

```text
cargo publish -p cow-sdk-core
cargo publish -p cow-sdk-contracts
cargo publish -p cow-sdk-app-data
cargo publish -p cow-sdk-orderbook
cargo publish -p cow-sdk-signing
cargo publish -p cow-sdk-subgraph
cargo publish -p cow-sdk-trading
cargo publish -p cow-sdk-alloy-provider
cargo publish -p cow-sdk-alloy-signer
cargo publish -p cow-sdk-alloy
cargo publish -p cow-sdk-test
cargo publish -p cow-sdk
```

### Index propagation backoff

- After each `cargo publish` returns successfully, wait 30 to 60 seconds
  before the next step so the freshly-published version appears in the
  crates.io index that the next dependency resolution reads.
- If the next publish fails with a missing-dependency or index-cache
  resolution error, pause another 30 to 60 seconds and retry. Repeat up
  to three times before escalating.
- For the `cow-sdk-trading` and `cow-sdk` steps, which pull multiple
  freshly-published first-party dependencies,
  the safe fallback is a two-minute wait between that step and the
  previous step.
- If crates.io returns an HTTP 429 or a documented publication
  rate-limit error, stop the sequence, wait for the rate-limit window
  to elapse, and resume from the next unpublished crate. A successful
  `cargo publish` is idempotent on the version number and cannot be
  re-executed for the same version.

### Ownership

`cargo owner --add <username> <crate>` and
`cargo owner --remove <username> <crate>` manage the per-crate owner
set on crates.io. Any ownership change applies per crate; record the
executed commands in `CHANGELOG.md` under the release heading or in
the release announcement so the owner list stays auditable.

### Yank rollback

If a published version must not be resolved by new builds, yank each
affected crate at that version:

```text
cargo yank --version <version> cow-sdk-<crate>
```

`cargo yank` marks the version unsafe to select by default for new
dependency resolution. It does not remove the crate from crates.io;
projects that have already locked the yanked version continue to
build.

Rules:

- Yank every crate in the release that carries the broken change.
- Post a release-retraction notice in `CHANGELOG.md` naming the yanked
  version, the reason, and the recommended replacement, and notify
  downstream consumers through the same announcement channel used for
  routine releases. When the yank responds to a security issue, follow
  the private disclosure path in `SECURITY.md` before publishing the
  retraction notice.
- Open a rollback pull request that either reverts the breaking change
  or advances to a corrected patch version. The next functional
  release is always a new version; a yanked version number is not
  re-used.
- If the flaw does not materialize and the yank is reversed later, run
  `cargo yank --undo --version <version>` against every previously
  yanked crate and update the `CHANGELOG.md` retraction notice
  accordingly.

### Post-publish confirmation

- Record the release tag, the published versions, any ownership
  changes executed in the same window, and any yank or unyank events
  in `CHANGELOG.md` under the released version heading.
- Confirm the released crates resolve cleanly from a fresh checkout
  by running the public `cargo add cow-sdk` path described in
  [Getting Started](getting-started.md) before announcing the release.

## 7. Provenance-Sensitive Parity Proof

Use this lane when the release needs explicit proof against pinned upstream
repositories instead of only the committed fixture contract.

Quick setup:

```text
git clone https://github.com/cowprotocol/contracts.git <path>/contracts
git clone https://github.com/cowprotocol/services.git <path>/services
```

If you prefer the parity maintainer to create the checkouts from the pinned
source lock (one blob-less clone per lock repository at `<path>/<id>`), run:

```text
cargo xtask parity sync --root <path>
```

Then deep-validate every pinned repository and the vendored OpenAPI body
against those independent roots:

```text
cargo parity-validate --upstream-root <path>
```

Rules:

- supplied roots must be independent git checkouts or worktrees
- same-checkout directory copies are not valid provenance evidence
- `release-readiness.yml` owns the routine automated provenance-sensitive lane

The `alloy-release-candidate.yml` workflow owns the alloy forward-compat
canary on scheduled and manually-dispatched runs. Set the `ALLOY_CANARY_REF`
repository variable to test a specific upstream ref; otherwise the workflow
uses its pinned SHA fallback. The workflow has no pull-request trigger, so
candidate drift is reported without changing routine PR gates.

There is no `cargo-semver-checks` CI lane through the pre-1.0 cycle: a pre-1.0
semver report against an unpublished baseline is non-blocking, so the lane was
removed and breaking changes remain the goal until 1.0 is on the runway. When
1.0 is on the runway, reintroduce semver checking as a gate so that a breaking
change against the prior published version requires a deliberate major version
bump in the workspace `Cargo.toml`.

## 8. WASM And Browser Surfaces

Build the WASM surfaces:

```text
cargo build --target wasm32-unknown-unknown -p cow-sdk
cargo build --target wasm32-unknown-unknown -p cow-sdk-app-data
cargo build --target wasm32-unknown-unknown -p cow-sdk-core
cargo build --target wasm32-unknown-unknown -p cow-sdk-js
```

Run the wasm-pack browser lanes in headless Firefox. The CI lanes provision the
runner with `browser-actions/setup-firefox` and geckodriver `0.36.0`; locally,
put a matching `geckodriver` on `PATH`:

```text
wasm-pack test --headless --firefox crates/js
```

JavaScript and TypeScript wasm package checks:

```text
cargo test -p cow-sdk-js --test host_pure_helpers
wasm-pack test crates/js --headless --firefox
bash crates/js/npm/scripts/build.sh
node crates/js/npm/scripts/verify-exports.mjs
pnpm install --dir e2e/wasm-typescript --frozen-lockfile
pnpm --dir e2e/wasm-typescript run test:vitest
pnpm --dir e2e/wasm-typescript run test:playwright
pnpm --dir e2e/wasm-typescript run test:type-check
pnpm install --dir e2e/wasm-typescript-cf --frozen-lockfile
pnpm --dir e2e/wasm-typescript-cf run test
pnpm --dir e2e/wasm-typescript-cf run test:type-check
```

The npm package name is baked into `package.template.json` and rendered into
`package.json` by `render-package-json.mjs` during the build. The package
exports map, declaration snapshots, every flavour's web (`…/edge`, `…/edge/wasm`)
and source-phase (`…/module`) subpaths, and generated `dist` metadata cleanup are
part of the release check.

## 9. Optional Validation Smoke

Use the smoke kit when a release needs a live deployment-registry presence
confirmation in addition to the deterministic proof surfaces above. The probe
covers the production and staging deployments of every registry row; the
deployment-only Lens chain carries none of them and is not probed.

```text
cargo registry-confirm --mode release --chain-ids 1,100,42161,8453,11155111,137,43114,56,9745,59144,57073
```

## 10. Manual Confirmation Before Publish

- if parity inputs changed, confirm that the pinned SHAs and fixture provenance
  still align

## 11. Software Bill Of Materials

The `release-readiness.yml` workflow emits a CycloneDX Software Bill of
Materials on every non-schedule run through a dedicated `sbom` job that
depends on the publication dry-run. The job installs `cargo-cyclonedx`,
regenerates the CycloneDX JSON for every workspace crate, and uploads
the combined output as a workflow artifact.

- Artifact name: `cow-rs-sbom-cyclonedx`
- Artifact format: CycloneDX JSON (`*.cdx.json`), one file per workspace
  crate, archived together
- Download location: GitHub Actions run page for the
  `release-readiness` workflow, under "Artifacts"
- Retention: 90 days

The SBOM artifact surfaces component provenance for reviewers and
downstream consumers on each release-readiness workflow run.

The same publication dry-run now generates a SLSA provenance attestation for
the crate tarballs and uploads it as `cow-rs-build-provenance` with 90-day
retention. Download it from the same release-readiness workflow run page when
reviewing provenance for a candidate publication.

## 12. Reproducible builds

The release artifacts produced by the release-readiness automation are
reproducible at two tiers.

**Tier one: source and lockfile reproducibility.** The workspace commits
`Cargo.lock` so every dependency resolves to the same version on every build,
and the Rust toolchain version is pinned via `rust-toolchain.toml`. A consumer
who checks out the tagged release commit and builds with the pinned toolchain
produces a build whose dependency tree matches the release-readiness build
byte-for-byte.

**Tier two: binary reproducibility (planned).** The WebAssembly artifacts
produced for the `cow-sdk-js` npm package build are not currently asserted to
be byte-reproducible. A future extension to the
release-readiness automation will pin the `wasm-pack` toolchain version,
capture the build environment provenance through the existing attestation
lane, and add a binary-reproducibility check that compares two independent
builds of the same source commit.

Provenance attestations for every published crate tarball ship as the
`cow-rs-build-provenance` artifact alongside the software-bill-of-materials
artifact. Consumers verify the attestation chain using any tool that supports
the in-toto attestation format.
