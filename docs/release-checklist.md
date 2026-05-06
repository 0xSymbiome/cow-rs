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
cargo audit --deny unsound --deny unmaintained --ignore RUSTSEC-2024-0436
cargo check-alloy-provider-invariant
cargo check-alloy-signer-invariant
cargo tree --invert alloy-provider -p cow-sdk-core -p cow-sdk-contracts -p cow-sdk-signing -p cow-sdk-orderbook -p cow-sdk-subgraph -p cow-sdk-app-data -p cow-sdk-trading -p cow-sdk-browser-wallet -p cow-sdk-transport-wasm -p cow-sdk-alloy-provider -p cow-sdk-alloy-signer -p cow-sdk-alloy -p cow-sdk
```

The native Alloy dependency gates enforce explicit allow-lists:
`alloy-provider` is allowed only in `cow-sdk-alloy-provider` and
`cow-sdk-alloy`, while `alloy-signer-local` is allowed only in
`cow-sdk-alloy-signer` and `cow-sdk-alloy`. CI normalises the raw Cargo tree
output via `cargo check-alloy-provider-invariant` and
`cargo check-alloy-signer-invariant`; contributors should use the wrappers
rather than reading raw Cargo tree output directly.

This command is guarded for drift by `scripts/check-release-docs-agree.sh`;
any mismatch against `docs/verification-matrix.md`,
`.github/workflows/_quality-gate.yml`, `CONTRIBUTING.md`, or
`PROPERTIES.md` fails the `docs-agree-on-release-gates` CI job.

- The `_quality-gate.yml` lane enforces both the `alloy-*` workspace-pin
  same-minor invariant and the inner-workspace WASM pin diff against the
  root workspace pins.

`cargo audit` is the blocking RustSec gate for published advisories. It keeps
vulnerabilities, unsound advisories, and unmaintained advisories blocking while
deriving its reviewed ignore arguments from `.github/config/deny.toml`.
`cargo deny` also runs with yanked advisory policy set to deny, so yanked
published-upstream cases must stay explicit in the public audit evidence until
a published replacement exists.

This command is guarded for drift by `scripts/check-release-docs-agree.sh`;
any mismatch against `docs/verification-matrix.md` or the advisory tolerance
register in `.github/config/deny.toml` fails the
`docs-agree-on-release-gates` CI job.

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
- Windows stable lane for light native host coverage
- `crate-checks.yml` for maintenance-depth crate isolation
- `codeql.yml` for semantic security analysis

## 4. Depth Reporting

`test-depth.yml` is the maintained depth-reporting lane. It publishes coverage
and mutation reports for follow-up work without defining threshold-based branch
protection.
The mutation report runs on the weekly schedule and can also be requested
manually with a narrower scope.

Coverage:

```text
cargo +nightly llvm-cov --workspace --all-features --doctests --json --summary-only --output-path target/coverage-summary.json --ignore-filename-regex "(^|/)(tests|examples|e2e)(/|$)|crates/subgraph/src/query_documents/|crates/subgraph/tests/schema_evidence/"
cargo +nightly llvm-cov --workspace --all-features --doctests --lcov --output-path target/coverage-lcov.info --ignore-filename-regex "(^|/)(tests|examples|e2e)(/|$)|crates/subgraph/src/query_documents/|crates/subgraph/tests/schema_evidence/"
```

Mutation:

```text
cargo mutants -p cow-sdk-contracts -p cow-sdk-signing -p cow-sdk-app-data --output target/mutants-report
cargo mutants -p cow-sdk-orderbook -p cow-sdk-trading --file crates/orderbook/src/request.rs --file crates/orderbook/src/transform.rs --file crates/trading/src/order.rs --file crates/trading/src/slippage.rs --annotations none --no-times --re "decoded_body|execute_with|calculate_total_fee|add_decimal_strings|sanitize_protocol_fee_bps|partner_fee_bps|calculate_unique_order_id|adjust_buy_amount" --output target/mutants-report-orderbook-trading
cargo mutants -p cow-sdk-subgraph -p cow-sdk-browser-wallet --file crates/subgraph/src/api.rs --file crates/subgraph/src/types.rs --file crates/browser-wallet/src/wallet.rs --file crates/browser-wallet/src/provider.rs --file crates/browser-wallet/src/error.rs --annotations none --no-times --re "run_query_with_config|config_with_override|base_url_for|deserialize_string_or_number|deserialize_optional_string_or_number|deserialize_u64_from_string_or_number|value_to_string|single_wallet|wallet_at|requires_explicit_selection|refresh_session|switch_or_add_chain|switch_chain_request|add_chain_request|validate_wallet_text|validate_wallet_url|query_accounts|query_chain_id|reset_session|parse_chain_id_value|parse_quantity_to_decimal|parse_address_array|transaction_to_rpc|from_rpc" --output target/mutants-report-subgraph-browser-wallet
```

Nightly retry soak:

```text
cargo test -p cow-sdk-orderbook --test request_contract retry_timeout_soak_exercises_deterministic_waveforms -- --ignored --exact
```

## 5. Repo-Local Parity And Publication Proof

Reserved-placeholder `0.0.1-reserved.0` crates.io and docs.rs entries may
already be live for published crate names. Treat those publishes as
name-reservation perimeter only; they do not satisfy the functional release
contract described in this checklist.

Validate the committed parity contract from the current checkout:

```text
cargo parity-validate --source-lock parity/source-lock.yaml
```

Before creating the release tag, re-affirm the source-lock pins:

- run `cargo parity-validate --source-lock parity/source-lock.yaml` and
  confirm the command reports no fixture or provenance diffs
- confirm the tag commit's `parity/source-lock.yaml` matches the upstream
  pins recorded in
  [Source-Lock Provenance Audit](audit/source-lock-provenance-audit.md), or
  confirm any pin movement landed through a reviewed bump with rationale
- confirm the
  [per-chain provenance](audit/deployment-registry-audit.md#per-chain-provenance)
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
cargo package -p cow-sdk-transport-wasm --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo package -p cow-sdk-trading --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-signing.path='crates/signing'" --config "patch.crates-io.cow-sdk-app-data.path='crates/app-data'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'" --config "patch.crates-io.cow-sdk-transport-wasm.path='crates/transport-wasm'"
cargo package -p cow-sdk-browser-wallet --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo package -p cow-sdk-alloy-provider --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo package -p cow-sdk-alloy-signer --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'"
cargo package -p cow-sdk-alloy --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-alloy-provider.path='crates/alloy-provider'" --config "patch.crates-io.cow-sdk-alloy-signer.path='crates/alloy-signer'"
cargo package -p cow-sdk --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-signing.path='crates/signing'" --config "patch.crates-io.cow-sdk-app-data.path='crates/app-data'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'" --config "patch.crates-io.cow-sdk-trading.path='crates/trading'" --config "patch.crates-io.cow-sdk-browser-wallet.path='crates/browser-wallet'" --config "patch.crates-io.cow-sdk-alloy-provider.path='crates/alloy-provider'" --config "patch.crates-io.cow-sdk-alloy-signer.path='crates/alloy-signer'" --config "patch.crates-io.cow-sdk-alloy.path='crates/alloy'"
```

Then run the registry-validation dry-run in the same order:

```text
cargo publish --dry-run -p cow-sdk-core --allow-dirty
cargo publish --dry-run -p cow-sdk-contracts --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'" --config "patch.crates-io.cow-sdk-subgraph.path='crates/subgraph'"
cargo publish --dry-run -p cow-sdk-app-data --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo publish --dry-run -p cow-sdk-orderbook --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo publish --dry-run -p cow-sdk-signing --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'"
cargo publish --dry-run -p cow-sdk-subgraph --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo publish --dry-run -p cow-sdk-transport-wasm --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo publish --dry-run -p cow-sdk-trading --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-signing.path='crates/signing'" --config "patch.crates-io.cow-sdk-app-data.path='crates/app-data'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'" --config "patch.crates-io.cow-sdk-transport-wasm.path='crates/transport-wasm'"
cargo publish --dry-run -p cow-sdk-browser-wallet --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo publish --dry-run -p cow-sdk-alloy-provider --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo publish --dry-run -p cow-sdk-alloy-signer --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'"
cargo publish --dry-run -p cow-sdk-alloy --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-alloy-provider.path='crates/alloy-provider'" --config "patch.crates-io.cow-sdk-alloy-signer.path='crates/alloy-signer'"
cargo publish --dry-run -p cow-sdk --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-signing.path='crates/signing'" --config "patch.crates-io.cow-sdk-app-data.path='crates/app-data'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'" --config "patch.crates-io.cow-sdk-trading.path='crates/trading'" --config "patch.crates-io.cow-sdk-browser-wallet.path='crates/browser-wallet'" --config "patch.crates-io.cow-sdk-alloy-provider.path='crates/alloy-provider'" --config "patch.crates-io.cow-sdk-alloy-signer.path='crates/alloy-signer'" --config "patch.crates-io.cow-sdk-alloy.path='crates/alloy'"
```

## 6. Manual Publish Sequence

The functional `0.1.0` crates.io release publishes the crates of the
`cow-sdk` family in dependency order so every step depends only on a
version already indexed by the registry. Reserved-placeholder
`0.0.1-reserved.0` publishes are independent of the functional release
and do not satisfy this sequence.

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
cargo publish -p cow-sdk-transport-wasm
cargo publish -p cow-sdk-trading
cargo publish -p cow-sdk-browser-wallet
cargo publish -p cow-sdk-alloy-provider
cargo publish -p cow-sdk-alloy-signer
cargo publish -p cow-sdk-alloy
cargo publish -p cow-sdk
```

### Index propagation backoff

- After each `cargo publish` returns successfully, wait 30 to 60 seconds
  before the next step so the freshly-published version appears in the
  crates.io index that the next dependency resolution reads.
- If the next publish fails with a missing-dependency or index-cache
  resolution error, pause another 30 to 60 seconds and retry. Repeat up
  to three times before escalating.
- For the `cow-sdk-trading`, `cow-sdk-browser-wallet`, and `cow-sdk`
  steps, which pull multiple freshly-published first-party dependencies,
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
git clone https://github.com/cowprotocol/cow-sdk.git <path>/cow-sdk
git clone https://github.com/cowprotocol/contracts.git <path>/contracts
git clone https://github.com/cowprotocol/services.git <path>/services
```

If you prefer the parity maintainer to create the sibling checkouts from the
pinned source lock, run:

```text
cargo parity-provision-upstreams --source-lock parity/source-lock.yaml --output-root <path>
```

Before relying on manually supplied upstream roots, run the report-only root
check so parent-checkout, remote, or commit mismatches are visible:

```text
cargo check-source-lock-roots --cow-sdk-root <path>/cow-sdk --contracts-root <path>/contracts --services-root <path>/services
```

Then validate against those independent roots:

```text
cargo parity-validate --source-lock parity/source-lock.yaml --cow-sdk-root <path>/cow-sdk --contracts-root <path>/contracts --services-root <path>/services
```

Rules:

- supplied roots must be independent git checkouts or worktrees
- same-checkout directory copies are not valid provenance evidence
- `release-readiness.yml` owns the routine automated provenance-sensitive lane

The `services-drift.yml` workflow runs weekly against the pinned upstream
services, contracts, and cow-sdk repositories. It records OpenAPI drift,
newly-added services error tags, request or response shape changes, generated
settlement chain-table drift, and supported-chain README drift as a
`parity-drift` tracking report before those changes reach the release window.
It never mutates `parity/source-lock.yaml`; source-lock movement remains a
reviewed pull request.

The `alloy-release-candidate.yml` workflow owns the alloy forward-compat
canary on scheduled and manually-dispatched runs. Set the `ALLOY_CANARY_REF`
repository variable to test a specific upstream ref; otherwise the workflow
uses its pinned SHA fallback. The workflow has no pull-request trigger, so
candidate drift is reported without changing routine PR gates.

Continuous integration runs `cargo-semver-checks` on every pull request that
touches a published crate's `src/` tree. The lane is informational through
the pre-1.0 cycle; the workflow summary reports each crate's compatibility
status against the most-recently-published version on the public registry,
but a non-zero report does not block the merge. At the 1.0 release boundary,
remove `continue-on-error: true` from the `cargo-semver-checks` step in
`.github/workflows/_quality-gate.yml` to promote the lane from informational
to gating; from that point a breaking change against the prior published
version requires a deliberate major version bump in the workspace
`Cargo.toml`.

## 8. WASM And Browser Surfaces

Build the WASM surfaces:

```text
cargo build --target wasm32-unknown-unknown -p cow-sdk
cargo build --target wasm32-unknown-unknown -p cow-sdk --features browser-wallet
cargo build --target wasm32-unknown-unknown -p cow-sdk-app-data
cargo build --target wasm32-unknown-unknown -p cow-sdk-transport-wasm
cargo build --target wasm32-unknown-unknown --manifest-path examples/wasm/browser-wallet-console/Cargo.toml
```

Set up the pinned browser runner before every `wasm-pack` browser lane:

```text
mkdir -p target/wasm-runner
cargo wasm-runner-setup --webdriver-json target/wasm-runner/webdriver.json
export WASM_BINDGEN_TEST_WEBDRIVER_JSON="$(pwd)/target/wasm-runner/webdriver.json"
export WASM_BINDGEN_TEST_CHROMEDRIVER="$(python - <<'PY'
import json
import os
with open(os.environ["WASM_BINDGEN_TEST_WEBDRIVER_JSON"], encoding="utf-8") as handle:
    print(json.load(handle)["cow:wasmRunner"]["chromedriver"])
PY
)"
```

Deterministic SDK verification console checks:

```text
cd examples/wasm/sdk-verification-console
wasm-pack test --headless --chrome --chromedriver "$WASM_BINDGEN_TEST_CHROMEDRIVER"
```

```text
bun install --cwd e2e/sdk-verification
bun run --cwd e2e/sdk-verification playwright install --with-deps chromium
bun run --cwd e2e/sdk-verification test
```

Deterministic browser-wallet console checks:

```text
# 1. Host-side crate
cargo test -p cow-sdk-browser-wallet

# 2. Direct-bridge wasm (browser-wallet crate)
cd crates/browser-wallet && wasm-pack test --headless --chrome --chromedriver "$WASM_BINDGEN_TEST_CHROMEDRIVER"

# 3. WASM build of the published SDK with the browser-wallet feature
cargo build --target wasm32-unknown-unknown -p cow-sdk --features browser-wallet

# 4. Browser-wallet console WASM build
cargo build --target wasm32-unknown-unknown --manifest-path examples/wasm/browser-wallet-console/Cargo.toml

# 5. Browser-wallet console host-side tests
cargo test --manifest-path examples/wasm/browser-wallet-console/Cargo.toml

# 6. Browser-wallet console wasm-bindgen tests
cd examples/wasm/browser-wallet-console \
  && wasm-pack build --target web \
  && wasm-pack test --headless --chrome --chromedriver "$WASM_BINDGEN_TEST_CHROMEDRIVER"

# 7. Playwright DOM lane under Chromium and Firefox
bun install --cwd e2e/browser-wallet --frozen-lockfile
bun run --cwd e2e/browser-wallet playwright install --with-deps chromium firefox
bun run --cwd e2e/browser-wallet test
```

The deterministic browser-wallet Playwright lane excludes live extension specs.
Use `scripts/validation-smoke/browser-wallet-live/README.md` for the manual
extension-backed canary when a release needs installed-wallet confirmation.

## 9. Optional Validation Smoke

Use the smoke kit when a change needs live service confirmation, live
extension-backed wallet confirmation, or deployed-page inspection in addition
to the deterministic proof surfaces above.

```text
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- orderbook-live
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- subgraph-live
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- browser-wallet-live --url http://127.0.0.1:8081
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- wasm-pages --sdk-verification-url https://<owner>.github.io/<repo>/sdk-verification-console/ --browser-wallet-url https://<owner>.github.io/<repo>/browser-wallet-console/
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- all
```

## 10. Manual Confirmation Before Publish

- serve the WASM examples over HTTP and confirm that the built artifacts load
- if the browser-wallet console changed, run an extension-backed spot check on
  a supported chain
- if GitHub Pages content changed, inspect the deployed console pages after the
  Pages workflow completes
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
produced by `wasm-pack build` in `examples/wasm/*/pkg/` are not currently
asserted to be byte-reproducible. A future extension to the
release-readiness automation will pin the `wasm-pack` toolchain version,
capture the build environment provenance through the existing attestation
lane, and add a binary-reproducibility check that compares two independent
builds of the same source commit.

Provenance attestations for every published crate tarball ship as the
`cow-rs-build-provenance` artifact alongside the software-bill-of-materials
artifact. Consumers verify the attestation chain using any tool that supports
the in-toto attestation format.
