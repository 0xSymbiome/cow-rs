# Repository File Map

> **Branch:** `feat/ferrous-foundation` &nbsp;&middot;&nbsp; **HEAD:** `5be3d6b` &nbsp;&middot;&nbsp; **Generated:** 2026-06-02  
> **Total tracked files:** **1420**

A navigable inventory of every file tracked by Git on this branch, grouped by the role each directory plays in the workspace. Use the table of contents to jump straight to a section; full file listings are collapsed by default so the high-level shape stays scannable.

---

## Table of contents

1. [At a glance](#at-a-glance)
2. [Top-level layout](#top-level-layout)
3. [File composition by extension](#file-composition-by-extension)
4. [Workspace crates (`crates/`)](#workspace-crates-crates)
5. [Examples (`examples/`)](#examples-examples)
6. [End-to-end harnesses (`e2e/`)](#end-to-end-harnesses-e2e)
7. [Maintenance scripts (`scripts/`)](#maintenance-scripts-scripts)
8. [Upstream parity (`parity/`)](#upstream-parity-parity)
9. [Documentation (`docs/`)](#documentation-docs)
10. [Fuzzing (`fuzz/`)](#fuzzing-fuzz)
11. [CI & repo-level configuration](#ci--repo-level-configuration)
12. [Full file index](#full-file-index)

---

## At a glance

- **794 files** live under `crates/` — 18 workspace member crates make up roughly 56% of the repo.
- **153 files** under `docs/` are mostly architecture decision records and audit notes.
- **73 files** under `parity/` are golden fixtures captured from upstream services to keep the Rust SDK byte-compatible.
- **107 files** under `fuzz/` cover cargo-fuzz targets and their seed corpora.
- **137 files** under `examples/` + `e2e/` are runnable demos and integration harnesses.
- **86 files** under `scripts/` are maintenance tool crates (parity refresh, policy refresh, validation runners).

---

## Top-level layout

| Path | Files | Purpose |
|------|------:|---------|
| `crates/` | 794 | Workspace member crates (the SDK itself) |
| `docs/` | 153 | Architecture decision records, audit notes, provider notes |
| `fuzz/` | 107 | cargo-fuzz targets, corpora, and failure artifacts |
| `examples/` | 89 | Runnable usage examples (Rust + TypeScript) |
| `scripts/` | 86 | Internal maintenance tool crates |
| `parity/` | 73 | Golden fixtures + pinned specs from upstream services |
| `e2e/` | 48 | End-to-end integration harnesses |
| `.github/` | 37 | GitHub Actions workflows and repo config |
| `tests/` | 15 | Workspace-level integration tests |
| `.cargo/` | 2 | Cargo configuration |
| `.gitattributes` | 1 | Git attributes |
| `rust-toolchain.toml` | 1 | Pinned Rust toolchain |
| `ROADMAP.md` | 1 | Roadmap document |
| `README.md` | 1 | Top-level README |
| `PROPERTIES.md` | 1 | Property-based testing index |
| `.githooks/` | 1 | Tracked git hook scripts |
| `CHANGELOG.md` | 1 | Release changelog |
| `MAP.md` | 1 |  |
| `LICENSE` | 1 | License text |
| `.gitignore` | 1 | Top-level git ignore rules |
| `.yamllint` | 1 | YAML lint configuration |
| `SECURITY.md` | 1 | Security policy |
| `Cargo.lock` | 1 | Workspace lockfile |
| `Cargo.toml` | 1 | Workspace manifest |
| `llvm-cov-summary.txt` | 1 | Coverage summary snapshot |
| `CONTRIBUTING.md` | 1 | Contribution guide |

---

## File composition by extension

| Extension | Files | Typical role |
|-----------|------:|--------------|
| `.rs` | 703 | Rust source and tests |
| `.md` | 250 | Markdown docs (ADRs, audit notes, READMEs) |
| `.json` | 156 | JSON schemas, ABIs, parity fixtures |
| `.ts` | 74 | TypeScript (examples, e2e, wasm bindings) |
| `.toml` | 52 | Cargo manifests and tool configs |
| `.sol` | 40 | Solidity sources / vendored contract code |
| `.yaml` | 35 | CI workflows, OpenAPI specs, config |
| `.yml` | 27 | CI workflows and config |
| `.stderr` | 19 | trybuild compile-fail snapshots |
| `.lock` | 10 | Cargo / package lockfiles |
| `.sh` | 8 | Shell scripts |
| `.txt` | 8 | Plain text fixtures / summaries |
| `.mjs` | 6 | JavaScript modules |
| `(none)` | 6 |  |
| `.html` | 5 | Static HTML for browser examples |
| `.graphql` | 4 | GraphQL queries (subgraph) |
| `.gitignore` | 4 |  |
| `.bin` | 2 | Binary fixtures |
| `.sha256` | 2 | Checksum files |
| `.keep` | 2 |  |
| `.snap` | 2 | Snapshot test outputs |
| `.npmignore` | 1 |  |
| `.jsonc` | 1 |  |
| `.gitattributes` | 1 |  |
| `.yamllint` | 1 |  |
| `.proptest-regressions` | 1 | proptest regression seeds |

---

## Workspace crates (`crates/`)

18 member crates compose the SDK. Sizes are file counts, not lines of code. Descriptions are pulled live from each crate's `Cargo.toml`.

| Crate | Files | Purpose |
|-------|------:|---------|
| [`contracts`](crates/contracts) | 160 | CoW Protocol low-level contracts helpers for hashing, settlement encoding, and on-chain interaction plumbing |
| [`wasm`](crates/wasm) | 105 | TypeScript-callable wasm-bindgen leaf for the CoW Protocol Rust SDK |
| [`app-data`](crates/app-data) | 99 | CoW Protocol app-data encoding, schema validation, and CID compatibility |
| [`trading`](crates/trading) | 81 | High-level CoW Protocol trading orchestration surface |
| [`core`](crates/core) | 59 | Shared CoW Protocol core types and validation primitives |
| [`orderbook`](crates/orderbook) | 40 | Typed CoW Protocol orderbook client models and decoding helpers |
| [`cow-shed`](crates/cow-shed) | 32 | CoW Protocol COW Shed proxy address, EIP-712, and calldata helpers |
| [`browser-wallet`](crates/browser-wallet) | 29 | Browser wallet integration for the CoW Protocol Rust SDK |
| [`signing`](crates/signing) | 27 | Deterministic CoW Protocol order hashing, EIP-712 signing, and UID helpers |
| [`subgraph`](crates/subgraph) | 27 | Typed CoW Protocol subgraph query primitives |
| [`alloy`](crates/alloy) | 27 | Composed Alloy provider and signer adapter for the CoW Protocol Rust SDK |
| [`alloy-provider`](crates/alloy-provider) | 25 | Alloy-backed read-only Provider adapter for the CoW Protocol Rust SDK |
| [`alloy-signer`](crates/alloy-signer) | 22 | Alloy-backed local-keystore Signer adapter for the CoW Protocol Rust SDK |
| [`sdk`](crates/sdk) | 18 | Facade crate for CoW Protocol Rust SDK surfaces |
| [`transport-policy`](crates/transport-policy) | 17 | Retry, rate-limit, and transport classification policy for CoW Protocol SDK HTTP clients |
| [`pure-helpers`](crates/pure-helpers) | 10 | Runtime-neutral helper functions for the CoW Protocol Rust SDK wasm surface |
| [`transport-wasm`](crates/transport-wasm) | 8 | Browser fetch-based HTTP transport for the CoW Protocol Rust SDK |
| [`composable`](crates/composable) | 8 | Reserved crate manifest for future CoW Protocol composable order helpers |

---

## Examples (`examples/`)

| Example | Files | Purpose |
|---------|------:|---------|
| [`native`](examples/native) | 32 | Native Rust scenario walkthroughs |
| [`wasm`](examples/wasm) | 26 | Browser console scenarios (raw wasm) |
| [`wasm-typescript-browser-mm`](examples/wasm-typescript-browser-mm) | 9 | TypeScript browser market-maker demo |
| [`wasm-typescript-cloudflare-proxy`](examples/wasm-typescript-cloudflare-proxy) | 14 | TypeScript Cloudflare Worker proxy example |
| [`wasm-typescript-node-viem`](examples/wasm-typescript-node-viem) | 6 | Node + viem TypeScript example |

---

## End-to-end harnesses (`e2e/`)

| Harness | Files | Purpose |
|---------|------:|---------|
| [`browser-wallet`](e2e/browser-wallet) | 10 | Browser-wallet end-to-end harness |
| [`sdk-verification`](e2e/sdk-verification) | 9 | SDK facade verification harness |
| [`wasm-typescript`](e2e/wasm-typescript) | 14 | Wasm + TypeScript integration harness |
| [`wasm-typescript-cf`](e2e/wasm-typescript-cf) | 11 | Wasm + TypeScript Cloudflare harness |
| [`wasm-typescript-deno`](e2e/wasm-typescript-deno) | 3 | Wasm + TypeScript Deno harness |

---

## Maintenance scripts (`scripts/`)

| Script crate | Files | Purpose |
|---------|------:|---------|
| [`parity-maintainer`](scripts/parity-maintainer) | 30 | Upstream parity fixture refresh + drift detection |
| [`policy-maintainer`](scripts/policy-maintainer) | 38 | Transport policy config maintenance |
| [`validation-depth`](scripts/validation-depth) | 4 | Deep validation runner |
| [`validation-smoke`](scripts/validation-smoke) | 11 | Smoke validation runner |

---

## Upstream parity (`parity/`)

| Subtree | Files | Purpose |
|---------|------:|---------|
| [`dependency-audit`](parity/dependency-audit) | 1 | Dependency audit reports |
| [`fixtures`](parity/fixtures) | 52 | Golden fixtures captured from upstream services |
| [`openapi`](parity/openapi) | 10 | OpenAPI specs pinned for parity |
| [`source-lock`](parity/source-lock) | 1 | Upstream source lockfiles |

---

## Documentation (`docs/`)

| Subtree | Files | Purpose |
|---------|------:|---------|
| [`adr`](docs/adr) | 63 | Architecture Decision Records |
| [`audit`](docs/audit) | 65 | Audit notes and review artifacts |
| [`providers`](docs/providers) | 2 | Provider integration notes |

---

## Fuzzing (`fuzz/`)

| Subtree | Files | Purpose |
|---------|------:|---------|
| [`corpus`](fuzz/corpus) | 52 | Seed corpora per target |
| [`fuzz_targets`](fuzz/fuzz_targets) | 52 | cargo-fuzz target sources |

---

## CI & repo-level configuration

| Path | Files | Purpose |
|------|------:|---------|
| `.github/workflows/` | 22 | GitHub Actions pipelines |
| `.github/config/`    | 9 | Shared CI config |
| `.githooks/`         | 1 | Tracked git hooks |
| `.cargo/`            | 2 | Cargo config (e.g. rustflags) |
| `tests/`             | 15 | Workspace-level integration tests |

---

## Full file index

Every tracked file, grouped by the directory it lives in. Each section is collapsed by default — click to expand.

<details>
<summary><code>(repo root)</code> &mdash; 15 file(s)</summary>

- [`.gitattributes`](.gitattributes)
- [`.gitignore`](.gitignore)
- [`.yamllint`](.yamllint)
- [`Cargo.lock`](Cargo.lock)
- [`Cargo.toml`](Cargo.toml)
- [`CHANGELOG.md`](CHANGELOG.md)
- [`CONTRIBUTING.md`](CONTRIBUTING.md)
- [`LICENSE`](LICENSE)
- [`llvm-cov-summary.txt`](llvm-cov-summary.txt)
- [`MAP.md`](MAP.md)
- [`PROPERTIES.md`](PROPERTIES.md)
- [`README.md`](README.md)
- [`ROADMAP.md`](ROADMAP.md)
- [`rust-toolchain.toml`](rust-toolchain.toml)
- [`SECURITY.md`](SECURITY.md)

</details>

<details>
<summary><code>.cargo/</code> &mdash; 2 file(s)</summary>

- [`config.toml`](.cargo/config.toml)
- [`mutants.toml`](.cargo/mutants.toml)

</details>

<details>
<summary><code>.githooks/</code> &mdash; 1 file(s)</summary>

- [`commit-msg`](.githooks/commit-msg)

</details>

<details>
<summary><code>.github/</code> &mdash; 2 file(s)</summary>

- [`commit-template.md`](.github/commit-template.md)
- [`dependabot.yml`](.github/dependabot.yml)

</details>

<details>
<summary><code>.github/codeql/</code> &mdash; 1 file(s)</summary>

- [`codeql-config.yml`](.github/codeql/codeql-config.yml)

</details>

<details>
<summary><code>.github/config/</code> &mdash; 9 file(s)</summary>

- [`audit-refresh-map.yml`](.github/config/audit-refresh-map.yml)
- [`deny-unknown-fields-allowlist.yaml`](.github/config/deny-unknown-fields-allowlist.yaml)
- [`deny.toml`](.github/config/deny.toml)
- [`enum-policy.yaml`](.github/config/enum-policy.yaml)
- [`nextest.toml`](.github/config/nextest.toml)
- [`panic-allowlist.yaml`](.github/config/panic-allowlist.yaml)
- [`principle-adr-map.yaml`](.github/config/principle-adr-map.yaml)
- [`typos.toml`](.github/config/typos.toml)
- [`wasm-test-versions.yaml`](.github/config/wasm-test-versions.yaml)

</details>

<details>
<summary><code>.github/ISSUE_TEMPLATE/</code> &mdash; 1 file(s)</summary>

- [`services-drift-report.yml`](.github/ISSUE_TEMPLATE/services-drift-report.yml)

</details>

<details>
<summary><code>.github/release-evidence/</code> &mdash; 2 file(s)</summary>

- [`release-readiness-status-0.1.0.yaml`](.github/release-evidence/release-readiness-status-0.1.0.yaml)
- [`validation-evidence-0.1.0.md`](.github/release-evidence/validation-evidence-0.1.0.md)

</details>

<details>
<summary><code>.github/workflows/</code> &mdash; 22 file(s)</summary>

- [`_quality-gate.yml`](.github/workflows/_quality-gate.yml)
- [`alloy-release-candidate.yml`](.github/workflows/alloy-release-candidate.yml)
- [`benchmarks.yml`](.github/workflows/benchmarks.yml)
- [`browser-wallet-e2e.yml`](.github/workflows/browser-wallet-e2e.yml)
- [`ci.yml`](.github/workflows/ci.yml)
- [`codeql.yml`](.github/workflows/codeql.yml)
- [`commit-format.yml`](.github/workflows/commit-format.yml)
- [`crate-checks.yml`](.github/workflows/crate-checks.yml)
- [`docs-quality.yml`](.github/workflows/docs-quality.yml)
- [`encode-prefixed-grep-gate.yml`](.github/workflows/encode-prefixed-grep-gate.yml)
- [`fuzz.yml`](.github/workflows/fuzz.yml)
- [`never-swap-gates.yml`](.github/workflows/never-swap-gates.yml)
- [`policy-maintainer.yml`](.github/workflows/policy-maintainer.yml)
- [`release-readiness.yml`](.github/workflows/release-readiness.yml)
- [`release-version-coherence.yml`](.github/workflows/release-version-coherence.yml)
- [`retry-soak.yml`](.github/workflows/retry-soak.yml)
- [`sdk-verification-e2e.yml`](.github/workflows/sdk-verification-e2e.yml)
- [`services-drift.yml`](.github/workflows/services-drift.yml)
- [`test-depth.yml`](.github/workflows/test-depth.yml)
- [`wasm-imports-grep-gate.yml`](.github/workflows/wasm-imports-grep-gate.yml)
- [`wasm-pages.yml`](.github/workflows/wasm-pages.yml)
- [`wasm.yml`](.github/workflows/wasm.yml)

</details>

<details>
<summary><code>crates/alloy/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/alloy/Cargo.toml)
- [`README.md`](crates/alloy/README.md)

</details>

<details>
<summary><code>crates/alloy-provider/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/alloy-provider/Cargo.toml)
- [`README.md`](crates/alloy-provider/README.md)

</details>

<details>
<summary><code>crates/alloy-provider/src/</code> &mdash; 7 file(s)</summary>

- [`builder.rs`](crates/alloy-provider/src/builder.rs)
- [`client.rs`](crates/alloy-provider/src/client.rs)
- [`conversion.rs`](crates/alloy-provider/src/conversion.rs)
- [`error.rs`](crates/alloy-provider/src/error.rs)
- [`lib.rs`](crates/alloy-provider/src/lib.rs)
- [`provider.rs`](crates/alloy-provider/src/provider.rs)
- [`read_contract.rs`](crates/alloy-provider/src/read_contract.rs)

</details>

<details>
<summary><code>crates/alloy-provider/tests/</code> &mdash; 10 file(s)</summary>

- [`builder_contract.rs`](crates/alloy-provider/tests/builder_contract.rs)
- [`cancellation_contract.rs`](crates/alloy-provider/tests/cancellation_contract.rs)
- [`compile_fail.rs`](crates/alloy-provider/tests/compile_fail.rs)
- [`dependency_boundary_contract.rs`](crates/alloy-provider/tests/dependency_boundary_contract.rs)
- [`error_class_contract.rs`](crates/alloy-provider/tests/error_class_contract.rs)
- [`provider_contract.rs`](crates/alloy-provider/tests/provider_contract.rs)
- [`read_contract_no_panic.rs`](crates/alloy-provider/tests/read_contract_no_panic.rs)
- [`read_contract_parity.rs`](crates/alloy-provider/tests/read_contract_parity.rs)
- [`redaction_contract.rs`](crates/alloy-provider/tests/redaction_contract.rs)
- [`seam_contract.rs`](crates/alloy-provider/tests/seam_contract.rs)

</details>

<details>
<summary><code>crates/alloy-provider/tests/trybuild/</code> &mdash; 6 file(s)</summary>

- [`external_marker_construction_fails.rs`](crates/alloy-provider/tests/trybuild/external_marker_construction_fails.rs)
- [`external_marker_construction_fails.stderr`](crates/alloy-provider/tests/trybuild/external_marker_construction_fails.stderr)
- [`no_signer.rs`](crates/alloy-provider/tests/trybuild/no_signer.rs)
- [`no_signer.stderr`](crates/alloy-provider/tests/trybuild/no_signer.stderr)
- [`no_signing_provider.rs`](crates/alloy-provider/tests/trybuild/no_signing_provider.rs)
- [`no_signing_provider.stderr`](crates/alloy-provider/tests/trybuild/no_signing_provider.stderr)

</details>

<details>
<summary><code>crates/alloy-signer/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/alloy-signer/Cargo.toml)
- [`README.md`](crates/alloy-signer/README.md)

</details>

<details>
<summary><code>crates/alloy-signer/src/</code> &mdash; 5 file(s)</summary>

- [`builder.rs`](crates/alloy-signer/src/builder.rs)
- [`conversion.rs`](crates/alloy-signer/src/conversion.rs)
- [`error.rs`](crates/alloy-signer/src/error.rs)
- [`lib.rs`](crates/alloy-signer/src/lib.rs)
- [`signer.rs`](crates/alloy-signer/src/signer.rs)

</details>

<details>
<summary><code>crates/alloy-signer/tests/</code> &mdash; 9 file(s)</summary>

- [`cancellation_contract.rs`](crates/alloy-signer/tests/cancellation_contract.rs)
- [`compile_fail.rs`](crates/alloy-signer/tests/compile_fail.rs)
- [`dependency_boundary_contract.rs`](crates/alloy-signer/tests/dependency_boundary_contract.rs)
- [`eip191_reference_vectors.rs`](crates/alloy-signer/tests/eip191_reference_vectors.rs)
- [`eip712_reference_vectors.rs`](crates/alloy-signer/tests/eip712_reference_vectors.rs)
- [`proptests.rs`](crates/alloy-signer/tests/proptests.rs)
- [`redaction_contract.rs`](crates/alloy-signer/tests/redaction_contract.rs)
- [`signer_contract.rs`](crates/alloy-signer/tests/signer_contract.rs)
- [`signer_error_trait_contract.rs`](crates/alloy-signer/tests/signer_error_trait_contract.rs)

</details>

<details>
<summary><code>crates/alloy-signer/tests/trybuild/</code> &mdash; 6 file(s)</summary>

- [`external_marker_construction_fails.rs`](crates/alloy-signer/tests/trybuild/external_marker_construction_fails.rs)
- [`external_marker_construction_fails.stderr`](crates/alloy-signer/tests/trybuild/external_marker_construction_fails.stderr)
- [`no_provider.rs`](crates/alloy-signer/tests/trybuild/no_provider.rs)
- [`no_provider.stderr`](crates/alloy-signer/tests/trybuild/no_provider.stderr)
- [`no_signing_provider.rs`](crates/alloy-signer/tests/trybuild/no_signing_provider.rs)
- [`no_signing_provider.stderr`](crates/alloy-signer/tests/trybuild/no_signing_provider.stderr)

</details>

<details>
<summary><code>crates/alloy/src/</code> &mdash; 6 file(s)</summary>

- [`builder.rs`](crates/alloy/src/builder.rs)
- [`client.rs`](crates/alloy/src/client.rs)
- [`conversion.rs`](crates/alloy/src/conversion.rs)
- [`error.rs`](crates/alloy/src/error.rs)
- [`handle.rs`](crates/alloy/src/handle.rs)
- [`lib.rs`](crates/alloy/src/lib.rs)

</details>

<details>
<summary><code>crates/alloy/tests/</code> &mdash; 15 file(s)</summary>

- [`builder_contract.rs`](crates/alloy/tests/builder_contract.rs)
- [`cancellation_contract.rs`](crates/alloy/tests/cancellation_contract.rs)
- [`chain_coherence_mismatch.rs`](crates/alloy/tests/chain_coherence_mismatch.rs)
- [`chain_coherence.rs`](crates/alloy/tests/chain_coherence.rs)
- [`compile_fail.rs`](crates/alloy/tests/compile_fail.rs)
- [`eip712_reference_vectors.rs`](crates/alloy/tests/eip712_reference_vectors.rs)
- [`error_contract.rs`](crates/alloy/tests/error_contract.rs)
- [`handle_survives_drop.rs`](crates/alloy/tests/handle_survives_drop.rs)
- [`no_broadcast_for_sign_transaction.rs`](crates/alloy/tests/no_broadcast_for_sign_transaction.rs)
- [`provider_contract.rs`](crates/alloy/tests/provider_contract.rs)
- [`read_contract_contract.rs`](crates/alloy/tests/read_contract_contract.rs)
- [`redaction_contract.rs`](crates/alloy/tests/redaction_contract.rs)
- [`send_transaction_does_not_wait_for_confirmation.rs`](crates/alloy/tests/send_transaction_does_not_wait_for_confirmation.rs)
- [`signer_error_trait_contract.rs`](crates/alloy/tests/signer_error_trait_contract.rs)
- [`signing_provider_contract.rs`](crates/alloy/tests/signing_provider_contract.rs)

</details>

<details>
<summary><code>crates/alloy/tests/trybuild/</code> &mdash; 4 file(s)</summary>

- [`no_provider_on_handle.rs`](crates/alloy/tests/trybuild/no_provider_on_handle.rs)
- [`no_provider_on_handle.stderr`](crates/alloy/tests/trybuild/no_provider_on_handle.stderr)
- [`no_signer_on_client.rs`](crates/alloy/tests/trybuild/no_signer_on_client.rs)
- [`no_signer_on_client.stderr`](crates/alloy/tests/trybuild/no_signer_on_client.stderr)

</details>

<details>
<summary><code>crates/app-data/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/app-data/Cargo.toml)
- [`README.md`](crates/app-data/README.md)

</details>

<details>
<summary><code>crates/app-data/benches/</code> &mdash; 1 file(s)</summary>

- [`stringify.rs`](crates/app-data/benches/stringify.rs)

</details>

<details>
<summary><code>crates/app-data/schemas/</code> &mdash; 27 file(s)</summary>

- [`definitions.json`](crates/app-data/schemas/definitions.json)
- [`v0.1.0.json`](crates/app-data/schemas/v0.1.0.json)
- [`v0.10.0.json`](crates/app-data/schemas/v0.10.0.json)
- [`v0.11.0.json`](crates/app-data/schemas/v0.11.0.json)
- [`v0.2.0.json`](crates/app-data/schemas/v0.2.0.json)
- [`v0.3.0.json`](crates/app-data/schemas/v0.3.0.json)
- [`v0.4.0.json`](crates/app-data/schemas/v0.4.0.json)
- [`v0.5.0.json`](crates/app-data/schemas/v0.5.0.json)
- [`v0.6.0.json`](crates/app-data/schemas/v0.6.0.json)
- [`v0.7.0.json`](crates/app-data/schemas/v0.7.0.json)
- [`v0.8.0.json`](crates/app-data/schemas/v0.8.0.json)
- [`v0.9.0.json`](crates/app-data/schemas/v0.9.0.json)
- [`v1.0.0.json`](crates/app-data/schemas/v1.0.0.json)
- [`v1.1.0.json`](crates/app-data/schemas/v1.1.0.json)
- [`v1.10.0.json`](crates/app-data/schemas/v1.10.0.json)
- [`v1.11.0.json`](crates/app-data/schemas/v1.11.0.json)
- [`v1.12.0.json`](crates/app-data/schemas/v1.12.0.json)
- [`v1.13.0.json`](crates/app-data/schemas/v1.13.0.json)
- [`v1.14.0.json`](crates/app-data/schemas/v1.14.0.json)
- [`v1.2.0.json`](crates/app-data/schemas/v1.2.0.json)
- [`v1.3.0.json`](crates/app-data/schemas/v1.3.0.json)
- [`v1.4.0.json`](crates/app-data/schemas/v1.4.0.json)
- [`v1.5.0.json`](crates/app-data/schemas/v1.5.0.json)
- [`v1.6.0.json`](crates/app-data/schemas/v1.6.0.json)
- [`v1.7.0.json`](crates/app-data/schemas/v1.7.0.json)
- [`v1.8.0.json`](crates/app-data/schemas/v1.8.0.json)
- [`v1.9.0.json`](crates/app-data/schemas/v1.9.0.json)

</details>

<details>
<summary><code>crates/app-data/schemas/bridging/</code> &mdash; 4 file(s)</summary>

- [`v0.1.0.json`](crates/app-data/schemas/bridging/v0.1.0.json)
- [`v0.2.0.json`](crates/app-data/schemas/bridging/v0.2.0.json)
- [`v0.3.0.json`](crates/app-data/schemas/bridging/v0.3.0.json)
- [`v0.4.0.json`](crates/app-data/schemas/bridging/v0.4.0.json)

</details>

<details>
<summary><code>crates/app-data/schemas/flashloan/</code> &mdash; 2 file(s)</summary>

- [`v0.1.0.json`](crates/app-data/schemas/flashloan/v0.1.0.json)
- [`v0.2.0.json`](crates/app-data/schemas/flashloan/v0.2.0.json)

</details>

<details>
<summary><code>crates/app-data/schemas/hook/</code> &mdash; 2 file(s)</summary>

- [`v0.1.0.json`](crates/app-data/schemas/hook/v0.1.0.json)
- [`v0.2.0.json`](crates/app-data/schemas/hook/v0.2.0.json)

</details>

<details>
<summary><code>crates/app-data/schemas/hooks/</code> &mdash; 2 file(s)</summary>

- [`v0.1.0.json`](crates/app-data/schemas/hooks/v0.1.0.json)
- [`v0.2.0.json`](crates/app-data/schemas/hooks/v0.2.0.json)

</details>

<details>
<summary><code>crates/app-data/schemas/orderClass/</code> &mdash; 3 file(s)</summary>

- [`v0.1.0.json`](crates/app-data/schemas/orderClass/v0.1.0.json)
- [`v0.2.0.json`](crates/app-data/schemas/orderClass/v0.2.0.json)
- [`v0.3.0.json`](crates/app-data/schemas/orderClass/v0.3.0.json)

</details>

<details>
<summary><code>crates/app-data/schemas/partnerFee/</code> &mdash; 2 file(s)</summary>

- [`v0.1.0.json`](crates/app-data/schemas/partnerFee/v0.1.0.json)
- [`v1.0.0.json`](crates/app-data/schemas/partnerFee/v1.0.0.json)

</details>

<details>
<summary><code>crates/app-data/schemas/quote/</code> &mdash; 5 file(s)</summary>

- [`v0.1.0.json`](crates/app-data/schemas/quote/v0.1.0.json)
- [`v0.2.0.json`](crates/app-data/schemas/quote/v0.2.0.json)
- [`v0.3.0.json`](crates/app-data/schemas/quote/v0.3.0.json)
- [`v1.0.0.json`](crates/app-data/schemas/quote/v1.0.0.json)
- [`v1.1.0.json`](crates/app-data/schemas/quote/v1.1.0.json)

</details>

<details>
<summary><code>crates/app-data/schemas/referrer/</code> &mdash; 3 file(s)</summary>

- [`v0.1.0.json`](crates/app-data/schemas/referrer/v0.1.0.json)
- [`v0.2.0.json`](crates/app-data/schemas/referrer/v0.2.0.json)
- [`v1.0.0.json`](crates/app-data/schemas/referrer/v1.0.0.json)

</details>

<details>
<summary><code>crates/app-data/schemas/replacedOrder/</code> &mdash; 1 file(s)</summary>

- [`v0.1.0.json`](crates/app-data/schemas/replacedOrder/v0.1.0.json)

</details>

<details>
<summary><code>crates/app-data/schemas/signer/</code> &mdash; 1 file(s)</summary>

- [`v0.1.0.json`](crates/app-data/schemas/signer/v0.1.0.json)

</details>

<details>
<summary><code>crates/app-data/schemas/userConsents/</code> &mdash; 1 file(s)</summary>

- [`v0.1.0.json`](crates/app-data/schemas/userConsents/v0.1.0.json)

</details>

<details>
<summary><code>crates/app-data/schemas/utm/</code> &mdash; 3 file(s)</summary>

- [`v0.1.0.json`](crates/app-data/schemas/utm/v0.1.0.json)
- [`v0.2.0.json`](crates/app-data/schemas/utm/v0.2.0.json)
- [`v0.3.0.json`](crates/app-data/schemas/utm/v0.3.0.json)

</details>

<details>
<summary><code>crates/app-data/schemas/widget/</code> &mdash; 1 file(s)</summary>

- [`v0.1.0.json`](crates/app-data/schemas/widget/v0.1.0.json)

</details>

<details>
<summary><code>crates/app-data/schemas/wrappers/</code> &mdash; 2 file(s)</summary>

- [`v0.1.0.json`](crates/app-data/schemas/wrappers/v0.1.0.json)
- [`v0.2.0.json`](crates/app-data/schemas/wrappers/v0.2.0.json)

</details>

<details>
<summary><code>crates/app-data/src/</code> &mdash; 6 file(s)</summary>

- [`cid.rs`](crates/app-data/src/cid.rs)
- [`errors.rs`](crates/app-data/src/errors.rs)
- [`fetch.rs`](crates/app-data/src/fetch.rs)
- [`info.rs`](crates/app-data/src/info.rs)
- [`lib.rs`](crates/app-data/src/lib.rs)
- [`schema.rs`](crates/app-data/src/schema.rs)

</details>

<details>
<summary><code>crates/app-data/src/metadata/</code> &mdash; 3 file(s)</summary>

- [`flashloan.rs`](crates/app-data/src/metadata/flashloan.rs)
- [`hooks.rs`](crates/app-data/src/metadata/hooks.rs)
- [`mod.rs`](crates/app-data/src/metadata/mod.rs)

</details>

<details>
<summary><code>crates/app-data/src/types/</code> &mdash; 6 file(s)</summary>

- [`doc.rs`](crates/app-data/src/types/doc.rs)
- [`ipfs.rs`](crates/app-data/src/types/ipfs.rs)
- [`mod.rs`](crates/app-data/src/types/mod.rs)
- [`params.rs`](crates/app-data/src/types/params.rs)
- [`partner_fee.rs`](crates/app-data/src/types/partner_fee.rs)
- [`validation.rs`](crates/app-data/src/types/validation.rs)

</details>

<details>
<summary><code>crates/app-data/tests/</code> &mdash; 18 file(s)</summary>

- [`app_data_info_contract.rs`](crates/app-data/tests/app_data_info_contract.rs)
- [`canonical_json_contract.rs`](crates/app-data/tests/canonical_json_contract.rs)
- [`cid_contract.rs`](crates/app-data/tests/cid_contract.rs)
- [`error_contract.rs`](crates/app-data/tests/error_contract.rs)
- [`error_variant_shape.rs`](crates/app-data/tests/error_variant_shape.rs)
- [`fetch_contract.rs`](crates/app-data/tests/fetch_contract.rs)
- [`flashloan_contract.rs`](crates/app-data/tests/flashloan_contract.rs)
- [`hooks_contract.rs`](crates/app-data/tests/hooks_contract.rs)
- [`ipfs_config_redaction_contract.rs`](crates/app-data/tests/ipfs_config_redaction_contract.rs)
- [`json_recursion_contract.rs`](crates/app-data/tests/json_recursion_contract.rs)
- [`metadata_signer_contract.rs`](crates/app-data/tests/metadata_signer_contract.rs)
- [`parity_contract.rs`](crates/app-data/tests/parity_contract.rs)
- [`partner_fee_contract.rs`](crates/app-data/tests/partner_fee_contract.rs)
- [`property_contract.rs`](crates/app-data/tests/property_contract.rs)
- [`schema_contract.rs`](crates/app-data/tests/schema_contract.rs)
- [`schema_regression_matrix.rs`](crates/app-data/tests/schema_regression_matrix.rs)
- [`v0_cid_is_out_of_scope.rs`](crates/app-data/tests/v0_cid_is_out_of_scope.rs)
- [`validated_shape_contract.rs`](crates/app-data/tests/validated_shape_contract.rs)

</details>

<details>
<summary><code>crates/app-data/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/app-data/tests/common/mod.rs)

</details>

<details>
<summary><code>crates/app-data/tests/proptest-regressions/</code> &mdash; 1 file(s)</summary>

- [`property_contract.txt`](crates/app-data/tests/proptest-regressions/property_contract.txt)

</details>

<details>
<summary><code>crates/app-data/tests/ui/</code> &mdash; 2 file(s)</summary>

- [`partner_fee_bps_width_witness.rs`](crates/app-data/tests/ui/partner_fee_bps_width_witness.rs)
- [`partner_fee_bps_width_witness.stderr`](crates/app-data/tests/ui/partner_fee_bps_width_witness.stderr)

</details>

<details>
<summary><code>crates/browser-wallet/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/browser-wallet/Cargo.toml)
- [`README.md`](crates/browser-wallet/README.md)

</details>

<details>
<summary><code>crates/browser-wallet/src/</code> &mdash; 6 file(s)</summary>

- [`error.rs`](crates/browser-wallet/src/error.rs)
- [`events.rs`](crates/browser-wallet/src/events.rs)
- [`js.rs`](crates/browser-wallet/src/js.rs)
- [`lib.rs`](crates/browser-wallet/src/lib.rs)
- [`mock.rs`](crates/browser-wallet/src/mock.rs)
- [`signer.rs`](crates/browser-wallet/src/signer.rs)

</details>

<details>
<summary><code>crates/browser-wallet/src/provider/</code> &mdash; 6 file(s)</summary>

- [`builder.rs`](crates/browser-wallet/src/provider/builder.rs)
- [`mod.rs`](crates/browser-wallet/src/provider/mod.rs)
- [`origin.rs`](crates/browser-wallet/src/provider/origin.rs)
- [`provider_impl.rs`](crates/browser-wallet/src/provider/provider_impl.rs)
- [`signing_provider_impl.rs`](crates/browser-wallet/src/provider/signing_provider_impl.rs)
- [`transport.rs`](crates/browser-wallet/src/provider/transport.rs)

</details>

<details>
<summary><code>crates/browser-wallet/src/wallet/</code> &mdash; 5 file(s)</summary>

- [`chain_mgmt.rs`](crates/browser-wallet/src/wallet/chain_mgmt.rs)
- [`chain.rs`](crates/browser-wallet/src/wallet/chain.rs)
- [`detect.rs`](crates/browser-wallet/src/wallet/detect.rs)
- [`discovery.rs`](crates/browser-wallet/src/wallet/discovery.rs)
- [`mod.rs`](crates/browser-wallet/src/wallet/mod.rs)

</details>

<details>
<summary><code>crates/browser-wallet/tests/</code> &mdash; 10 file(s)</summary>

- [`non_exhaustive_type_contract.rs`](crates/browser-wallet/tests/non_exhaustive_type_contract.rs)
- [`origin_contract.rs`](crates/browser-wallet/tests/origin_contract.rs)
- [`provider_contract.rs`](crates/browser-wallet/tests/provider_contract.rs)
- [`signer_contract.rs`](crates/browser-wallet/tests/signer_contract.rs)
- [`signer_error_trait_contract.rs`](crates/browser-wallet/tests/signer_error_trait_contract.rs)
- [`signing_provider_contract.rs`](crates/browser-wallet/tests/signing_provider_contract.rs)
- [`state_machine_contract.rs`](crates/browser-wallet/tests/state_machine_contract.rs)
- [`transaction_receipt_parsing.rs`](crates/browser-wallet/tests/transaction_receipt_parsing.rs)
- [`wallet_contract.rs`](crates/browser-wallet/tests/wallet_contract.rs)
- [`wasm_bridge_contract.rs`](crates/browser-wallet/tests/wasm_bridge_contract.rs)

</details>

<details>
<summary><code>crates/composable/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/composable/Cargo.toml)
- [`README.md`](crates/composable/README.md)

</details>

<details>
<summary><code>crates/composable/tests/</code> &mdash; 4 file(s)</summary>

- [`good_after_time_contract.rs`](crates/composable/tests/good_after_time_contract.rs)
- [`perpetual_stable_swap_contract.rs`](crates/composable/tests/perpetual_stable_swap_contract.rs)
- [`stop_loss_contract.rs`](crates/composable/tests/stop_loss_contract.rs)
- [`trade_above_threshold_contract.rs`](crates/composable/tests/trade_above_threshold_contract.rs)

</details>

<details>
<summary><code>crates/composable/tests/fixtures/</code> &mdash; 2 file(s)</summary>

- [`eip1271_blob_shape_a.json`](crates/composable/tests/fixtures/eip1271_blob_shape_a.json)
- [`eip1271_blob_shape_b.json`](crates/composable/tests/fixtures/eip1271_blob_shape_b.json)

</details>

<details>
<summary><code>crates/contracts/</code> &mdash; 6 file(s)</summary>

- [`build.rs`](crates/contracts/build.rs)
- [`Cargo.toml`](crates/contracts/Cargo.toml)
- [`deployment-coverage.yaml`](crates/contracts/deployment-coverage.yaml)
- [`deployment-provenance.yaml`](crates/contracts/deployment-provenance.yaml)
- [`README.md`](crates/contracts/README.md)
- [`registry.toml`](crates/contracts/registry.toml)

</details>

<details>
<summary><code>crates/contracts/abi/composable-cow/</code> &mdash; 3 file(s)</summary>

- [`BaseConditionalOrder.sol`](crates/contracts/abi/composable-cow/BaseConditionalOrder.sol)
- [`ComposableCoW.sol`](crates/contracts/abi/composable-cow/ComposableCoW.sol)
- [`ERC1271Forwarder.sol`](crates/contracts/abi/composable-cow/ERC1271Forwarder.sol)

</details>

<details>
<summary><code>crates/contracts/abi/composable-cow/extensible/</code> &mdash; 1 file(s)</summary>

- [`ExtensibleFallbackHandler.sol`](crates/contracts/abi/composable-cow/extensible/ExtensibleFallbackHandler.sol)

</details>

<details>
<summary><code>crates/contracts/abi/composable-cow/interfaces/</code> &mdash; 3 file(s)</summary>

- [`IConditionalOrder.sol`](crates/contracts/abi/composable-cow/interfaces/IConditionalOrder.sol)
- [`ISwapGuard.sol`](crates/contracts/abi/composable-cow/interfaces/ISwapGuard.sol)
- [`IValueFactory.sol`](crates/contracts/abi/composable-cow/interfaces/IValueFactory.sol)

</details>

<details>
<summary><code>crates/contracts/abi/composable-cow/out/</code> &mdash; 7 file(s)</summary>

- [`ComposableCoW.json`](crates/contracts/abi/composable-cow/out/ComposableCoW.json)
- [`ExtensibleFallbackHandler.json`](crates/contracts/abi/composable-cow/out/ExtensibleFallbackHandler.json)
- [`GoodAfterTime.json`](crates/contracts/abi/composable-cow/out/GoodAfterTime.json)
- [`PerpetualStableSwap.json`](crates/contracts/abi/composable-cow/out/PerpetualStableSwap.json)
- [`StopLoss.json`](crates/contracts/abi/composable-cow/out/StopLoss.json)
- [`TradeAboveThreshold.json`](crates/contracts/abi/composable-cow/out/TradeAboveThreshold.json)
- [`TWAP.json`](crates/contracts/abi/composable-cow/out/TWAP.json)

</details>

<details>
<summary><code>crates/contracts/abi/composable-cow/types/</code> &mdash; 7 file(s)</summary>

- [`GoodAfterTime.sol`](crates/contracts/abi/composable-cow/types/GoodAfterTime.sol)
- [`PerpetualStableSwap.sol`](crates/contracts/abi/composable-cow/types/PerpetualStableSwap.sol)
- [`StopLoss.sol`](crates/contracts/abi/composable-cow/types/StopLoss.sol)
- [`TradeAboveThreshold.sol`](crates/contracts/abi/composable-cow/types/TradeAboveThreshold.sol)
- [`TWAP.sol`](crates/contracts/abi/composable-cow/types/TWAP.sol)
- [`TWAPOrder.sol`](crates/contracts/abi/composable-cow/types/TWAPOrder.sol)
- [`TWAPOrderMathLib.sol`](crates/contracts/abi/composable-cow/types/TWAPOrderMathLib.sol)

</details>

<details>
<summary><code>crates/contracts/abi/composable-cow/value_factories/</code> &mdash; 1 file(s)</summary>

- [`CurrentBlockTimestampFactory.sol`](crates/contracts/abi/composable-cow/value_factories/CurrentBlockTimestampFactory.sol)

</details>

<details>
<summary><code>crates/contracts/abi/composable-cow/vendored/</code> &mdash; 1 file(s)</summary>

- [`CoWSettlement.sol`](crates/contracts/abi/composable-cow/vendored/CoWSettlement.sol)

</details>

<details>
<summary><code>crates/contracts/abi/cow-shed/</code> &mdash; 14 file(s)</summary>

- [`COWShed.sol`](crates/contracts/abi/cow-shed/COWShed.sol)
- [`COWShedFactory.sol`](crates/contracts/abi/cow-shed/COWShedFactory.sol)
- [`COWShedForComposableCoW.sol`](crates/contracts/abi/cow-shed/COWShedForComposableCoW.sol)
- [`COWShedProxy.sol`](crates/contracts/abi/cow-shed/COWShedProxy.sol)
- [`COWShedStorage.sol`](crates/contracts/abi/cow-shed/COWShedStorage.sol)
- [`ERC1271Forwarder.sol`](crates/contracts/abi/cow-shed/ERC1271Forwarder.sol)
- [`IComposableCow.sol`](crates/contracts/abi/cow-shed/IComposableCow.sol)
- [`ICOWAuthHook.sol`](crates/contracts/abi/cow-shed/ICOWAuthHook.sol)
- [`IERC1271.sol`](crates/contracts/abi/cow-shed/IERC1271.sol)
- [`IPreSignStorage.sol`](crates/contracts/abi/cow-shed/IPreSignStorage.sol)
- [`LibAuthenticatedHooks.sol`](crates/contracts/abi/cow-shed/LibAuthenticatedHooks.sol)
- [`LibCowOrder.sol`](crates/contracts/abi/cow-shed/LibCowOrder.sol)
- [`PreSignStateStorage.sol`](crates/contracts/abi/cow-shed/PreSignStateStorage.sol)
- [`version-call-results.json`](crates/contracts/abi/cow-shed/version-call-results.json)

</details>

<details>
<summary><code>crates/contracts/abi/cow-shed/proxy-creation-code/</code> &mdash; 4 file(s)</summary>

- [`v1.0.0.bin`](crates/contracts/abi/cow-shed/proxy-creation-code/v1.0.0.bin)
- [`v1.0.0.bin.sha256`](crates/contracts/abi/cow-shed/proxy-creation-code/v1.0.0.bin.sha256)
- [`v1.0.1.bin`](crates/contracts/abi/cow-shed/proxy-creation-code/v1.0.1.bin)
- [`v1.0.1.bin.sha256`](crates/contracts/abi/cow-shed/proxy-creation-code/v1.0.1.bin.sha256)

</details>

<details>
<summary><code>crates/contracts/abi/eip1967/</code> &mdash; 1 file(s)</summary>

- [`GPv2EIP1967.sol`](crates/contracts/abi/eip1967/GPv2EIP1967.sol)

</details>

<details>
<summary><code>crates/contracts/abi/erc20/</code> &mdash; 1 file(s)</summary>

- [`IERC20.sol`](crates/contracts/abi/erc20/IERC20.sol)

</details>

<details>
<summary><code>crates/contracts/abi/eth-flow/</code> &mdash; 4 file(s)</summary>

- [`CoWSwapEthFlow.sol`](crates/contracts/abi/eth-flow/CoWSwapEthFlow.sol)
- [`CoWSwapOnchainOrders.sol`](crates/contracts/abi/eth-flow/CoWSwapOnchainOrders.sol)
- [`EthFlowOrder.sol`](crates/contracts/abi/eth-flow/EthFlowOrder.sol)
- [`ICoWSwapOnchainOrders.sol`](crates/contracts/abi/eth-flow/ICoWSwapOnchainOrders.sol)

</details>

<details>
<summary><code>crates/contracts/abi/settlement/</code> &mdash; 3 file(s)</summary>

- [`GPv2Interaction.sol`](crates/contracts/abi/settlement/GPv2Interaction.sol)
- [`GPv2Settlement.sol`](crates/contracts/abi/settlement/GPv2Settlement.sol)
- [`GPv2Trade.sol`](crates/contracts/abi/settlement/GPv2Trade.sol)

</details>

<details>
<summary><code>crates/contracts/abi/vault-relayer/</code> &mdash; 1 file(s)</summary>

- [`GPv2VaultRelayer.sol`](crates/contracts/abi/vault-relayer/GPv2VaultRelayer.sol)

</details>

<details>
<summary><code>crates/contracts/abi/weth/</code> &mdash; 1 file(s)</summary>

- [`IWrappedNativeToken.sol`](crates/contracts/abi/weth/IWrappedNativeToken.sol)

</details>

<details>
<summary><code>crates/contracts/benches/</code> &mdash; 2 file(s)</summary>

- [`order_hashing.rs`](crates/contracts/benches/order_hashing.rs)
- [`uid_packing.rs`](crates/contracts/benches/uid_packing.rs)

</details>

<details>
<summary><code>crates/contracts/src/</code> &mdash; 18 file(s)</summary>

- [`chain_ids.rs`](crates/contracts/src/chain_ids.rs)
- [`deploy.rs`](crates/contracts/src/deploy.rs)
- [`eip1271.rs`](crates/contracts/src/eip1271.rs)
- [`erc20.rs`](crates/contracts/src/erc20.rs)
- [`errors.rs`](crates/contracts/src/errors.rs)
- [`eth_flow.rs`](crates/contracts/src/eth_flow.rs)
- [`hex_field.rs`](crates/contracts/src/hex_field.rs)
- [`interaction.rs`](crates/contracts/src/interaction.rs)
- [`lib.rs`](crates/contracts/src/lib.rs)
- [`onchain_orders.rs`](crates/contracts/src/onchain_orders.rs)
- [`primitives.rs`](crates/contracts/src/primitives.rs)
- [`proxy.rs`](crates/contracts/src/proxy.rs)
- [`reader.rs`](crates/contracts/src/reader.rs)
- [`signature.rs`](crates/contracts/src/signature.rs)
- [`swap.rs`](crates/contracts/src/swap.rs)
- [`vault.rs`](crates/contracts/src/vault.rs)
- [`verify.rs`](crates/contracts/src/verify.rs)
- [`weth.rs`](crates/contracts/src/weth.rs)

</details>

<details>
<summary><code>crates/contracts/src/deployments/</code> &mdash; 7 file(s)</summary>

- [`chain_id.rs`](crates/contracts/src/deployments/chain_id.rs)
- [`contract_id.rs`](crates/contracts/src/deployments/contract_id.rs)
- [`coverage.rs`](crates/contracts/src/deployments/coverage.rs)
- [`env.rs`](crates/contracts/src/deployments/env.rs)
- [`mod.rs`](crates/contracts/src/deployments/mod.rs)
- [`registry.rs`](crates/contracts/src/deployments/registry.rs)
- [`verification.rs`](crates/contracts/src/deployments/verification.rs)

</details>

<details>
<summary><code>crates/contracts/src/order/</code> &mdash; 6 file(s)</summary>

- [`hash.rs`](crates/contracts/src/order/hash.rs)
- [`mod.rs`](crates/contracts/src/order/mod.rs)
- [`sol_cancellations.rs`](crates/contracts/src/order/sol_cancellations.rs)
- [`sol_types.rs`](crates/contracts/src/order/sol_types.rs)
- [`types.rs`](crates/contracts/src/order/types.rs)
- [`uid.rs`](crates/contracts/src/order/uid.rs)

</details>

<details>
<summary><code>crates/contracts/src/settlement/</code> &mdash; 4 file(s)</summary>

- [`codec.rs`](crates/contracts/src/settlement/codec.rs)
- [`encoder.rs`](crates/contracts/src/settlement/encoder.rs)
- [`events.rs`](crates/contracts/src/settlement/events.rs)
- [`mod.rs`](crates/contracts/src/settlement/mod.rs)

</details>

<details>
<summary><code>crates/contracts/tests/</code> &mdash; 39 file(s)</summary>

- [`build_rs_compile_fail.rs`](crates/contracts/tests/build_rs_compile_fail.rs)
- [`composable_chain_coverage_contract.rs`](crates/contracts/tests/composable_chain_coverage_contract.rs)
- [`contract_id_variants_contract.rs`](crates/contracts/tests/contract_id_variants_contract.rs)
- [`custom_error_selector_table_contract.rs`](crates/contracts/tests/custom_error_selector_table_contract.rs)
- [`deployment_contract.rs`](crates/contracts/tests/deployment_contract.rs)
- [`deployment_coverage_contract.rs`](crates/contracts/tests/deployment_coverage_contract.rs)
- [`deployment_provenance_contract.rs`](crates/contracts/tests/deployment_provenance_contract.rs)
- [`erc20.rs`](crates/contracts/tests/erc20.rs)
- [`error_contract.rs`](crates/contracts/tests/error_contract.rs)
- [`error_variant_shape.rs`](crates/contracts/tests/error_variant_shape.rs)
- [`eth_flow_events_contract.rs`](crates/contracts/tests/eth_flow_events_contract.rs)
- [`interaction_contract.rs`](crates/contracts/tests/interaction_contract.rs)
- [`non_exhaustive_dto_contract.rs`](crates/contracts/tests/non_exhaustive_dto_contract.rs)
- [`onchain_orders.rs`](crates/contracts/tests/onchain_orders.rs)
- [`order_contract.rs`](crates/contracts/tests/order_contract.rs)
- [`order_digest_parity_contract.rs`](crates/contracts/tests/order_digest_parity_contract.rs)
- [`parity_contract.rs`](crates/contracts/tests/parity_contract.rs)
- [`property_contract.rs`](crates/contracts/tests/property_contract.rs)
- [`proxy_contract.rs`](crates/contracts/tests/proxy_contract.rs)
- [`proxy_creation_code_sha256_contract.rs`](crates/contracts/tests/proxy_creation_code_sha256_contract.rs)
- [`reader_contract.rs`](crates/contracts/tests/reader_contract.rs)
- [`recoverable_signature_contract.rs`](crates/contracts/tests/recoverable_signature_contract.rs)
- [`registry_capability_rows_contract.rs`](crates/contracts/tests/registry_capability_rows_contract.rs)
- [`registry_environment_scope_contract.rs`](crates/contracts/tests/registry_environment_scope_contract.rs)
- [`registry.rs`](crates/contracts/tests/registry.rs)
- [`schema_v2_rejection.rs`](crates/contracts/tests/schema_v2_rejection.rs)
- [`schema_v2_success.rs`](crates/contracts/tests/schema_v2_success.rs)
- [`selector_parity_composable_contract.rs`](crates/contracts/tests/selector_parity_composable_contract.rs)
- [`selector_parity_cow_shed_contract.rs`](crates/contracts/tests/selector_parity_cow_shed_contract.rs)
- [`settlement_contract.rs`](crates/contracts/tests/settlement_contract.rs)
- [`settlement_events_contract.rs`](crates/contracts/tests/settlement_events_contract.rs)
- [`signature_contract.rs`](crates/contracts/tests/signature_contract.rs)
- [`swap_contract.rs`](crates/contracts/tests/swap_contract.rs)
- [`trybuild_schema_v2.rs`](crates/contracts/tests/trybuild_schema_v2.rs)
- [`ui.rs`](crates/contracts/tests/ui.rs)
- [`v_normalization_contract.rs`](crates/contracts/tests/v_normalization_contract.rs)
- [`vault_contract.rs`](crates/contracts/tests/vault_contract.rs)
- [`verify_telemetry_contract.rs`](crates/contracts/tests/verify_telemetry_contract.rs)
- [`weth.rs`](crates/contracts/tests/weth.rs)

</details>

<details>
<summary><code>crates/contracts/tests/build_rs_compile_fail/</code> &mdash; 6 file(s)</summary>

- [`bad_schema_version.toml`](crates/contracts/tests/build_rs_compile_fail/bad_schema_version.toml)
- [`duplicate_entry.toml`](crates/contracts/tests/build_rs_compile_fail/duplicate_entry.toml)
- [`invalid_address.toml`](crates/contracts/tests/build_rs_compile_fail/invalid_address.toml)
- [`malformed_syntax.toml`](crates/contracts/tests/build_rs_compile_fail/malformed_syntax.toml)
- [`unknown_contract_id.toml`](crates/contracts/tests/build_rs_compile_fail/unknown_contract_id.toml)
- [`unsupported_chain.toml`](crates/contracts/tests/build_rs_compile_fail/unsupported_chain.toml)

</details>

<details>
<summary><code>crates/contracts/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/contracts/tests/common/mod.rs)

</details>

<details>
<summary><code>crates/contracts/tests/fixtures/</code> &mdash; 6 file(s)</summary>

- [`composable_canonical_selectors.json`](crates/contracts/tests/fixtures/composable_canonical_selectors.json)
- [`cow_shed_canonical_selectors.json`](crates/contracts/tests/fixtures/cow_shed_canonical_selectors.json)
- [`deployment-provenance-happy.yaml`](crates/contracts/tests/fixtures/deployment-provenance-happy.yaml)
- [`deployment-provenance-missing-row.yaml`](crates/contracts/tests/fixtures/deployment-provenance-missing-row.yaml)
- [`deployment-provenance-skipped.yaml`](crates/contracts/tests/fixtures/deployment-provenance-skipped.yaml)
- [`domain_separator_parity.json`](crates/contracts/tests/fixtures/domain_separator_parity.json)

</details>

<details>
<summary><code>crates/contracts/tests/fixtures/schema_v2_rejection/</code> &mdash; 5 file(s)</summary>

- [`capability_under_prod.toml`](crates/contracts/tests/fixtures/schema_v2_rejection/capability_under_prod.toml)
- [`duplicate_registry_key.toml`](crates/contracts/tests/fixtures/schema_v2_rejection/duplicate_registry_key.toml)
- [`gpv2_environment_agnostic.toml`](crates/contracts/tests/fixtures/schema_v2_rejection/gpv2_environment_agnostic.toml)
- [`unsupported_deployment_chain.toml`](crates/contracts/tests/fixtures/schema_v2_rejection/unsupported_deployment_chain.toml)
- [`unsupported_schema_version.toml`](crates/contracts/tests/fixtures/schema_v2_rejection/unsupported_schema_version.toml)

</details>

<details>
<summary><code>crates/contracts/tests/fixtures/schema_v2_success/</code> &mdash; 3 file(s)</summary>

- [`env_specific_gpv2.toml`](crates/contracts/tests/fixtures/schema_v2_success/env_specific_gpv2.toml)
- [`environment_agnostic_composable.toml`](crates/contracts/tests/fixtures/schema_v2_success/environment_agnostic_composable.toml)
- [`mixed_contract_families.toml`](crates/contracts/tests/fixtures/schema_v2_success/mixed_contract_families.toml)

</details>

<details>
<summary><code>crates/contracts/tests/proptest-regressions/</code> &mdash; 1 file(s)</summary>

- [`property_contract.txt`](crates/contracts/tests/proptest-regressions/property_contract.txt)

</details>

<details>
<summary><code>crates/contracts/tests/ui/</code> &mdash; 4 file(s)</summary>

- [`non_exhaustive_external_match.rs`](crates/contracts/tests/ui/non_exhaustive_external_match.rs)
- [`non_exhaustive_external_match.stderr`](crates/contracts/tests/ui/non_exhaustive_external_match.stderr)
- [`typestate_marker_sealing.rs`](crates/contracts/tests/ui/typestate_marker_sealing.rs)
- [`typestate_marker_sealing.stderr`](crates/contracts/tests/ui/typestate_marker_sealing.stderr)

</details>

<details>
<summary><code>crates/core/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/core/Cargo.toml)
- [`README.md`](crates/core/README.md)

</details>

<details>
<summary><code>crates/core/src/</code> &mdash; 5 file(s)</summary>

- [`cancellation.rs`](crates/core/src/cancellation.rs)
- [`errors.rs`](crates/core/src/errors.rs)
- [`lib.rs`](crates/core/src/lib.rs)
- [`prelude.rs`](crates/core/src/prelude.rs)
- [`validation.rs`](crates/core/src/validation.rs)

</details>

<details>
<summary><code>crates/core/src/config/</code> &mdash; 6 file(s)</summary>

- [`chains.rs`](crates/core/src/config/chains.rs)
- [`env.rs`](crates/core/src/config/env.rs)
- [`hosts.rs`](crates/core/src/config/hosts.rs)
- [`http.rs`](crates/core/src/config/http.rs)
- [`mod.rs`](crates/core/src/config/mod.rs)
- [`protocol.rs`](crates/core/src/config/protocol.rs)

</details>

<details>
<summary><code>crates/core/src/redaction/</code> &mdash; 3 file(s)</summary>

- [`body.rs`](crates/core/src/redaction/body.rs)
- [`mod.rs`](crates/core/src/redaction/mod.rs)
- [`wrappers.rs`](crates/core/src/redaction/wrappers.rs)

</details>

<details>
<summary><code>crates/core/src/traits/</code> &mdash; 8 file(s)</summary>

- [`contract.rs`](crates/core/src/traits/contract.rs)
- [`log_provider.rs`](crates/core/src/traits/log_provider.rs)
- [`mod.rs`](crates/core/src/traits/mod.rs)
- [`provider.rs`](crates/core/src/traits/provider.rs)
- [`signer.rs`](crates/core/src/traits/signer.rs)
- [`transaction.rs`](crates/core/src/traits/transaction.rs)
- [`transport.rs`](crates/core/src/traits/transport.rs)
- [`typed_data.rs`](crates/core/src/traits/typed_data.rs)

</details>

<details>
<summary><code>crates/core/src/transport/</code> &mdash; 4 file(s)</summary>

- [`error.rs`](crates/core/src/transport/error.rs)
- [`http.rs`](crates/core/src/transport/http.rs)
- [`mod.rs`](crates/core/src/transport/mod.rs)
- [`reqwest.rs`](crates/core/src/transport/reqwest.rs)

</details>

<details>
<summary><code>crates/core/src/types/</code> &mdash; 8 file(s)</summary>

- [`amount.rs`](crates/core/src/types/amount.rs)
- [`app_code.rs`](crates/core/src/types/app_code.rs)
- [`identity.rs`](crates/core/src/types/identity.rs)
- [`logs.rs`](crates/core/src/types/logs.rs)
- [`mod.rs`](crates/core/src/types/mod.rs)
- [`order.rs`](crates/core/src/types/order.rs)
- [`quote.rs`](crates/core/src/types/quote.rs)
- [`validity.rs`](crates/core/src/types/validity.rs)

</details>

<details>
<summary><code>crates/core/tests/</code> &mdash; 15 file(s)</summary>

- [`amount_arithmetic_ui.rs`](crates/core/tests/amount_arithmetic_ui.rs)
- [`cancellation_contract.rs`](crates/core/tests/cancellation_contract.rs)
- [`cancellation_coverage_validator.rs`](crates/core/tests/cancellation_coverage_validator.rs)
- [`cid_parity_contract.rs`](crates/core/tests/cid_parity_contract.rs)
- [`config_contract.rs`](crates/core/tests/config_contract.rs)
- [`property_contract.rs`](crates/core/tests/property_contract.rs)
- [`provider_capability_split_contract.rs`](crates/core/tests/provider_capability_split_contract.rs)
- [`redaction_contract.rs`](crates/core/tests/redaction_contract.rs)
- [`token_balance_parity.rs`](crates/core/tests/token_balance_parity.rs)
- [`token_balance_ui.rs`](crates/core/tests/token_balance_ui.rs)
- [`trait_evolution_contract.rs`](crates/core/tests/trait_evolution_contract.rs)
- [`traits_contract.rs`](crates/core/tests/traits_contract.rs)
- [`transport_contract.rs`](crates/core/tests/transport_contract.rs)
- [`types_contract.rs`](crates/core/tests/types_contract.rs)
- [`wire_format_preservation_contract.rs`](crates/core/tests/wire_format_preservation_contract.rs)

</details>

<details>
<summary><code>crates/core/tests/fixtures/transport/</code> &mdash; 3 file(s)</summary>

- [`delete_order_ok.txt`](crates/core/tests/fixtures/transport/delete_order_ok.txt)
- [`get_orders_ok.json`](crates/core/tests/fixtures/transport/get_orders_ok.json)
- [`post_quote_ok.json`](crates/core/tests/fixtures/transport/post_quote_ok.json)

</details>

<details>
<summary><code>crates/core/tests/proptest-regressions/</code> &mdash; 1 file(s)</summary>

- [`property_contract.txt`](crates/core/tests/proptest-regressions/property_contract.txt)

</details>

<details>
<summary><code>crates/core/tests/ui/</code> &mdash; 4 file(s)</summary>

- [`amount_arithmetic_operators_removed.rs`](crates/core/tests/ui/amount_arithmetic_operators_removed.rs)
- [`amount_arithmetic_operators_removed.stderr`](crates/core/tests/ui/amount_arithmetic_operators_removed.stderr)
- [`token_balance_split_cross_side.rs`](crates/core/tests/ui/token_balance_split_cross_side.rs)
- [`token_balance_split_cross_side.stderr`](crates/core/tests/ui/token_balance_split_cross_side.stderr)

</details>

<details>
<summary><code>crates/cow-shed/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/cow-shed/Cargo.toml)
- [`README.md`](crates/cow-shed/README.md)

</details>

<details>
<summary><code>crates/cow-shed/src/</code> &mdash; 3 file(s)</summary>

- [`errors.rs`](crates/cow-shed/src/errors.rs)
- [`lib.rs`](crates/cow-shed/src/lib.rs)
- [`version.rs`](crates/cow-shed/src/version.rs)

</details>

<details>
<summary><code>crates/cow-shed/src/address/</code> &mdash; 2 file(s)</summary>

- [`mod.rs`](crates/cow-shed/src/address/mod.rs)
- [`proxy_code.rs`](crates/cow-shed/src/address/proxy_code.rs)

</details>

<details>
<summary><code>crates/cow-shed/src/bindings/</code> &mdash; 4 file(s)</summary>

- [`factory.rs`](crates/cow-shed/src/bindings/factory.rs)
- [`mod.rs`](crates/cow-shed/src/bindings/mod.rs)
- [`shed_for_composable.rs`](crates/cow-shed/src/bindings/shed_for_composable.rs)
- [`shed.rs`](crates/cow-shed/src/bindings/shed.rs)

</details>

<details>
<summary><code>crates/cow-shed/src/calls/</code> &mdash; 3 file(s)</summary>

- [`execute_hooks.rs`](crates/cow-shed/src/calls/execute_hooks.rs)
- [`mod.rs`](crates/cow-shed/src/calls/mod.rs)
- [`pre_sign.rs`](crates/cow-shed/src/calls/pre_sign.rs)

</details>

<details>
<summary><code>crates/cow-shed/src/eip712/</code> &mdash; 4 file(s)</summary>

- [`domain.rs`](crates/cow-shed/src/eip712/domain.rs)
- [`hash.rs`](crates/cow-shed/src/eip712/hash.rs)
- [`mod.rs`](crates/cow-shed/src/eip712/mod.rs)
- [`sol_types.rs`](crates/cow-shed/src/eip712/sol_types.rs)

</details>

<details>
<summary><code>crates/cow-shed/src/types/</code> &mdash; 4 file(s)</summary>

- [`call.rs`](crates/cow-shed/src/types/call.rs)
- [`deadline.rs`](crates/cow-shed/src/types/deadline.rs)
- [`mod.rs`](crates/cow-shed/src/types/mod.rs)
- [`nonce.rs`](crates/cow-shed/src/types/nonce.rs)

</details>

<details>
<summary><code>crates/cow-shed/tests/</code> &mdash; 10 file(s)</summary>

- [`calldata_parity_contract.rs`](crates/cow-shed/tests/calldata_parity_contract.rs)
- [`domain_separator_parity_contract.rs`](crates/cow-shed/tests/domain_separator_parity_contract.rs)
- [`eip712_message_hash_parity_contract.rs`](crates/cow-shed/tests/eip712_message_hash_parity_contract.rs)
- [`eip712_type_hash_parity_contract.rs`](crates/cow-shed/tests/eip712_type_hash_parity_contract.rs)
- [`eoa_signature_byte_order_contract.rs`](crates/cow-shed/tests/eoa_signature_byte_order_contract.rs)
- [`init_code_derivation_contract.rs`](crates/cow-shed/tests/init_code_derivation_contract.rs)
- [`non_exhaustive_surface_contract.rs`](crates/cow-shed/tests/non_exhaustive_surface_contract.rs)
- [`panic_surface_contract.rs`](crates/cow-shed/tests/panic_surface_contract.rs)
- [`proxy_address_parity_contract.rs`](crates/cow-shed/tests/proxy_address_parity_contract.rs)
- [`selector_parity_contract.rs`](crates/cow-shed/tests/selector_parity_contract.rs)

</details>

<details>
<summary><code>crates/orderbook/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/orderbook/Cargo.toml)
- [`README.md`](crates/orderbook/README.md)

</details>

<details>
<summary><code>crates/orderbook/benches/</code> &mdash; 1 file(s)</summary>

- [`quote_cost.rs`](crates/orderbook/benches/quote_cost.rs)

</details>

<details>
<summary><code>crates/orderbook/examples/</code> &mdash; 1 file(s)</summary>

- [`paginated_orders_fetch.rs`](crates/orderbook/examples/paginated_orders_fetch.rs)

</details>

<details>
<summary><code>crates/orderbook/src/</code> &mdash; 7 file(s)</summary>

- [`api.rs`](crates/orderbook/src/api.rs)
- [`builder.rs`](crates/orderbook/src/builder.rs)
- [`error.rs`](crates/orderbook/src/error.rs)
- [`lib.rs`](crates/orderbook/src/lib.rs)
- [`rejection.rs`](crates/orderbook/src/rejection.rs)
- [`request.rs`](crates/orderbook/src/request.rs)
- [`transform.rs`](crates/orderbook/src/transform.rs)

</details>

<details>
<summary><code>crates/orderbook/src/types/</code> &mdash; 8 file(s)</summary>

- [`app_data.rs`](crates/orderbook/src/types/app_data.rs)
- [`auction.rs`](crates/orderbook/src/types/auction.rs)
- [`enums.rs`](crates/orderbook/src/types/enums.rs)
- [`lists.rs`](crates/orderbook/src/types/lists.rs)
- [`mod.rs`](crates/orderbook/src/types/mod.rs)
- [`order.rs`](crates/orderbook/src/types/order.rs)
- [`prices.rs`](crates/orderbook/src/types/prices.rs)
- [`quote.rs`](crates/orderbook/src/types/quote.rs)

</details>

<details>
<summary><code>crates/orderbook/tests/</code> &mdash; 19 file(s)</summary>

- [`api_contract.rs`](crates/orderbook/tests/api_contract.rs)
- [`builder_contract.rs`](crates/orderbook/tests/builder_contract.rs)
- [`cancellation_composition_contract.rs`](crates/orderbook/tests/cancellation_composition_contract.rs)
- [`delay_for_zero_contract.rs`](crates/orderbook/tests/delay_for_zero_contract.rs)
- [`error_variant_shape.rs`](crates/orderbook/tests/error_variant_shape.rs)
- [`fee_amount_is_not_a_public_builder_setter.rs`](crates/orderbook/tests/fee_amount_is_not_a_public_builder_setter.rs)
- [`host_policy_contract.rs`](crates/orderbook/tests/host_policy_contract.rs)
- [`invariant_contract.rs`](crates/orderbook/tests/invariant_contract.rs)
- [`openapi_dto_coverage.rs`](crates/orderbook/tests/openapi_dto_coverage.rs)
- [`order_creation_fee_deserialize.rs`](crates/orderbook/tests/order_creation_fee_deserialize.rs)
- [`parity_contract.rs`](crates/orderbook/tests/parity_contract.rs)
- [`rejection_category_contract.rs`](crates/orderbook/tests/rejection_category_contract.rs)
- [`rejection_contract.rs`](crates/orderbook/tests/rejection_contract.rs)
- [`request_contract.rs`](crates/orderbook/tests/request_contract.rs)
- [`schema_source_contract.rs`](crates/orderbook/tests/schema_source_contract.rs)
- [`signing_scheme_bridge_contract.rs`](crates/orderbook/tests/signing_scheme_bridge_contract.rs)
- [`transform_contract.rs`](crates/orderbook/tests/transform_contract.rs)
- [`types_contract.rs`](crates/orderbook/tests/types_contract.rs)
- [`wire_contract.rs`](crates/orderbook/tests/wire_contract.rs)

</details>

<details>
<summary><code>crates/orderbook/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/orderbook/tests/common/mod.rs)

</details>

<details>
<summary><code>crates/orderbook/tests/proptest-regressions/</code> &mdash; 1 file(s)</summary>

- [`order_creation_fee_deserialize.txt`](crates/orderbook/tests/proptest-regressions/order_creation_fee_deserialize.txt)

</details>

<details>
<summary><code>crates/pure-helpers/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/pure-helpers/Cargo.toml)
- [`README.md`](crates/pure-helpers/README.md)

</details>

<details>
<summary><code>crates/pure-helpers/src/</code> &mdash; 7 file(s)</summary>

- [`app_data.rs`](crates/pure-helpers/src/app_data.rs)
- [`chains.rs`](crates/pure-helpers/src/chains.rs)
- [`dto.rs`](crates/pure-helpers/src/dto.rs)
- [`errors.rs`](crates/pure-helpers/src/errors.rs)
- [`lib.rs`](crates/pure-helpers/src/lib.rs)
- [`signing.rs`](crates/pure-helpers/src/signing.rs)
- [`uid.rs`](crates/pure-helpers/src/uid.rs)

</details>

<details>
<summary><code>crates/pure-helpers/tests/</code> &mdash; 1 file(s)</summary>

- [`no_ffi_imports.rs`](crates/pure-helpers/tests/no_ffi_imports.rs)

</details>

<details>
<summary><code>crates/sdk/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/sdk/Cargo.toml)
- [`README.md`](crates/sdk/README.md)

</details>

<details>
<summary><code>crates/sdk/examples/</code> &mdash; 2 file(s)</summary>

- [`README.md`](crates/sdk/examples/README.md)
- [`wasm_smoke.rs`](crates/sdk/examples/wasm_smoke.rs)

</details>

<details>
<summary><code>crates/sdk/examples/support/</code> &mdash; 1 file(s)</summary>

- [`order_sign_submit_smoke.rs`](crates/sdk/examples/support/order_sign_submit_smoke.rs)

</details>

<details>
<summary><code>crates/sdk/src/</code> &mdash; 2 file(s)</summary>

- [`lib.rs`](crates/sdk/src/lib.rs)
- [`prelude.rs`](crates/sdk/src/prelude.rs)

</details>

<details>
<summary><code>crates/sdk/tests/</code> &mdash; 8 file(s)</summary>

- [`cross_fixture_amount_roundtrip.rs`](crates/sdk/tests/cross_fixture_amount_roundtrip.rs)
- [`error_class_contract.rs`](crates/sdk/tests/error_class_contract.rs)
- [`error_redaction_contract.rs`](crates/sdk/tests/error_redaction_contract.rs)
- [`parity_fixture_sort.rs`](crates/sdk/tests/parity_fixture_sort.rs)
- [`public_api_default_features_only.rs`](crates/sdk/tests/public_api_default_features_only.rs)
- [`public_api_with_all_features.rs`](crates/sdk/tests/public_api_with_all_features.rs)
- [`public_api.rs`](crates/sdk/tests/public_api.rs)
- [`ui.rs`](crates/sdk/tests/ui.rs)

</details>

<details>
<summary><code>crates/sdk/tests/fixtures/</code> &mdash; 2 file(s)</summary>

- [`public_api_default_features_only.snap`](crates/sdk/tests/fixtures/public_api_default_features_only.snap)
- [`public_api_with_all_features.snap`](crates/sdk/tests/fixtures/public_api_with_all_features.snap)

</details>

<details>
<summary><code>crates/sdk/tests/ui/</code> &mdash; 1 file(s)</summary>

- [`orderbook_client_reachable_through_trading_re_export.rs`](crates/sdk/tests/ui/orderbook_client_reachable_through_trading_re_export.rs)

</details>

<details>
<summary><code>crates/signing/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/signing/Cargo.toml)
- [`README.md`](crates/signing/README.md)

</details>

<details>
<summary><code>crates/signing/benches/</code> &mdash; 1 file(s)</summary>

- [`typed_data.rs`](crates/signing/benches/typed_data.rs)

</details>

<details>
<summary><code>crates/signing/src/</code> &mdash; 6 file(s)</summary>

- [`cache.rs`](crates/signing/src/cache.rs)
- [`cancellation.rs`](crates/signing/src/cancellation.rs)
- [`domain.rs`](crates/signing/src/domain.rs)
- [`errors.rs`](crates/signing/src/errors.rs)
- [`lib.rs`](crates/signing/src/lib.rs)
- [`order_signing.rs`](crates/signing/src/order_signing.rs)

</details>

<details>
<summary><code>crates/signing/src/eip1271/</code> &mdash; 4 file(s)</summary>

- [`error.rs`](crates/signing/src/eip1271/error.rs)
- [`mod.rs`](crates/signing/src/eip1271/mod.rs)
- [`provider.rs`](crates/signing/src/eip1271/provider.rs)
- [`sol_types.rs`](crates/signing/src/eip1271/sol_types.rs)

</details>

<details>
<summary><code>crates/signing/tests/</code> &mdash; 9 file(s)</summary>

- [`cancellation_contract.rs`](crates/signing/tests/cancellation_contract.rs)
- [`domain_contract.rs`](crates/signing/tests/domain_contract.rs)
- [`eip1271_cache_contract.rs`](crates/signing/tests/eip1271_cache_contract.rs)
- [`eip1271_contract.rs`](crates/signing/tests/eip1271_contract.rs)
- [`order_signing_contract.rs`](crates/signing/tests/order_signing_contract.rs)
- [`parity_contract.rs`](crates/signing/tests/parity_contract.rs)
- [`property_contract.rs`](crates/signing/tests/property_contract.rs)
- [`ui.rs`](crates/signing/tests/ui.rs)
- [`wasm_cache_contract.rs`](crates/signing/tests/wasm_cache_contract.rs)

</details>

<details>
<summary><code>crates/signing/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/signing/tests/common/mod.rs)

</details>

<details>
<summary><code>crates/signing/tests/fixtures/</code> &mdash; 1 file(s)</summary>

- [`domain_separator_parity.json`](crates/signing/tests/fixtures/domain_separator_parity.json)

</details>

<details>
<summary><code>crates/signing/tests/proptest-regressions/</code> &mdash; 1 file(s)</summary>

- [`property_contract.txt`](crates/signing/tests/proptest-regressions/property_contract.txt)

</details>

<details>
<summary><code>crates/signing/tests/ui/</code> &mdash; 2 file(s)</summary>

- [`eip1271_error_match_requires_wildcard.rs`](crates/signing/tests/ui/eip1271_error_match_requires_wildcard.rs)
- [`eip1271_error_match_requires_wildcard.stderr`](crates/signing/tests/ui/eip1271_error_match_requires_wildcard.stderr)

</details>

<details>
<summary><code>crates/subgraph/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/subgraph/Cargo.toml)
- [`README.md`](crates/subgraph/README.md)

</details>

<details>
<summary><code>crates/subgraph/examples/</code> &mdash; 1 file(s)</summary>

- [`typed_query_with_escape_hatch.rs`](crates/subgraph/examples/typed_query_with_escape_hatch.rs)

</details>

<details>
<summary><code>crates/subgraph/src/</code> &mdash; 6 file(s)</summary>

- [`api.rs`](crates/subgraph/src/api.rs)
- [`builder.rs`](crates/subgraph/src/builder.rs)
- [`error.rs`](crates/subgraph/src/error.rs)
- [`lib.rs`](crates/subgraph/src/lib.rs)
- [`queries.rs`](crates/subgraph/src/queries.rs)
- [`types.rs`](crates/subgraph/src/types.rs)

</details>

<details>
<summary><code>crates/subgraph/src/query_documents/</code> &mdash; 3 file(s)</summary>

- [`last_days_volume.graphql`](crates/subgraph/src/query_documents/last_days_volume.graphql)
- [`last_hours_volume.graphql`](crates/subgraph/src/query_documents/last_hours_volume.graphql)
- [`totals.graphql`](crates/subgraph/src/query_documents/totals.graphql)

</details>

<details>
<summary><code>crates/subgraph/tests/</code> &mdash; 12 file(s)</summary>

- [`api_contract.rs`](crates/subgraph/tests/api_contract.rs)
- [`builder_contract.rs`](crates/subgraph/tests/builder_contract.rs)
- [`builder_ui.rs`](crates/subgraph/tests/builder_ui.rs)
- [`cancellation_composition_contract.rs`](crates/subgraph/tests/cancellation_composition_contract.rs)
- [`error_contract.rs`](crates/subgraph/tests/error_contract.rs)
- [`error_redaction_contract.rs`](crates/subgraph/tests/error_redaction_contract.rs)
- [`host_policy_contract.rs`](crates/subgraph/tests/host_policy_contract.rs)
- [`invariant_contract.rs`](crates/subgraph/tests/invariant_contract.rs)
- [`parity_contract.rs`](crates/subgraph/tests/parity_contract.rs)
- [`query_contract.rs`](crates/subgraph/tests/query_contract.rs)
- [`schema_source_contract.rs`](crates/subgraph/tests/schema_source_contract.rs)
- [`types_contract.rs`](crates/subgraph/tests/types_contract.rs)

</details>

<details>
<summary><code>crates/subgraph/tests/schema_evidence/</code> &mdash; 1 file(s)</summary>

- [`schema.graphql`](crates/subgraph/tests/schema_evidence/schema.graphql)

</details>

<details>
<summary><code>crates/subgraph/tests/ui/</code> &mdash; 2 file(s)</summary>

- [`builder_wasm32_missing_transport.rs`](crates/subgraph/tests/ui/builder_wasm32_missing_transport.rs)
- [`builder_wasm32_missing_transport.stderr`](crates/subgraph/tests/ui/builder_wasm32_missing_transport.stderr)

</details>

<details>
<summary><code>crates/trading/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/trading/Cargo.toml)
- [`README.md`](crates/trading/README.md)

</details>

<details>
<summary><code>crates/trading/benches/</code> &mdash; 1 file(s)</summary>

- [`order_build.rs`](crates/trading/benches/order_build.rs)

</details>

<details>
<summary><code>crates/trading/examples/</code> &mdash; 2 file(s)</summary>

- [`signed_order_end_to_end.rs`](crates/trading/examples/signed_order_end_to_end.rs)
- [`typestate_builder_example.rs`](crates/trading/examples/typestate_builder_example.rs)

</details>

<details>
<summary><code>crates/trading/src/</code> &mdash; 11 file(s)</summary>

- [`allowance.rs`](crates/trading/src/allowance.rs)
- [`app_data.rs`](crates/trading/src/app_data.rs)
- [`cancel.rs`](crates/trading/src/cancel.rs)
- [`error.rs`](crates/trading/src/error.rs)
- [`lib.rs`](crates/trading/src/lib.rs)
- [`onchain.rs`](crates/trading/src/onchain.rs)
- [`order.rs`](crates/trading/src/order.rs)
- [`parameters.rs`](crates/trading/src/parameters.rs)
- [`quote.rs`](crates/trading/src/quote.rs)
- [`validation.rs`](crates/trading/src/validation.rs)
- [`wait.rs`](crates/trading/src/wait.rs)

</details>

<details>
<summary><code>crates/trading/src/post/</code> &mdash; 7 file(s)</summary>

- [`from_quote.rs`](crates/trading/src/post/from_quote.rs)
- [`generic.rs`](crates/trading/src/post/generic.rs)
- [`limit.rs`](crates/trading/src/post/limit.rs)
- [`mod.rs`](crates/trading/src/post/mod.rs)
- [`native.rs`](crates/trading/src/post/native.rs)
- [`swap.rs`](crates/trading/src/post/swap.rs)
- [`verify.rs`](crates/trading/src/post/verify.rs)

</details>

<details>
<summary><code>crates/trading/src/sdk/</code> &mdash; 10 file(s)</summary>

- [`allowance.rs`](crates/trading/src/sdk/allowance.rs)
- [`builder.rs`](crates/trading/src/sdk/builder.rs)
- [`cancel.rs`](crates/trading/src/sdk/cancel.rs)
- [`helper_only.rs`](crates/trading/src/sdk/helper_only.rs)
- [`helpers.rs`](crates/trading/src/sdk/helpers.rs)
- [`mod.rs`](crates/trading/src/sdk/mod.rs)
- [`post.rs`](crates/trading/src/sdk/post.rs)
- [`presign.rs`](crates/trading/src/sdk/presign.rs)
- [`query.rs`](crates/trading/src/sdk/query.rs)
- [`quote.rs`](crates/trading/src/sdk/quote.rs)

</details>

<details>
<summary><code>crates/trading/src/slippage/</code> &mdash; 4 file(s)</summary>

- [`amounts.rs`](crates/trading/src/slippage/amounts.rs)
- [`breakdown.rs`](crates/trading/src/slippage/breakdown.rs)
- [`mod.rs`](crates/trading/src/slippage/mod.rs)
- [`policy.rs`](crates/trading/src/slippage/policy.rs)

</details>

<details>
<summary><code>crates/trading/src/types/</code> &mdash; 12 file(s)</summary>

- [`advanced.rs`](crates/trading/src/types/advanced.rs)
- [`allowance.rs`](crates/trading/src/types/allowance.rs)
- [`context.rs`](crates/trading/src/types/context.rs)
- [`eip1271.rs`](crates/trading/src/types/eip1271.rs)
- [`mod.rs`](crates/trading/src/types/mod.rs)
- [`options.rs`](crates/trading/src/types/options.rs)
- [`overrides.rs`](crates/trading/src/types/overrides.rs)
- [`result.rs`](crates/trading/src/types/result.rs)
- [`seams.rs`](crates/trading/src/types/seams.rs)
- [`slippage.rs`](crates/trading/src/types/slippage.rs)
- [`trade.rs`](crates/trading/src/types/trade.rs)
- [`trader.rs`](crates/trading/src/types/trader.rs)

</details>

<details>
<summary><code>crates/trading/tests/</code> &mdash; 22 file(s)</summary>

- [`allowance_contract.rs`](crates/trading/tests/allowance_contract.rs)
- [`app_code_contract.rs`](crates/trading/tests/app_code_contract.rs)
- [`app_data_merge_contract.rs`](crates/trading/tests/app_data_merge_contract.rs)
- [`cancel_contract.rs`](crates/trading/tests/cancel_contract.rs)
- [`cancellation_composition_contract.rs`](crates/trading/tests/cancellation_composition_contract.rs)
- [`error_variant_shape.rs`](crates/trading/tests/error_variant_shape.rs)
- [`invariant_contract.rs`](crates/trading/tests/invariant_contract.rs)
- [`limit_from_quote_contract.rs`](crates/trading/tests/limit_from_quote_contract.rs)
- [`onchain_contract.rs`](crates/trading/tests/onchain_contract.rs)
- [`order_contract.rs`](crates/trading/tests/order_contract.rs)
- [`parameters_contract.rs`](crates/trading/tests/parameters_contract.rs)
- [`parity_contract.rs`](crates/trading/tests/parity_contract.rs)
- [`post_contract.rs`](crates/trading/tests/post_contract.rs)
- [`property_contract.rs`](crates/trading/tests/property_contract.rs)
- [`quote_contract.rs`](crates/trading/tests/quote_contract.rs)
- [`quote_projection_parity.rs`](crates/trading/tests/quote_projection_parity.rs)
- [`sdk_contract.rs`](crates/trading/tests/sdk_contract.rs)
- [`slippage_contract.rs`](crates/trading/tests/slippage_contract.rs)
- [`types_contract.rs`](crates/trading/tests/types_contract.rs)
- [`ui.rs`](crates/trading/tests/ui.rs)
- [`validation_contract.rs`](crates/trading/tests/validation_contract.rs)
- [`wait_helper_contract.rs`](crates/trading/tests/wait_helper_contract.rs)

</details>

<details>
<summary><code>crates/trading/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/trading/tests/common/mod.rs)

</details>

<details>
<summary><code>crates/trading/tests/proptest-regressions/</code> &mdash; 1 file(s)</summary>

- [`property_contract.txt`](crates/trading/tests/proptest-regressions/property_contract.txt)

</details>

<details>
<summary><code>crates/trading/tests/ui/</code> &mdash; 8 file(s)</summary>

- [`client_rejection_external_match_requires_wildcard.rs`](crates/trading/tests/ui/client_rejection_external_match_requires_wildcard.rs)
- [`client_rejection_external_match_requires_wildcard.stderr`](crates/trading/tests/ui/client_rejection_external_match_requires_wildcard.stderr)
- [`helper_only_sdk_no_offchain_cancel.rs`](crates/trading/tests/ui/helper_only_sdk_no_offchain_cancel.rs)
- [`helper_only_sdk_no_offchain_cancel.stderr`](crates/trading/tests/ui/helper_only_sdk_no_offchain_cancel.stderr)
- [`helper_only_sdk_no_quote_methods.rs`](crates/trading/tests/ui/helper_only_sdk_no_quote_methods.rs)
- [`helper_only_sdk_no_quote_methods.stderr`](crates/trading/tests/ui/helper_only_sdk_no_quote_methods.stderr)
- [`trading_sdk_no_free_constructors.rs`](crates/trading/tests/ui/trading_sdk_no_free_constructors.rs)
- [`trading_sdk_no_free_constructors.stderr`](crates/trading/tests/ui/trading_sdk_no_free_constructors.stderr)

</details>

<details>
<summary><code>crates/transport-policy/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/transport-policy/Cargo.toml)
- [`README.md`](crates/transport-policy/README.md)

</details>

<details>
<summary><code>crates/transport-policy/src/</code> &mdash; 10 file(s)</summary>

- [`classify.rs`](crates/transport-policy/src/classify.rs)
- [`jitter.rs`](crates/transport-policy/src/jitter.rs)
- [`lib.rs`](crates/transport-policy/src/lib.rs)
- [`policy.rs`](crates/transport-policy/src/policy.rs)
- [`rate_limit.rs`](crates/transport-policy/src/rate_limit.rs)
- [`retry_after.rs`](crates/transport-policy/src/retry_after.rs)
- [`retry.rs`](crates/transport-policy/src/retry.rs)
- [`runner.rs`](crates/transport-policy/src/runner.rs)
- [`status.rs`](crates/transport-policy/src/status.rs)
- [`time.rs`](crates/transport-policy/src/time.rs)

</details>

<details>
<summary><code>crates/transport-policy/tests/</code> &mdash; 5 file(s)</summary>

- [`classify_contract.rs`](crates/transport-policy/tests/classify_contract.rs)
- [`policy_contract.rs`](crates/transport-policy/tests/policy_contract.rs)
- [`retry_after_contract.proptest-regressions`](crates/transport-policy/tests/retry_after_contract.proptest-regressions)
- [`retry_after_contract.rs`](crates/transport-policy/tests/retry_after_contract.rs)
- [`retry_after_fixture_contract.rs`](crates/transport-policy/tests/retry_after_fixture_contract.rs)

</details>

<details>
<summary><code>crates/transport-wasm/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/transport-wasm/Cargo.toml)
- [`README.md`](crates/transport-wasm/README.md)

</details>

<details>
<summary><code>crates/transport-wasm/src/</code> &mdash; 2 file(s)</summary>

- [`fetch.rs`](crates/transport-wasm/src/fetch.rs)
- [`lib.rs`](crates/transport-wasm/src/lib.rs)

</details>

<details>
<summary><code>crates/transport-wasm/tests/</code> &mdash; 3 file(s)</summary>

- [`fetch_contract.rs`](crates/transport-wasm/tests/fetch_contract.rs)
- [`parity_contract.rs`](crates/transport-wasm/tests/parity_contract.rs)
- [`wasm.rs`](crates/transport-wasm/tests/wasm.rs)

</details>

<details>
<summary><code>crates/transport-wasm/tests/wasm/</code> &mdash; 1 file(s)</summary>

- [`fetch_smoke.rs`](crates/transport-wasm/tests/wasm/fetch_smoke.rs)

</details>

<details>
<summary><code>crates/wasm/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/wasm/Cargo.toml)
- [`README.md`](crates/wasm/README.md)

</details>

<details>
<summary><code>crates/wasm/npm/</code> &mdash; 11 file(s)</summary>

- [`.gitignore`](crates/wasm/npm/.gitignore)
- [`.npmignore`](crates/wasm/npm/.npmignore)
- [`flavours.json`](crates/wasm/npm/flavours.json)
- [`LICENSE`](crates/wasm/npm/LICENSE)
- [`package.json`](crates/wasm/npm/package.json)
- [`package.template.json`](crates/wasm/npm/package.template.json)
- [`pnpm-lock.yaml`](crates/wasm/npm/pnpm-lock.yaml)
- [`README.md`](crates/wasm/npm/README.md)
- [`tsconfig.facade.json`](crates/wasm/npm/tsconfig.facade.json)
- [`tsconfig.json`](crates/wasm/npm/tsconfig.json)
- [`vitest.config.ts`](crates/wasm/npm/vitest.config.ts)

</details>

<details>
<summary><code>crates/wasm/npm/scripts/</code> &mdash; 10 file(s)</summary>

- [`build.sh`](crates/wasm/npm/scripts/build.sh)
- [`compile-facade.sh`](crates/wasm/npm/scripts/compile-facade.sh)
- [`measure-wasm-size.mjs`](crates/wasm/npm/scripts/measure-wasm-size.mjs)
- [`pack-and-resolve-tarball.sh`](crates/wasm/npm/scripts/pack-and-resolve-tarball.sh)
- [`prepublish-guard.sh`](crates/wasm/npm/scripts/prepublish-guard.sh)
- [`render-package-json.mjs`](crates/wasm/npm/scripts/render-package-json.mjs)
- [`verify-exports.mjs`](crates/wasm/npm/scripts/verify-exports.mjs)
- [`verify-facade-denylist.mjs`](crates/wasm/npm/scripts/verify-facade-denylist.mjs)
- [`verify-no-raw-exports.mjs`](crates/wasm/npm/scripts/verify-no-raw-exports.mjs)
- [`verify-package-resolution.sh`](crates/wasm/npm/scripts/verify-package-resolution.sh)

</details>

<details>
<summary><code>crates/wasm/npm/src/</code> &mdash; 10 file(s)</summary>

- [`callbacks.ts`](crates/wasm/npm/src/callbacks.ts)
- [`cloudflare.ts`](crates/wasm/npm/src/cloudflare.ts)
- [`default.ts`](crates/wasm/npm/src/default.ts)
- [`envelope.ts`](crates/wasm/npm/src/envelope.ts)
- [`errors.ts`](crates/wasm/npm/src/errors.ts)
- [`index.ts`](crates/wasm/npm/src/index.ts)
- [`internal.ts`](crates/wasm/npm/src/internal.ts)
- [`options.ts`](crates/wasm/npm/src/options.ts)
- [`orderbook.ts`](crates/wasm/npm/src/orderbook.ts)
- [`signing.ts`](crates/wasm/npm/src/signing.ts)

</details>

<details>
<summary><code>crates/wasm/npm/src/raw/</code> &mdash; 4 file(s)</summary>

- [`cloudflare.ts`](crates/wasm/npm/src/raw/cloudflare.ts)
- [`default.ts`](crates/wasm/npm/src/raw/default.ts)
- [`orderbook.ts`](crates/wasm/npm/src/raw/orderbook.ts)
- [`signing.ts`](crates/wasm/npm/src/raw/signing.ts)

</details>

<details>
<summary><code>crates/wasm/npm/tests/</code> &mdash; 7 file(s)</summary>

- [`facade-cancellation.test.ts`](crates/wasm/npm/tests/facade-cancellation.test.ts)
- [`facade-default.test.ts`](crates/wasm/npm/tests/facade-default.test.ts)
- [`facade-error-normalization.test.ts`](crates/wasm/npm/tests/facade-error-normalization.test.ts)
- [`facade-orderbook.test.ts`](crates/wasm/npm/tests/facade-orderbook.test.ts)
- [`facade-resource-cleanup.test.ts`](crates/wasm/npm/tests/facade-resource-cleanup.test.ts)
- [`facade-signing.test.ts`](crates/wasm/npm/tests/facade-signing.test.ts)
- [`fixtures.ts`](crates/wasm/npm/tests/fixtures.ts)

</details>

<details>
<summary><code>crates/wasm/snapshots/facade/</code> &mdash; 5 file(s)</summary>

- [`.keep`](crates/wasm/snapshots/facade/.keep)
- [`cloudflare.d.ts`](crates/wasm/snapshots/facade/cloudflare.d.ts)
- [`default.d.ts`](crates/wasm/snapshots/facade/default.d.ts)
- [`orderbook.d.ts`](crates/wasm/snapshots/facade/orderbook.d.ts)
- [`signing.d.ts`](crates/wasm/snapshots/facade/signing.d.ts)

</details>

<details>
<summary><code>crates/wasm/snapshots/raw/</code> &mdash; 8 file(s)</summary>

- [`.keep`](crates/wasm/snapshots/raw/.keep)
- [`cloudflare-web.d.ts`](crates/wasm/snapshots/raw/cloudflare-web.d.ts)
- [`default-bundler.d.ts`](crates/wasm/snapshots/raw/default-bundler.d.ts)
- [`default-nodejs.d.ts`](crates/wasm/snapshots/raw/default-nodejs.d.ts)
- [`orderbook-bundler.d.ts`](crates/wasm/snapshots/raw/orderbook-bundler.d.ts)
- [`orderbook-nodejs.d.ts`](crates/wasm/snapshots/raw/orderbook-nodejs.d.ts)
- [`signing-bundler.d.ts`](crates/wasm/snapshots/raw/signing-bundler.d.ts)
- [`signing-nodejs.d.ts`](crates/wasm/snapshots/raw/signing-nodejs.d.ts)

</details>

<details>
<summary><code>crates/wasm/src/</code> &mdash; 1 file(s)</summary>

- [`lib.rs`](crates/wasm/src/lib.rs)

</details>

<details>
<summary><code>crates/wasm/src/exports/</code> &mdash; 15 file(s)</summary>

- [`callbacks.rs`](crates/wasm/src/exports/callbacks.rs)
- [`cancel.rs`](crates/wasm/src/exports/cancel.rs)
- [`chains.rs`](crates/wasm/src/exports/chains.rs)
- [`eip1271.rs`](crates/wasm/src/exports/eip1271.rs)
- [`envelope.rs`](crates/wasm/src/exports/envelope.rs)
- [`errors.rs`](crates/wasm/src/exports/errors.rs)
- [`events.rs`](crates/wasm/src/exports/events.rs)
- [`ipfs.rs`](crates/wasm/src/exports/ipfs.rs)
- [`mod.rs`](crates/wasm/src/exports/mod.rs)
- [`orderbook.rs`](crates/wasm/src/exports/orderbook.rs)
- [`registry.rs`](crates/wasm/src/exports/registry.rs)
- [`signing.rs`](crates/wasm/src/exports/signing.rs)
- [`subgraph.rs`](crates/wasm/src/exports/subgraph.rs)
- [`trading.rs`](crates/wasm/src/exports/trading.rs)
- [`transport.rs`](crates/wasm/src/exports/transport.rs)

</details>

<details>
<summary><code>crates/wasm/src/exports/dto/</code> &mdash; 12 file(s)</summary>

- [`app_data.rs`](crates/wasm/src/exports/dto/app_data.rs)
- [`contracts.rs`](crates/wasm/src/exports/dto/contracts.rs)
- [`core.rs`](crates/wasm/src/exports/dto/core.rs)
- [`events.rs`](crates/wasm/src/exports/dto/events.rs)
- [`mod.rs`](crates/wasm/src/exports/dto/mod.rs)
- [`order.rs`](crates/wasm/src/exports/dto/order.rs)
- [`orderbook.rs`](crates/wasm/src/exports/dto/orderbook.rs)
- [`quote.rs`](crates/wasm/src/exports/dto/quote.rs)
- [`signing.rs`](crates/wasm/src/exports/dto/signing.rs)
- [`subgraph.rs`](crates/wasm/src/exports/dto/subgraph.rs)
- [`trading.rs`](crates/wasm/src/exports/dto/trading.rs)
- [`transport.rs`](crates/wasm/src/exports/dto/transport.rs)

</details>

<details>
<summary><code>crates/wasm/tests/</code> &mdash; 18 file(s)</summary>

- [`host_pure_helpers.rs`](crates/wasm/tests/host_pure_helpers.rs)
- [`wasm_callback_contract.rs`](crates/wasm/tests/wasm_callback_contract.rs)
- [`wasm_callback_lifetime_contract.rs`](crates/wasm/tests/wasm_callback_lifetime_contract.rs)
- [`wasm_callback_transport_contract.rs`](crates/wasm/tests/wasm_callback_transport_contract.rs)
- [`wasm_cancellation_contract.rs`](crates/wasm/tests/wasm_cancellation_contract.rs)
- [`wasm_eip1271_contract.rs`](crates/wasm/tests/wasm_eip1271_contract.rs)
- [`wasm_envelope_contract.rs`](crates/wasm/tests/wasm_envelope_contract.rs)
- [`wasm_error_abi_contract.rs`](crates/wasm/tests/wasm_error_abi_contract.rs)
- [`wasm_facade_snapshot_contract.rs`](crates/wasm/tests/wasm_facade_snapshot_contract.rs)
- [`wasm_fail_closed_contract.rs`](crates/wasm/tests/wasm_fail_closed_contract.rs)
- [`wasm_ipfs_contract.rs`](crates/wasm/tests/wasm_ipfs_contract.rs)
- [`wasm_redaction_contract.rs`](crates/wasm/tests/wasm_redaction_contract.rs)
- [`wasm_retry_runner_contract.rs`](crates/wasm/tests/wasm_retry_runner_contract.rs)
- [`wasm_send_sync_contract.rs`](crates/wasm/tests/wasm_send_sync_contract.rs)
- [`wasm_snapshot_surface_contract.rs`](crates/wasm/tests/wasm_snapshot_surface_contract.rs)
- [`wasm_surface_contract.rs`](crates/wasm/tests/wasm_surface_contract.rs)
- [`wasm_transport_policy_contract.rs`](crates/wasm/tests/wasm_transport_policy_contract.rs)
- [`wasm_workflow_coverage_contract.rs`](crates/wasm/tests/wasm_workflow_coverage_contract.rs)

</details>

<details>
<summary><code>crates/wasm/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/wasm/tests/common/mod.rs)

</details>

<details>
<summary><code>crates/wasm/tests/fixtures/</code> &mdash; 1 file(s)</summary>

- [`eip1271_upstream_vector.json`](crates/wasm/tests/fixtures/eip1271_upstream_vector.json)

</details>

<details>
<summary><code>docs/</code> &mdash; 23 file(s)</summary>

- [`alloy-doctrine.md`](docs/alloy-doctrine.md)
- [`alloy-major-release-runbook.md`](docs/alloy-major-release-runbook.md)
- [`architecture.md`](docs/architecture.md)
- [`browser-runtime-proof-posture.md`](docs/browser-runtime-proof-posture.md)
- [`code-of-conduct.md`](docs/code-of-conduct.md)
- [`deployments.md`](docs/deployments.md)
- [`examples.md`](docs/examples.md)
- [`getting-started.md`](docs/getting-started.md)
- [`integrations.md`](docs/integrations.md)
- [`msrv-policy.md`](docs/msrv-policy.md)
- [`observability.md`](docs/observability.md)
- [`parity-matrix.md`](docs/parity-matrix.md)
- [`parity-scope.md`](docs/parity-scope.md)
- [`parity-sources.md`](docs/parity-sources.md)
- [`performance.md`](docs/performance.md)
- [`principles.md`](docs/principles.md)
- [`publication-handoff.md`](docs/publication-handoff.md)
- [`README.md`](docs/README.md)
- [`release-checklist.md`](docs/release-checklist.md)
- [`transport.md`](docs/transport.md)
- [`validation-scope.md`](docs/validation-scope.md)
- [`verification-guide.md`](docs/verification-guide.md)
- [`verification-matrix.md`](docs/verification-matrix.md)

</details>

<details>
<summary><code>docs/adr/</code> &mdash; 63 file(s)</summary>

- [`0000-template.md`](docs/adr/0000-template.md)
- [`0001-multi-crate-sdk-family-with-thin-facade.md`](docs/adr/0001-multi-crate-sdk-family-with-thin-facade.md)
- [`0002-dedicated-trading-orchestration-crate.md`](docs/adr/0002-dedicated-trading-orchestration-crate.md)
- [`0003-separate-read-only-subgraph-crate.md`](docs/adr/0003-separate-read-only-subgraph-crate.md)
- [`0004-feature-gated-browser-wallet-sidecar.md`](docs/adr/0004-feature-gated-browser-wallet-sidecar.md)
- [`0005-boundary-specific-runtime-contracts-and-strong-domain-types.md`](docs/adr/0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [`0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md`](docs/adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)
- [`0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md`](docs/adr/0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md)
- [`0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md`](docs/adr/0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md)
- [`0009-wasm-verification-consoles-hybrid-extensibility-and-two-tier-proof.md`](docs/adr/0009-wasm-verification-consoles-hybrid-extensibility-and-two-tier-proof.md)
- [`0010-runtime-neutral-async-and-transport-posture.md`](docs/adr/0010-runtime-neutral-async-and-transport-posture.md)
- [`0011-typed-amount-boundary-and-typestate-ready-state-construction.md`](docs/adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
- [`0012-alloy-sol-bindings-and-registry-authority.md`](docs/adr/0012-alloy-sol-bindings-and-registry-authority.md)
- [`0013-http-transport-injection-and-typestate-builders.md`](docs/adr/0013-http-transport-injection-and-typestate-builders.md)
- [`0014-eip1271-verification-cache.md`](docs/adr/0014-eip1271-verification-cache.md)
- [`0015-client-side-order-bounds-validator.md`](docs/adr/0015-client-side-order-bounds-validator.md)
- [`0016-split-sell-and-buy-token-balance-enums.md`](docs/adr/0016-split-sell-and-buy-token-balance-enums.md)
- [`0017-typed-orderbook-rejection-parser.md`](docs/adr/0017-typed-orderbook-rejection-parser.md)
- [`0018-typed-app-data-merge.md`](docs/adr/0018-typed-app-data-merge.md)
- [`0019-http-transport-sole-dispatch.md`](docs/adr/0019-http-transport-sole-dispatch.md)
- [`0020-ethflow-owner-threading.md`](docs/adr/0020-ethflow-owner-threading.md)
- [`0021-orderbook-total-fee-policy.md`](docs/adr/0021-orderbook-total-fee-policy.md)
- [`0022-ecdsa-signature-v-normalization.md`](docs/adr/0022-ecdsa-signature-v-normalization.md)
- [`0023-legacy-compatibility-shim-removal.md`](docs/adr/0023-legacy-compatibility-shim-removal.md)
- [`0024-asyncprovider-asyncsigningprovider-capability-split.md`](docs/adr/0024-asyncprovider-asyncsigningprovider-capability-split.md)
- [`0025-workspace-url-redaction-convention.md`](docs/adr/0025-workspace-url-redaction-convention.md)
- [`0026-alloy-major-release-absorption-plan.md`](docs/adr/0026-alloy-major-release-absorption-plan.md)
- [`0027-post-quantum-signing-absorption-plan.md`](docs/adr/0027-post-quantum-signing-absorption-plan.md)
- [`0028-account-abstraction-integration-plan.md`](docs/adr/0028-account-abstraction-integration-plan.md)
- [`0029-trait-evolution-extension-traits.md`](docs/adr/0029-trait-evolution-extension-traits.md)
- [`0030-workspace-locked-versioning-tag-baseline.md`](docs/adr/0030-workspace-locked-versioning-tag-baseline.md)
- [`0031-wire-dto-openapi-driven-with-order-auction-order-split.md`](docs/adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md)
- [`0032-deployment-authority-machine-readable-provenance.md`](docs/adr/0032-deployment-authority-machine-readable-provenance.md)
- [`0033-minimum-viable-panic-surface.md`](docs/adr/0033-minimum-viable-panic-surface.md)
- [`0034-interaction-encoder-target-policy.md`](docs/adr/0034-interaction-encoder-target-policy.md)
- [`0035-alloy-provider-adapter.md`](docs/adr/0035-alloy-provider-adapter.md)
- [`0036-alloy-signer-adapter.md`](docs/adr/0036-alloy-signer-adapter.md)
- [`0037-alloy-umbrella-adapter.md`](docs/adr/0037-alloy-umbrella-adapter.md)
- [`0038-transaction-lifecycle-types.md`](docs/adr/0038-transaction-lifecycle-types.md)
- [`0039-typescript-callable-wasm-sdk-surface.md`](docs/adr/0039-typescript-callable-wasm-sdk-surface.md)
- [`0040-wallet-provider-callback-boundary-for-js-consumers.md`](docs/adr/0040-wallet-provider-callback-boundary-for-js-consumers.md)
- [`0041-transport-policy-l3-layering.md`](docs/adr/0041-transport-policy-l3-layering.md)
- [`0042-pure-helpers-extraction.md`](docs/adr/0042-pure-helpers-extraction.md)
- [`0043-callback-registry-internalization.md`](docs/adr/0043-callback-registry-internalization.md)
- [`0044-bundle-size-profile-and-flavor-builds.md`](docs/adr/0044-bundle-size-profile-and-flavor-builds.md)
- [`0045-async-signer-trait-narrowing.md`](docs/adr/0045-async-signer-trait-narrowing.md)
- [`0046-transport-policy-js-exposure.md`](docs/adr/0046-transport-policy-js-exposure.md)
- [`0047-typescript-facade-architecture.md`](docs/adr/0047-typescript-facade-architecture.md)
- [`0048-composable-conditional-order-framework.md`](docs/adr/0048-composable-conditional-order-framework.md)
- [`0049-cow-shed-account-abstraction-proxy.md`](docs/adr/0049-cow-shed-account-abstraction-proxy.md)
- [`0050-eip1271-signature-blob-encoding.md`](docs/adr/0050-eip1271-signature-blob-encoding.md)
- [`0051-signing-owned-eip1271-signature-provider-trait.md`](docs/adr/0051-signing-owned-eip1271-signature-provider-trait.md)
- [`0052-alloy-primitives-canonical-primitive-layer.md`](docs/adr/0052-alloy-primitives-canonical-primitive-layer.md)
- [`0053-typed-signer-rejection-classification.md`](docs/adr/0053-typed-signer-rejection-classification.md)
- [`0054-onchain-order-event-decoding-is-fail-closed.md`](docs/adr/0054-onchain-order-event-decoding-is-fail-closed.md)
- [`0055-bounded-response-reads.md`](docs/adr/0055-bounded-response-reads.md)
- [`0056-settlement-event-decoding-is-fail-closed.md`](docs/adr/0056-settlement-event-decoding-is-fail-closed.md)
- [`0057-log-provider-capability-trait.md`](docs/adr/0057-log-provider-capability-trait.md)
- [`0058-typed-quote-request-response-surface.md`](docs/adr/0058-typed-quote-request-response-surface.md)
- [`0059-hash-concrete-orderdata-directly.md`](docs/adr/0059-hash-concrete-orderdata-directly.md)
- [`0060-uniform-error-classification.md`](docs/adr/0060-uniform-error-classification.md)
- [`0061-wasm-abi-receiver-pay-to-owner.md`](docs/adr/0061-wasm-abi-receiver-pay-to-owner.md)
- [`README.md`](docs/adr/README.md)

</details>

<details>
<summary><code>docs/audit/</code> &mdash; 65 file(s)</summary>

- [`alloy-provider-adapter-audit.md`](docs/audit/alloy-provider-adapter-audit.md)
- [`alloy-signer-adapter-audit.md`](docs/audit/alloy-signer-adapter-audit.md)
- [`alloy-umbrella-adapter-audit.md`](docs/audit/alloy-umbrella-adapter-audit.md)
- [`bounded-response-reads-audit.md`](docs/audit/bounded-response-reads-audit.md)
- [`browser-wallet-alloy-dependency-audit.md`](docs/audit/browser-wallet-alloy-dependency-audit.md)
- [`browser-wallet-chain-coherence-audit.md`](docs/audit/browser-wallet-chain-coherence-audit.md)
- [`browser-wallet-trust-posture-audit.md`](docs/audit/browser-wallet-trust-posture-audit.md)
- [`cid-dependency-audit.md`](docs/audit/cid-dependency-audit.md)
- [`composable-contract-bindings-audit.md`](docs/audit/composable-contract-bindings-audit.md)
- [`composable-watch-tower-boundary-audit.md`](docs/audit/composable-watch-tower-boundary-audit.md)
- [`contract-bindings-parity-audit.md`](docs/audit/contract-bindings-parity-audit.md)
- [`cooperative-cancellation-contract-audit.md`](docs/audit/cooperative-cancellation-contract-audit.md)
- [`cow-sdk-wasm-comparative-benchmark-validation-note.md`](docs/audit/cow-sdk-wasm-comparative-benchmark-validation-note.md)
- [`cow-shed-app-data-integration-audit.md`](docs/audit/cow-shed-app-data-integration-audit.md)
- [`cow-shed-contract-bindings-audit.md`](docs/audit/cow-shed-contract-bindings-audit.md)
- [`credential-surface-audit.md`](docs/audit/credential-surface-audit.md)
- [`credential-surface-contract-hygiene-audit.md`](docs/audit/credential-surface-contract-hygiene-audit.md)
- [`dependency-gate-audit.md`](docs/audit/dependency-gate-audit.md)
- [`deployment-registry-audit.md`](docs/audit/deployment-registry-audit.md)
- [`ecdsa-signature-normalization-audit.md`](docs/audit/ecdsa-signature-normalization-audit.md)
- [`eip1271-verification-cache-audit.md`](docs/audit/eip1271-verification-cache-audit.md)
- [`error-classification-audit.md`](docs/audit/error-classification-audit.md)
- [`fuzz-coverage-audit.md`](docs/audit/fuzz-coverage-audit.md)
- [`http-transport-contract-audit.md`](docs/audit/http-transport-contract-audit.md)
- [`lens-chain-evidence-audit.md`](docs/audit/lens-chain-evidence-audit.md)
- [`log-provider-capability-audit.md`](docs/audit/log-provider-capability-audit.md)
- [`onchain-order-log-decoding-audit.md`](docs/audit/onchain-order-log-decoding-audit.md)
- [`panic-free-public-surface-audit.md`](docs/audit/panic-free-public-surface-audit.md)
- [`partner-api-routing-audit.md`](docs/audit/partner-api-routing-audit.md)
- [`quote-request-app-data-fix-review.md`](docs/audit/quote-request-app-data-fix-review.md)
- [`quote-response-surface-audit.md`](docs/audit/quote-response-surface-audit.md)
- [`README.md`](docs/audit/README.md)
- [`settlement-event-log-decoding-audit.md`](docs/audit/settlement-event-log-decoding-audit.md)
- [`shared-logic-reviewability-audit.md`](docs/audit/shared-logic-reviewability-audit.md)
- [`signer-error-classification-audit.md`](docs/audit/signer-error-classification-audit.md)
- [`source-lock-provenance-audit.md`](docs/audit/source-lock-provenance-audit.md)
- [`subgraph-error-display-audit.md`](docs/audit/subgraph-error-display-audit.md)
- [`trade-parameter-lifecycle-audit.md`](docs/audit/trade-parameter-lifecycle-audit.md)
- [`trading-app-data-merge-audit.md`](docs/audit/trading-app-data-merge-audit.md)
- [`trading-ethflow-owner-identity-audit.md`](docs/audit/trading-ethflow-owner-identity-audit.md)
- [`trading-order-bounds-validator-audit.md`](docs/audit/trading-order-bounds-validator-audit.md)
- [`trading-order-construction-integrity-audit.md`](docs/audit/trading-order-construction-integrity-audit.md)
- [`trading-orderbook-context-audit.md`](docs/audit/trading-orderbook-context-audit.md)
- [`trading-quote-orderbook-binding-audit.md`](docs/audit/trading-quote-orderbook-binding-audit.md)
- [`trading-sdk-runtime-prerequisites-audit.md`](docs/audit/trading-sdk-runtime-prerequisites-audit.md)
- [`transaction-receipt-shape-audit.md`](docs/audit/transaction-receipt-shape-audit.md)
- [`transport-policy-coverage-audit.md`](docs/audit/transport-policy-coverage-audit.md)
- [`typestate-builder-contract-audit.md`](docs/audit/typestate-builder-contract-audit.md)
- [`unsafe-code-policy-audit.md`](docs/audit/unsafe-code-policy-audit.md)
- [`url-credential-redaction-audit.md`](docs/audit/url-credential-redaction-audit.md)
- [`wasm-browser-runner-determinism-audit.md`](docs/audit/wasm-browser-runner-determinism-audit.md)
- [`wasm-callback-shape-design-audit.md`](docs/audit/wasm-callback-shape-design-audit.md)
- [`wasm-capability-coverage-audit.md`](docs/audit/wasm-capability-coverage-audit.md)
- [`wasm-component-model-future-prep-audit.md`](docs/audit/wasm-component-model-future-prep-audit.md)
- [`wasm-eip1271-parity-audit.md`](docs/audit/wasm-eip1271-parity-audit.md)
- [`wasm-example-proof-posture-audit.md`](docs/audit/wasm-example-proof-posture-audit.md)
- [`wasm-facade-architecture-audit.md`](docs/audit/wasm-facade-architecture-audit.md)
- [`wasm-performance-budget-audit.md`](docs/audit/wasm-performance-budget-audit.md)
- [`wasm-public-api-stability-audit.md`](docs/audit/wasm-public-api-stability-audit.md)
- [`wasm-schema-versioning-policy-audit.md`](docs/audit/wasm-schema-versioning-policy-audit.md)
- [`wasm-surface-audit.md`](docs/audit/wasm-surface-audit.md)
- [`wasm-type-generation-audit.md`](docs/audit/wasm-type-generation-audit.md)
- [`wasm-unsupported-target-audit.md`](docs/audit/wasm-unsupported-target-audit.md)
- [`wire-dto-coverage-audit.md`](docs/audit/wire-dto-coverage-audit.md)
- [`workflow-security-audit.md`](docs/audit/workflow-security-audit.md)

</details>

<details>
<summary><code>docs/providers/</code> &mdash; 2 file(s)</summary>

- [`adapting-alloy.md`](docs/providers/adapting-alloy.md)
- [`README.md`](docs/providers/README.md)

</details>

<details>
<summary><code>e2e/</code> &mdash; 1 file(s)</summary>

- [`tsconfig.base.json`](e2e/tsconfig.base.json)

</details>

<details>
<summary><code>e2e/browser-wallet/</code> &mdash; 5 file(s)</summary>

- [`bun.lock`](e2e/browser-wallet/bun.lock)
- [`globals.d.ts`](e2e/browser-wallet/globals.d.ts)
- [`package.json`](e2e/browser-wallet/package.json)
- [`playwright.config.ts`](e2e/browser-wallet/playwright.config.ts)
- [`tsconfig.json`](e2e/browser-wallet/tsconfig.json)

</details>

<details>
<summary><code>e2e/browser-wallet/fixtures/</code> &mdash; 2 file(s)</summary>

- [`cow-api.ts`](e2e/browser-wallet/fixtures/cow-api.ts)
- [`injected-wallet.ts`](e2e/browser-wallet/fixtures/injected-wallet.ts)

</details>

<details>
<summary><code>e2e/browser-wallet/test-results/</code> &mdash; 1 file(s)</summary>

- [`.last-run.json`](e2e/browser-wallet/test-results/.last-run.json)

</details>

<details>
<summary><code>e2e/browser-wallet/tests/</code> &mdash; 2 file(s)</summary>

- [`browser-wallet-console.spec.ts`](e2e/browser-wallet/tests/browser-wallet-console.spec.ts)
- [`injected-chain-coherence.spec.ts`](e2e/browser-wallet/tests/injected-chain-coherence.spec.ts)

</details>

<details>
<summary><code>e2e/sdk-verification/</code> &mdash; 4 file(s)</summary>

- [`bun.lock`](e2e/sdk-verification/bun.lock)
- [`package.json`](e2e/sdk-verification/package.json)
- [`playwright.config.ts`](e2e/sdk-verification/playwright.config.ts)
- [`tsconfig.json`](e2e/sdk-verification/tsconfig.json)

</details>

<details>
<summary><code>e2e/sdk-verification/fixtures/</code> &mdash; 1 file(s)</summary>

- [`cow-api.ts`](e2e/sdk-verification/fixtures/cow-api.ts)

</details>

<details>
<summary><code>e2e/sdk-verification/test-results/</code> &mdash; 1 file(s)</summary>

- [`.last-run.json`](e2e/sdk-verification/test-results/.last-run.json)

</details>

<details>
<summary><code>e2e/sdk-verification/tests/</code> &mdash; 3 file(s)</summary>

- [`live-orderbook-readiness.spec.ts`](e2e/sdk-verification/tests/live-orderbook-readiness.spec.ts)
- [`manual-network-panels.spec.ts`](e2e/sdk-verification/tests/manual-network-panels.spec.ts)
- [`sdk-verification-console.spec.ts`](e2e/sdk-verification/tests/sdk-verification-console.spec.ts)

</details>

<details>
<summary><code>e2e/wasm-typescript/</code> &mdash; 8 file(s)</summary>

- [`index.html`](e2e/wasm-typescript/index.html)
- [`package.json`](e2e/wasm-typescript/package.json)
- [`playwright.config.ts`](e2e/wasm-typescript/playwright.config.ts)
- [`pnpm-lock.yaml`](e2e/wasm-typescript/pnpm-lock.yaml)
- [`pnpm-workspace.yaml`](e2e/wasm-typescript/pnpm-workspace.yaml)
- [`tsconfig.json`](e2e/wasm-typescript/tsconfig.json)
- [`vite.config.ts`](e2e/wasm-typescript/vite.config.ts)
- [`vitest.config.ts`](e2e/wasm-typescript/vitest.config.ts)

</details>

<details>
<summary><code>e2e/wasm-typescript-cf/</code> &mdash; 6 file(s)</summary>

- [`package.json`](e2e/wasm-typescript-cf/package.json)
- [`pnpm-lock.yaml`](e2e/wasm-typescript-cf/pnpm-lock.yaml)
- [`pnpm-workspace.yaml`](e2e/wasm-typescript-cf/pnpm-workspace.yaml)
- [`tsconfig.json`](e2e/wasm-typescript-cf/tsconfig.json)
- [`vitest.config.ts`](e2e/wasm-typescript-cf/vitest.config.ts)
- [`wrangler.toml`](e2e/wasm-typescript-cf/wrangler.toml)

</details>

<details>
<summary><code>e2e/wasm-typescript-cf/src/</code> &mdash; 2 file(s)</summary>

- [`wasm.d.ts`](e2e/wasm-typescript-cf/src/wasm.d.ts)
- [`worker.ts`](e2e/wasm-typescript-cf/src/worker.ts)

</details>

<details>
<summary><code>e2e/wasm-typescript-cf/tests/</code> &mdash; 3 file(s)</summary>

- [`forbidden-instantiation.spec.ts`](e2e/wasm-typescript-cf/tests/forbidden-instantiation.spec.ts)
- [`init-once.spec.ts`](e2e/wasm-typescript-cf/tests/init-once.spec.ts)
- [`orderbook.spec.ts`](e2e/wasm-typescript-cf/tests/orderbook.spec.ts)

</details>

<details>
<summary><code>e2e/wasm-typescript-deno/</code> &mdash; 1 file(s)</summary>

- [`deno.jsonc`](e2e/wasm-typescript-deno/deno.jsonc)

</details>

<details>
<summary><code>e2e/wasm-typescript-deno/src/</code> &mdash; 1 file(s)</summary>

- [`index.ts`](e2e/wasm-typescript-deno/src/index.ts)

</details>

<details>
<summary><code>e2e/wasm-typescript-deno/tests/</code> &mdash; 1 file(s)</summary>

- [`signing_test.ts`](e2e/wasm-typescript-deno/tests/signing_test.ts)

</details>

<details>
<summary><code>e2e/wasm-typescript/src/</code> &mdash; 1 file(s)</summary>

- [`index.ts`](e2e/wasm-typescript/src/index.ts)

</details>

<details>
<summary><code>e2e/wasm-typescript/tests/</code> &mdash; 4 file(s)</summary>

- [`eip1271.spec.ts`](e2e/wasm-typescript/tests/eip1271.spec.ts)
- [`orderbook.spec.ts`](e2e/wasm-typescript/tests/orderbook.spec.ts)
- [`signing.spec.ts`](e2e/wasm-typescript/tests/signing.spec.ts)
- [`transport.spec.ts`](e2e/wasm-typescript/tests/transport.spec.ts)

</details>

<details>
<summary><code>e2e/wasm-typescript/tests/browser/</code> &mdash; 1 file(s)</summary>

- [`browser.spec.ts`](e2e/wasm-typescript/tests/browser/browser.spec.ts)

</details>

<details>
<summary><code>examples/</code> &mdash; 2 file(s)</summary>

- [`LICENSE`](examples/LICENSE)
- [`README.md`](examples/README.md)

</details>

<details>
<summary><code>examples/native/</code> &mdash; 3 file(s)</summary>

- [`Cargo.lock`](examples/native/Cargo.lock)
- [`Cargo.toml`](examples/native/Cargo.toml)
- [`README.md`](examples/native/README.md)

</details>

<details>
<summary><code>examples/native/scenarios/</code> &mdash; 26 file(s)</summary>

- [`alloy_provider_only.rs`](examples/native/scenarios/alloy_provider_only.rs)
- [`alloy_provider_with_custom_signer.rs`](examples/native/scenarios/alloy_provider_with_custom_signer.rs)
- [`alloy_quickstart.rs`](examples/native/scenarios/alloy_quickstart.rs)
- [`alloy_signer_only.rs`](examples/native/scenarios/alloy_signer_only.rs)
- [`alloy_signer_with_custom_provider.rs`](examples/native/scenarios/alloy_signer_with_custom_provider.rs)
- [`alloy_trading_full_flow.rs`](examples/native/scenarios/alloy_trading_full_flow.rs)
- [`app_data_roundtrip.rs`](examples/native/scenarios/app_data_roundtrip.rs)
- [`cancellation_combinator.rs`](examples/native/scenarios/cancellation_combinator.rs)
- [`error_classification_simulation.rs`](examples/native/scenarios/error_classification_simulation.rs)
- [`ethflow_transaction_simulation.rs`](examples/native/scenarios/ethflow_transaction_simulation.rs)
- [`limit_order_simulation.rs`](examples/native/scenarios/limit_order_simulation.rs)
- [`live_order_sepolia.rs`](examples/native/scenarios/live_order_sepolia.rs)
- [`onchain_order_actions_simulation.rs`](examples/native/scenarios/onchain_order_actions_simulation.rs)
- [`order_lifecycle_simulation.rs`](examples/native/scenarios/order_lifecycle_simulation.rs)
- [`order_list_history_simulation.rs`](examples/native/scenarios/order_list_history_simulation.rs)
- [`orderbook_live_probe.rs`](examples/native/scenarios/orderbook_live_probe.rs)
- [`orderbook_transport_roundtrip.rs`](examples/native/scenarios/orderbook_transport_roundtrip.rs)
- [`quote_only_simulation.rs`](examples/native/scenarios/quote_only_simulation.rs)
- [`sdk_surface_report.rs`](examples/native/scenarios/sdk_surface_report.rs)
- [`signing_roundtrip.rs`](examples/native/scenarios/signing_roundtrip.rs)
- [`simplest_swap_quickstart.rs`](examples/native/scenarios/simplest_swap_quickstart.rs)
- [`subgraph_custom_query_roundtrip.rs`](examples/native/scenarios/subgraph_custom_query_roundtrip.rs)
- [`subgraph_live_query.rs`](examples/native/scenarios/subgraph_live_query.rs)
- [`subgraph_query_roundtrip.rs`](examples/native/scenarios/subgraph_query_roundtrip.rs)
- [`trading_sdk_simulation.rs`](examples/native/scenarios/trading_sdk_simulation.rs)
- [`transaction_lifecycle.rs`](examples/native/scenarios/transaction_lifecycle.rs)

</details>

<details>
<summary><code>examples/native/src/</code> &mdash; 2 file(s)</summary>

- [`lib.rs`](examples/native/src/lib.rs)
- [`support.rs`](examples/native/src/support.rs)

</details>

<details>
<summary><code>examples/native/tests/</code> &mdash; 1 file(s)</summary>

- [`scenario_contract.rs`](examples/native/tests/scenario_contract.rs)

</details>

<details>
<summary><code>examples/wasm/</code> &mdash; 4 file(s)</summary>

- [`Cargo.lock`](examples/wasm/Cargo.lock)
- [`Cargo.toml`](examples/wasm/Cargo.toml)
- [`index.html`](examples/wasm/index.html)
- [`README.md`](examples/wasm/README.md)

</details>

<details>
<summary><code>examples/wasm-typescript-browser-mm/</code> &mdash; 7 file(s)</summary>

- [`index.html`](examples/wasm-typescript-browser-mm/index.html)
- [`package.json`](examples/wasm-typescript-browser-mm/package.json)
- [`playwright.config.ts`](examples/wasm-typescript-browser-mm/playwright.config.ts)
- [`pnpm-lock.yaml`](examples/wasm-typescript-browser-mm/pnpm-lock.yaml)
- [`README.md`](examples/wasm-typescript-browser-mm/README.md)
- [`tsconfig.json`](examples/wasm-typescript-browser-mm/tsconfig.json)
- [`vite.config.ts`](examples/wasm-typescript-browser-mm/vite.config.ts)

</details>

<details>
<summary><code>examples/wasm-typescript-browser-mm/src/</code> &mdash; 1 file(s)</summary>

- [`main.ts`](examples/wasm-typescript-browser-mm/src/main.ts)

</details>

<details>
<summary><code>examples/wasm-typescript-browser-mm/tests/</code> &mdash; 1 file(s)</summary>

- [`browser.spec.ts`](examples/wasm-typescript-browser-mm/tests/browser.spec.ts)

</details>

<details>
<summary><code>examples/wasm-typescript-cloudflare-proxy/</code> &mdash; 7 file(s)</summary>

- [`package.json`](examples/wasm-typescript-cloudflare-proxy/package.json)
- [`pnpm-lock.yaml`](examples/wasm-typescript-cloudflare-proxy/pnpm-lock.yaml)
- [`pnpm-workspace.yaml`](examples/wasm-typescript-cloudflare-proxy/pnpm-workspace.yaml)
- [`README.md`](examples/wasm-typescript-cloudflare-proxy/README.md)
- [`tsconfig.json`](examples/wasm-typescript-cloudflare-proxy/tsconfig.json)
- [`vitest.config.ts`](examples/wasm-typescript-cloudflare-proxy/vitest.config.ts)
- [`wrangler.toml`](examples/wasm-typescript-cloudflare-proxy/wrangler.toml)

</details>

<details>
<summary><code>examples/wasm-typescript-cloudflare-proxy/scripts/</code> &mdash; 1 file(s)</summary>

- [`build.mjs`](examples/wasm-typescript-cloudflare-proxy/scripts/build.mjs)

</details>

<details>
<summary><code>examples/wasm-typescript-cloudflare-proxy/src/</code> &mdash; 3 file(s)</summary>

- [`vite-env.d.ts`](examples/wasm-typescript-cloudflare-proxy/src/vite-env.d.ts)
- [`wasm.d.ts`](examples/wasm-typescript-cloudflare-proxy/src/wasm.d.ts)
- [`worker.ts`](examples/wasm-typescript-cloudflare-proxy/src/worker.ts)

</details>

<details>
<summary><code>examples/wasm-typescript-cloudflare-proxy/tests/</code> &mdash; 3 file(s)</summary>

- [`forbidden-instantiation.spec.ts`](examples/wasm-typescript-cloudflare-proxy/tests/forbidden-instantiation.spec.ts)
- [`proxy.spec.ts`](examples/wasm-typescript-cloudflare-proxy/tests/proxy.spec.ts)
- [`worker.spec.ts`](examples/wasm-typescript-cloudflare-proxy/tests/worker.spec.ts)

</details>

<details>
<summary><code>examples/wasm-typescript-node-viem/</code> &mdash; 4 file(s)</summary>

- [`package.json`](examples/wasm-typescript-node-viem/package.json)
- [`pnpm-lock.yaml`](examples/wasm-typescript-node-viem/pnpm-lock.yaml)
- [`README.md`](examples/wasm-typescript-node-viem/README.md)
- [`tsconfig.json`](examples/wasm-typescript-node-viem/tsconfig.json)

</details>

<details>
<summary><code>examples/wasm-typescript-node-viem/src/</code> &mdash; 2 file(s)</summary>

- [`index.test.ts`](examples/wasm-typescript-node-viem/src/index.test.ts)
- [`index.ts`](examples/wasm-typescript-node-viem/src/index.ts)

</details>

<details>
<summary><code>examples/wasm/browser-wallet-console/</code> &mdash; 5 file(s)</summary>

- [`.gitignore`](examples/wasm/browser-wallet-console/.gitignore)
- [`Cargo.toml`](examples/wasm/browser-wallet-console/Cargo.toml)
- [`index.html`](examples/wasm/browser-wallet-console/index.html)
- [`LICENSE`](examples/wasm/browser-wallet-console/LICENSE)
- [`README.md`](examples/wasm/browser-wallet-console/README.md)

</details>

<details>
<summary><code>examples/wasm/browser-wallet-console/src/</code> &mdash; 1 file(s)</summary>

- [`lib.rs`](examples/wasm/browser-wallet-console/src/lib.rs)

</details>

<details>
<summary><code>examples/wasm/browser-wallet-console/tests/</code> &mdash; 6 file(s)</summary>

- [`selection_confirmation_contract.rs`](examples/wasm/browser-wallet-console/tests/selection_confirmation_contract.rs)
- [`selection_reconnect_contract.rs`](examples/wasm/browser-wallet-console/tests/selection_reconnect_contract.rs)
- [`session_actions_contract.rs`](examples/wasm/browser-wallet-console/tests/session_actions_contract.rs)
- [`transport_symbol_smoke.rs`](examples/wasm/browser-wallet-console/tests/transport_symbol_smoke.rs)
- [`walkthrough_contract.rs`](examples/wasm/browser-wallet-console/tests/walkthrough_contract.rs)
- [`wasm_deterministic.rs`](examples/wasm/browser-wallet-console/tests/wasm_deterministic.rs)

</details>

<details>
<summary><code>examples/wasm/sdk-verification-console/</code> &mdash; 5 file(s)</summary>

- [`.gitignore`](examples/wasm/sdk-verification-console/.gitignore)
- [`Cargo.toml`](examples/wasm/sdk-verification-console/Cargo.toml)
- [`index.html`](examples/wasm/sdk-verification-console/index.html)
- [`LICENSE`](examples/wasm/sdk-verification-console/LICENSE)
- [`README.md`](examples/wasm/sdk-verification-console/README.md)

</details>

<details>
<summary><code>examples/wasm/sdk-verification-console/src/</code> &mdash; 1 file(s)</summary>

- [`lib.rs`](examples/wasm/sdk-verification-console/src/lib.rs)

</details>

<details>
<summary><code>examples/wasm/sdk-verification-console/tests/</code> &mdash; 4 file(s)</summary>

- [`defaults_smoke.rs`](examples/wasm/sdk-verification-console/tests/defaults_smoke.rs)
- [`deterministic_exports.rs`](examples/wasm/sdk-verification-console/tests/deterministic_exports.rs)
- [`transport_symbol_smoke.rs`](examples/wasm/sdk-verification-console/tests/transport_symbol_smoke.rs)
- [`walkthrough_contract.rs`](examples/wasm/sdk-verification-console/tests/walkthrough_contract.rs)

</details>

<details>
<summary><code>fuzz/</code> &mdash; 3 file(s)</summary>

- [`Cargo.lock`](fuzz/Cargo.lock)
- [`Cargo.toml`](fuzz/Cargo.toml)
- [`README.md`](fuzz/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_amount_from_units/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_amount_from_units/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_amount_parse/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_amount_parse/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_amount_parse_units/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_amount_parse_units/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_app_data_cid_roundtrip/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_app_data_cid_roundtrip/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_app_data_merge/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_app_data_merge/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_app_data_params_from_doc/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_app_data_params_from_doc/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_app_data_size_limit/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_app_data_size_limit/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_append_query_string/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_append_query_string/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_calculate_total_fee/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_calculate_total_fee/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_cid_to_app_data_hex/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_cid_to_app_data_hex/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_contract_call_serde/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_contract_call_serde/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_core_identity_validators/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_core_identity_validators/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_decode_magic_value_response/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_decode_magic_value_response/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_decoded_body_canonical_status_text/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_decoded_body_canonical_status_text/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_ecdsa_v_normalization/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_ecdsa_v_normalization/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_eip1271_signature_data_codec/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_eip1271_signature_data_codec/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_erc20_permit_typed_data_hash/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_erc20_permit_typed_data_hash/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_eth_flow_event_log_decode/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_eth_flow_event_log_decode/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_ethflow_create_order_encode/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_ethflow_create_order_encode/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_flashloan_hints/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_flashloan_hints/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_hash_order_cancellations/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_hash_order_cancellations/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_hook_list_deserialize/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_hook_list_deserialize/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_jitter_delay_for_attempt/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_jitter_delay_for_attempt/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_onchain_order_log_decode/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_onchain_order_log_decode/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_order_bounds_validator/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_order_bounds_validator/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_order_signature_classify/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_order_signature_classify/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_order_uid_pack_unpack/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_order_uid_pack_unpack/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_orderbook_rejection_code/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_orderbook_rejection_code/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_orderbook_rejection_decode/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_orderbook_rejection_decode/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_parse_retry_after/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_parse_retry_after/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_partner_fee_from_value/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_partner_fee_from_value/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_recover_ecdsa_address/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_recover_ecdsa_address/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_recoverable_signature_differential/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_recoverable_signature_differential/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_recoverable_signature_parse_hex/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_recoverable_signature_parse_hex/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_redact_response_body/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_redact_response_body/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_retry_policy_delay/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_retry_policy_delay/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_rpc_error_payload_serde/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_rpc_error_payload_serde/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_schema_version_is_semver/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_schema_version_is_semver/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_settlement_event_log_decode/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_settlement_event_log_decode/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_settlement_invalidate_order_encode/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_settlement_invalidate_order_encode/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_settlement_settle_encode/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_settlement_settle_encode/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_signed_amount_parse/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_signed_amount_parse/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_signing_domain_separator/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_signing_domain_separator/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_slippage_amounts/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_slippage_amounts/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_slippage_policy_helpers/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_slippage_policy_helpers/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_stringify_deterministic/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_stringify_deterministic/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_subgraph_graphql_error_decode/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_subgraph_graphql_error_decode/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_transaction_request_serde/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_transaction_request_serde/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_transport_error_classify/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_transport_error_classify/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_typed_data_digest/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_typed_data_digest/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_valid_to_relative/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_valid_to_relative/README.md)

</details>

<details>
<summary><code>fuzz/corpus/fuzz_vault_relayer_transfer_from_accounts_encode/</code> &mdash; 1 file(s)</summary>

- [`README.md`](fuzz/corpus/fuzz_vault_relayer_transfer_from_accounts_encode/README.md)

</details>

<details>
<summary><code>fuzz/fuzz_targets/</code> &mdash; 52 file(s)</summary>

- [`fuzz_amount_from_units.rs`](fuzz/fuzz_targets/fuzz_amount_from_units.rs)
- [`fuzz_amount_parse_units.rs`](fuzz/fuzz_targets/fuzz_amount_parse_units.rs)
- [`fuzz_amount_parse.rs`](fuzz/fuzz_targets/fuzz_amount_parse.rs)
- [`fuzz_app_data_cid_roundtrip.rs`](fuzz/fuzz_targets/fuzz_app_data_cid_roundtrip.rs)
- [`fuzz_app_data_merge.rs`](fuzz/fuzz_targets/fuzz_app_data_merge.rs)
- [`fuzz_app_data_params_from_doc.rs`](fuzz/fuzz_targets/fuzz_app_data_params_from_doc.rs)
- [`fuzz_app_data_size_limit.rs`](fuzz/fuzz_targets/fuzz_app_data_size_limit.rs)
- [`fuzz_append_query_string.rs`](fuzz/fuzz_targets/fuzz_append_query_string.rs)
- [`fuzz_calculate_total_fee.rs`](fuzz/fuzz_targets/fuzz_calculate_total_fee.rs)
- [`fuzz_cid_to_app_data_hex.rs`](fuzz/fuzz_targets/fuzz_cid_to_app_data_hex.rs)
- [`fuzz_contract_call_serde.rs`](fuzz/fuzz_targets/fuzz_contract_call_serde.rs)
- [`fuzz_core_identity_validators.rs`](fuzz/fuzz_targets/fuzz_core_identity_validators.rs)
- [`fuzz_decode_magic_value_response.rs`](fuzz/fuzz_targets/fuzz_decode_magic_value_response.rs)
- [`fuzz_decoded_body_canonical_status_text.rs`](fuzz/fuzz_targets/fuzz_decoded_body_canonical_status_text.rs)
- [`fuzz_ecdsa_v_normalization.rs`](fuzz/fuzz_targets/fuzz_ecdsa_v_normalization.rs)
- [`fuzz_eip1271_signature_data_codec.rs`](fuzz/fuzz_targets/fuzz_eip1271_signature_data_codec.rs)
- [`fuzz_erc20_permit_typed_data_hash.rs`](fuzz/fuzz_targets/fuzz_erc20_permit_typed_data_hash.rs)
- [`fuzz_eth_flow_event_log_decode.rs`](fuzz/fuzz_targets/fuzz_eth_flow_event_log_decode.rs)
- [`fuzz_ethflow_create_order_encode.rs`](fuzz/fuzz_targets/fuzz_ethflow_create_order_encode.rs)
- [`fuzz_flashloan_hints.rs`](fuzz/fuzz_targets/fuzz_flashloan_hints.rs)
- [`fuzz_hash_order_cancellations.rs`](fuzz/fuzz_targets/fuzz_hash_order_cancellations.rs)
- [`fuzz_hook_list_deserialize.rs`](fuzz/fuzz_targets/fuzz_hook_list_deserialize.rs)
- [`fuzz_jitter_delay_for_attempt.rs`](fuzz/fuzz_targets/fuzz_jitter_delay_for_attempt.rs)
- [`fuzz_onchain_order_log_decode.rs`](fuzz/fuzz_targets/fuzz_onchain_order_log_decode.rs)
- [`fuzz_order_bounds_validator.rs`](fuzz/fuzz_targets/fuzz_order_bounds_validator.rs)
- [`fuzz_order_signature_classify.rs`](fuzz/fuzz_targets/fuzz_order_signature_classify.rs)
- [`fuzz_order_uid_pack_unpack.rs`](fuzz/fuzz_targets/fuzz_order_uid_pack_unpack.rs)
- [`fuzz_orderbook_rejection_code.rs`](fuzz/fuzz_targets/fuzz_orderbook_rejection_code.rs)
- [`fuzz_orderbook_rejection_decode.rs`](fuzz/fuzz_targets/fuzz_orderbook_rejection_decode.rs)
- [`fuzz_parse_retry_after.rs`](fuzz/fuzz_targets/fuzz_parse_retry_after.rs)
- [`fuzz_partner_fee_from_value.rs`](fuzz/fuzz_targets/fuzz_partner_fee_from_value.rs)
- [`fuzz_recover_ecdsa_address.rs`](fuzz/fuzz_targets/fuzz_recover_ecdsa_address.rs)
- [`fuzz_recoverable_signature_differential.rs`](fuzz/fuzz_targets/fuzz_recoverable_signature_differential.rs)
- [`fuzz_recoverable_signature_parse_hex.rs`](fuzz/fuzz_targets/fuzz_recoverable_signature_parse_hex.rs)
- [`fuzz_redact_response_body.rs`](fuzz/fuzz_targets/fuzz_redact_response_body.rs)
- [`fuzz_retry_policy_delay.rs`](fuzz/fuzz_targets/fuzz_retry_policy_delay.rs)
- [`fuzz_rpc_error_payload_serde.rs`](fuzz/fuzz_targets/fuzz_rpc_error_payload_serde.rs)
- [`fuzz_schema_version_is_semver.rs`](fuzz/fuzz_targets/fuzz_schema_version_is_semver.rs)
- [`fuzz_settlement_event_log_decode.rs`](fuzz/fuzz_targets/fuzz_settlement_event_log_decode.rs)
- [`fuzz_settlement_invalidate_order_encode.rs`](fuzz/fuzz_targets/fuzz_settlement_invalidate_order_encode.rs)
- [`fuzz_settlement_settle_encode.rs`](fuzz/fuzz_targets/fuzz_settlement_settle_encode.rs)
- [`fuzz_signed_amount_parse.rs`](fuzz/fuzz_targets/fuzz_signed_amount_parse.rs)
- [`fuzz_signing_domain_separator.rs`](fuzz/fuzz_targets/fuzz_signing_domain_separator.rs)
- [`fuzz_slippage_amounts.rs`](fuzz/fuzz_targets/fuzz_slippage_amounts.rs)
- [`fuzz_slippage_policy_helpers.rs`](fuzz/fuzz_targets/fuzz_slippage_policy_helpers.rs)
- [`fuzz_stringify_deterministic.rs`](fuzz/fuzz_targets/fuzz_stringify_deterministic.rs)
- [`fuzz_subgraph_graphql_error_decode.rs`](fuzz/fuzz_targets/fuzz_subgraph_graphql_error_decode.rs)
- [`fuzz_transaction_request_serde.rs`](fuzz/fuzz_targets/fuzz_transaction_request_serde.rs)
- [`fuzz_transport_error_classify.rs`](fuzz/fuzz_targets/fuzz_transport_error_classify.rs)
- [`fuzz_typed_data_digest.rs`](fuzz/fuzz_targets/fuzz_typed_data_digest.rs)
- [`fuzz_valid_to_relative.rs`](fuzz/fuzz_targets/fuzz_valid_to_relative.rs)
- [`fuzz_vault_relayer_transfer_from_accounts_encode.rs`](fuzz/fuzz_targets/fuzz_vault_relayer_transfer_from_accounts_encode.rs)

</details>

<details>
<summary><code>parity/</code> &mdash; 9 file(s)</summary>

- [`cow-shed-invariants.md`](parity/cow-shed-invariants.md)
- [`ink-composable-rows.json`](parity/ink-composable-rows.json)
- [`ink-probe-results.json`](parity/ink-probe-results.json)
- [`lens-probe-results.json`](parity/lens-probe-results.json)
- [`npm-evidence.yaml`](parity/npm-evidence.yaml)
- [`optimism-probe-results.json`](parity/optimism-probe-results.json)
- [`README.md`](parity/README.md)
- [`self-pinning-allowlist.yaml`](parity/self-pinning-allowlist.yaml)
- [`source-lock.yaml`](parity/source-lock.yaml)

</details>

<details>
<summary><code>parity/dependency-audit/</code> &mdash; 1 file(s)</summary>

- [`alloy-runtime-baseline.md`](parity/dependency-audit/alloy-runtime-baseline.md)

</details>

<details>
<summary><code>parity/fixtures/</code> &mdash; 8 file(s)</summary>

- [`app-data.json`](parity/fixtures/app-data.json)
- [`contracts.json`](parity/fixtures/contracts.json)
- [`core.json`](parity/fixtures/core.json)
- [`orderbook.json`](parity/fixtures/orderbook.json)
- [`sdk.json`](parity/fixtures/sdk.json)
- [`signing.json`](parity/fixtures/signing.json)
- [`subgraph.json`](parity/fixtures/subgraph.json)
- [`trading.json`](parity/fixtures/trading.json)

</details>

<details>
<summary><code>parity/fixtures/app_data/</code> &mdash; 3 file(s)</summary>

- [`canonical_json_utf16.json`](parity/fixtures/app_data/canonical_json_utf16.json)
- [`flashloan_v1.7.0.json`](parity/fixtures/app_data/flashloan_v1.7.0.json)
- [`hooks_v1.14.0.json`](parity/fixtures/app_data/hooks_v1.14.0.json)

</details>

<details>
<summary><code>parity/fixtures/composable/</code> &mdash; 16 file(s)</summary>

- [`conditional_order_params_decode.json`](parity/fixtures/composable/conditional_order_params_decode.json)
- [`forwarder_signature_blob.json`](parity/fixtures/composable/forwarder_signature_blob.json)
- [`good_after_time_revert_sites.json`](parity/fixtures/composable/good_after_time_revert_sites.json)
- [`multiplexer_leaf.json`](parity/fixtures/composable/multiplexer_leaf.json)
- [`params_hash.json`](parity/fixtures/composable/params_hash.json)
- [`perpetual_stable_swap_overflow.json`](parity/fixtures/composable/perpetual_stable_swap_overflow.json)
- [`perpetual_stable_swap_revert_sites.json`](parity/fixtures/composable/perpetual_stable_swap_revert_sites.json)
- [`poll_result_classification.json`](parity/fixtures/composable/poll_result_classification.json)
- [`poll_result_selectors.json`](parity/fixtures/composable/poll_result_selectors.json)
- [`safe_muxer_signature_blob.json`](parity/fixtures/composable/safe_muxer_signature_blob.json)
- [`selectors.json`](parity/fixtures/composable/selectors.json)
- [`stop_loss_revert_sites.json`](parity/fixtures/composable/stop_loss_revert_sites.json)
- [`trade_above_threshold_revert_sites.json`](parity/fixtures/composable/trade_above_threshold_revert_sites.json)
- [`twap_merkle_leaf.json`](parity/fixtures/composable/twap_merkle_leaf.json)
- [`twap_order_id.json`](parity/fixtures/composable/twap_order_id.json)
- [`twap_static_input.json`](parity/fixtures/composable/twap_static_input.json)

</details>

<details>
<summary><code>parity/fixtures/cow_shed/</code> &mdash; 5 file(s)</summary>

- [`domain_separator.json`](parity/fixtures/cow_shed/domain_separator.json)
- [`eoa_signature_byte_order.json`](parity/fixtures/cow_shed/eoa_signature_byte_order.json)
- [`execute_hooks_calldata.json`](parity/fixtures/cow_shed/execute_hooks_calldata.json)
- [`execute_hooks_digest.json`](parity/fixtures/cow_shed/execute_hooks_digest.json)
- [`proxy_addresses.json`](parity/fixtures/cow_shed/proxy_addresses.json)

</details>

<details>
<summary><code>parity/fixtures/ecdsa/</code> &mdash; 1 file(s)</summary>

- [`v_normalization.json`](parity/fixtures/ecdsa/v_normalization.json)

</details>

<details>
<summary><code>parity/fixtures/eip712/</code> &mdash; 1 file(s)</summary>

- [`order_digests.json`](parity/fixtures/eip712/order_digests.json)

</details>

<details>
<summary><code>parity/fixtures/orderbook/</code> &mdash; 10 file(s)</summary>

- [`app_data_upload_response.json`](parity/fixtures/orderbook/app_data_upload_response.json)
- [`onchain_order_data.json`](parity/fixtures/orderbook/onchain_order_data.json)
- [`order_parameters.json`](parity/fixtures/orderbook/order_parameters.json)
- [`order_quote_response.json`](parity/fixtures/orderbook/order_quote_response.json)
- [`order_with_full_metadata.json`](parity/fixtures/orderbook/order_with_full_metadata.json)
- [`solver_competition_response.json`](parity/fixtures/orderbook/solver_competition_response.json)
- [`solver_execution.json`](parity/fixtures/orderbook/solver_execution.json)
- [`stored_order_quote.json`](parity/fixtures/orderbook/stored_order_quote.json)
- [`total_surplus.json`](parity/fixtures/orderbook/total_surplus.json)
- [`trade.json`](parity/fixtures/orderbook/trade.json)

</details>

<details>
<summary><code>parity/fixtures/orderbook-requests/</code> &mdash; 4 file(s)</summary>

- [`app_data_put.json`](parity/fixtures/orderbook-requests/app_data_put.json)
- [`order_cancellations.json`](parity/fixtures/orderbook-requests/order_cancellations.json)
- [`order_creation.json`](parity/fixtures/orderbook-requests/order_creation.json)
- [`order_quote_request.json`](parity/fixtures/orderbook-requests/order_quote_request.json)

</details>

<details>
<summary><code>parity/fixtures/retry_after/</code> &mdash; 3 file(s)</summary>

- [`imf_fixdate_accept.json`](parity/fixtures/retry_after/imf_fixdate_accept.json)
- [`imf_fixdate_reject.json`](parity/fixtures/retry_after/imf_fixdate_reject.json)
- [`legacy_rfc850.json`](parity/fixtures/retry_after/legacy_rfc850.json)

</details>

<details>
<summary><code>parity/fixtures/signing/</code> &mdash; 1 file(s)</summary>

- [`eth_sign_typed_data_request.json`](parity/fixtures/signing/eth_sign_typed_data_request.json)

</details>

<details>
<summary><code>parity/openapi/</code> &mdash; 10 file(s)</summary>

- [`coverage.yaml`](parity/openapi/coverage.yaml)
- [`onchain-order-data-inventory.yaml`](parity/openapi/onchain-order-data-inventory.yaml)
- [`order-inventory.yaml`](parity/openapi/order-inventory.yaml)
- [`order-parameters-inventory.yaml`](parity/openapi/order-parameters-inventory.yaml)
- [`order-quote-response-inventory.yaml`](parity/openapi/order-quote-response-inventory.yaml)
- [`services-orderbook.yml`](parity/openapi/services-orderbook.yml)
- [`solver-execution-inventory.yaml`](parity/openapi/solver-execution-inventory.yaml)
- [`stored-order-quote-inventory.yaml`](parity/openapi/stored-order-quote-inventory.yaml)
- [`total-surplus-inventory.yaml`](parity/openapi/total-surplus-inventory.yaml)
- [`trade-inventory.yaml`](parity/openapi/trade-inventory.yaml)

</details>

<details>
<summary><code>parity/source-lock/</code> &mdash; 1 file(s)</summary>

- [`npm-package-evidence.json`](parity/source-lock/npm-package-evidence.json)

</details>

<details>
<summary><code>scripts/</code> &mdash; 3 file(s)</summary>

- [`check-audit-index-agreement.sh`](scripts/check-audit-index-agreement.sh)
- [`check-release-docs-agree.sh`](scripts/check-release-docs-agree.sh)
- [`check-services-drift.sh`](scripts/check-services-drift.sh)

</details>

<details>
<summary><code>scripts/parity-maintainer/</code> &mdash; 2 file(s)</summary>

- [`Cargo.lock`](scripts/parity-maintainer/Cargo.lock)
- [`Cargo.toml`](scripts/parity-maintainer/Cargo.toml)

</details>

<details>
<summary><code>scripts/parity-maintainer/src/</code> &mdash; 13 file(s)</summary>

- [`audit_refresh.rs`](scripts/parity-maintainer/src/audit_refresh.rs)
- [`audit_self_pinning.rs`](scripts/parity-maintainer/src/audit_self_pinning.rs)
- [`check_freshness.rs`](scripts/parity-maintainer/src/check_freshness.rs)
- [`composable_fixtures.rs`](scripts/parity-maintainer/src/composable_fixtures.rs)
- [`cow_shed_fixtures.rs`](scripts/parity-maintainer/src/cow_shed_fixtures.rs)
- [`diff_upstreams.rs`](scripts/parity-maintainer/src/diff_upstreams.rs)
- [`main.rs`](scripts/parity-maintainer/src/main.rs)
- [`openapi_coverage.rs`](scripts/parity-maintainer/src/openapi_coverage.rs)
- [`stale_phrase_catalog.rs`](scripts/parity-maintainer/src/stale_phrase_catalog.rs)
- [`stale_phrase_lint.rs`](scripts/parity-maintainer/src/stale_phrase_lint.rs)
- [`url_provenance.rs`](scripts/parity-maintainer/src/url_provenance.rs)
- [`vendor_openapi.rs`](scripts/parity-maintainer/src/vendor_openapi.rs)
- [`verify_sol_provenance.rs`](scripts/parity-maintainer/src/verify_sol_provenance.rs)

</details>

<details>
<summary><code>scripts/parity-maintainer/tests/</code> &mdash; 11 file(s)</summary>

- [`audit_self_pinning.rs`](scripts/parity-maintainer/tests/audit_self_pinning.rs)
- [`check_freshness.rs`](scripts/parity-maintainer/tests/check_freshness.rs)
- [`diff_upstreams.rs`](scripts/parity-maintainer/tests/diff_upstreams.rs)
- [`enum_policy.rs`](scripts/parity-maintainer/tests/enum_policy.rs)
- [`openapi_coverage.rs`](scripts/parity-maintainer/tests/openapi_coverage.rs)
- [`producer_path_existence.rs`](scripts/parity-maintainer/tests/producer_path_existence.rs)
- [`README.md`](scripts/parity-maintainer/tests/README.md)
- [`source_lock_schema_version.rs`](scripts/parity-maintainer/tests/source_lock_schema_version.rs)
- [`stale_phrase_lint.rs`](scripts/parity-maintainer/tests/stale_phrase_lint.rs)
- [`url_provenance.rs`](scripts/parity-maintainer/tests/url_provenance.rs)
- [`vendor_openapi.rs`](scripts/parity-maintainer/tests/vendor_openapi.rs)

</details>

<details>
<summary><code>scripts/parity-maintainer/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](scripts/parity-maintainer/tests/common/mod.rs)

</details>

<details>
<summary><code>scripts/parity-maintainer/tests/fixtures/</code> &mdash; 3 file(s)</summary>

- [`source-lock-v2.yaml`](scripts/parity-maintainer/tests/fixtures/source-lock-v2.yaml)
- [`source-lock-v3.yaml`](scripts/parity-maintainer/tests/fixtures/source-lock-v3.yaml)
- [`source-lock-v4.yaml`](scripts/parity-maintainer/tests/fixtures/source-lock-v4.yaml)

</details>

<details>
<summary><code>scripts/policy-maintainer/</code> &mdash; 2 file(s)</summary>

- [`Cargo.lock`](scripts/policy-maintainer/Cargo.lock)
- [`Cargo.toml`](scripts/policy-maintainer/Cargo.toml)

</details>

<details>
<summary><code>scripts/policy-maintainer/src/</code> &mdash; 22 file(s)</summary>

- [`check_adr_coverage.rs`](scripts/policy-maintainer/src/check_adr_coverage.rs)
- [`check_alloy_provider_invariant.rs`](scripts/policy-maintainer/src/check_alloy_provider_invariant.rs)
- [`check_alloy_signer_invariant.rs`](scripts/policy-maintainer/src/check_alloy_signer_invariant.rs)
- [`check_chain_patch_eligibility.rs`](scripts/policy-maintainer/src/check_chain_patch_eligibility.rs)
- [`check_deny_unknown_fields.rs`](scripts/policy-maintainer/src/check_deny_unknown_fields.rs)
- [`check_enum_policy.rs`](scripts/policy-maintainer/src/check_enum_policy.rs)
- [`check_msrv_notice.rs`](scripts/policy-maintainer/src/check_msrv_notice.rs)
- [`check_panic_allowlist.rs`](scripts/policy-maintainer/src/check_panic_allowlist.rs)
- [`check_property_citations.rs`](scripts/policy-maintainer/src/check_property_citations.rs)
- [`check_source_lock_roots.rs`](scripts/policy-maintainer/src/check_source_lock_roots.rs)
- [`check_stub.rs`](scripts/policy-maintainer/src/check_stub.rs)
- [`check_wasm_invariant.rs`](scripts/policy-maintainer/src/check_wasm_invariant.rs)
- [`check_wasm_runner_freshness.rs`](scripts/policy-maintainer/src/check_wasm_runner_freshness.rs)
- [`check_workspace_versions.rs`](scripts/policy-maintainer/src/check_workspace_versions.rs)
- [`classify_release.rs`](scripts/policy-maintainer/src/classify_release.rs)
- [`diagnostics.rs`](scripts/policy-maintainer/src/diagnostics.rs)
- [`fixtures.rs`](scripts/policy-maintainer/src/fixtures.rs)
- [`generate_validation_evidence.rs`](scripts/policy-maintainer/src/generate_validation_evidence.rs)
- [`lib.rs`](scripts/policy-maintainer/src/lib.rs)
- [`main.rs`](scripts/policy-maintainer/src/main.rs)
- [`run_deterministic_examples.rs`](scripts/policy-maintainer/src/run_deterministic_examples.rs)
- [`workspace.rs`](scripts/policy-maintainer/src/workspace.rs)

</details>

<details>
<summary><code>scripts/policy-maintainer/tests/</code> &mdash; 13 file(s)</summary>

- [`check_adr_coverage.rs`](scripts/policy-maintainer/tests/check_adr_coverage.rs)
- [`check_alloy_provider_invariant.rs`](scripts/policy-maintainer/tests/check_alloy_provider_invariant.rs)
- [`check_alloy_signer_invariant.rs`](scripts/policy-maintainer/tests/check_alloy_signer_invariant.rs)
- [`check_chain_patch_eligibility.rs`](scripts/policy-maintainer/tests/check_chain_patch_eligibility.rs)
- [`check_deny_unknown_fields.rs`](scripts/policy-maintainer/tests/check_deny_unknown_fields.rs)
- [`check_enum_policy.rs`](scripts/policy-maintainer/tests/check_enum_policy.rs)
- [`check_msrv_notice.rs`](scripts/policy-maintainer/tests/check_msrv_notice.rs)
- [`check_panic_allowlist.rs`](scripts/policy-maintainer/tests/check_panic_allowlist.rs)
- [`check_property_citations.rs`](scripts/policy-maintainer/tests/check_property_citations.rs)
- [`check_wasm_runner_freshness.rs`](scripts/policy-maintainer/tests/check_wasm_runner_freshness.rs)
- [`check_workspace_versions.rs`](scripts/policy-maintainer/tests/check_workspace_versions.rs)
- [`classify_release.rs`](scripts/policy-maintainer/tests/classify_release.rs)
- [`generate_validation_evidence.rs`](scripts/policy-maintainer/tests/generate_validation_evidence.rs)

</details>

<details>
<summary><code>scripts/policy-maintainer/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](scripts/policy-maintainer/tests/common/mod.rs)

</details>

<details>
<summary><code>scripts/validation-depth/</code> &mdash; 3 file(s)</summary>

- [`Cargo.lock`](scripts/validation-depth/Cargo.lock)
- [`Cargo.toml`](scripts/validation-depth/Cargo.toml)
- [`README.md`](scripts/validation-depth/README.md)

</details>

<details>
<summary><code>scripts/validation-depth/src/</code> &mdash; 1 file(s)</summary>

- [`main.rs`](scripts/validation-depth/src/main.rs)

</details>

<details>
<summary><code>scripts/validation-smoke/</code> &mdash; 3 file(s)</summary>

- [`Cargo.lock`](scripts/validation-smoke/Cargo.lock)
- [`Cargo.toml`](scripts/validation-smoke/Cargo.toml)
- [`README.md`](scripts/validation-smoke/README.md)

</details>

<details>
<summary><code>scripts/validation-smoke/browser-wallet-live/</code> &mdash; 1 file(s)</summary>

- [`README.md`](scripts/validation-smoke/browser-wallet-live/README.md)

</details>

<details>
<summary><code>scripts/validation-smoke/data/</code> &mdash; 1 file(s)</summary>

- [`cft-fallback.json`](scripts/validation-smoke/data/cft-fallback.json)

</details>

<details>
<summary><code>scripts/validation-smoke/src/</code> &mdash; 4 file(s)</summary>

- [`lib.rs`](scripts/validation-smoke/src/lib.rs)
- [`main.rs`](scripts/validation-smoke/src/main.rs)
- [`registry_confirm.rs`](scripts/validation-smoke/src/registry_confirm.rs)
- [`wasm_runner.rs`](scripts/validation-smoke/src/wasm_runner.rs)

</details>

<details>
<summary><code>scripts/validation-smoke/tests/</code> &mdash; 2 file(s)</summary>

- [`registry_confirm.rs`](scripts/validation-smoke/tests/registry_confirm.rs)
- [`wasm_runner.rs`](scripts/validation-smoke/tests/wasm_runner.rs)

</details>

<details>
<summary><code>tests/</code> &mdash; 15 file(s)</summary>

- [`alloy_provider_invariant_covers_every_published_crate.rs`](tests/alloy_provider_invariant_covers_every_published_crate.rs)
- [`alloy_read_contract_parity_invariant.rs`](tests/alloy_read_contract_parity_invariant.rs)
- [`alloy_signer_invariant_covers_every_published_crate.rs`](tests/alloy_signer_invariant_covers_every_published_crate.rs)
- [`alloy_two_family_lockfile_invariant.rs`](tests/alloy_two_family_lockfile_invariant.rs)
- [`alloy_two_family_pin_lockstep.rs`](tests/alloy_two_family_pin_lockstep.rs)
- [`alloy_umbrella_composition.rs`](tests/alloy_umbrella_composition.rs)
- [`Cargo.toml`](tests/Cargo.toml)
- [`dependency_default_features_audit.rs`](tests/dependency_default_features_audit.rs)
- [`msrv_consistency.rs`](tests/msrv_consistency.rs)
- [`services_drift_report_schema.rs`](tests/services_drift_report_schema.rs)
- [`signer_rejection_propagation_invariant.rs`](tests/signer_rejection_propagation_invariant.rs)
- [`supported_chains_doc_table.rs`](tests/supported_chains_doc_table.rs)
- [`transaction_lifecycle_cross_adapter_invariant.rs`](tests/transaction_lifecycle_cross_adapter_invariant.rs)
- [`wasm_dependency_invariant.rs`](tests/wasm_dependency_invariant.rs)
- [`workspace_alloy_pin_lockstep.rs`](tests/workspace_alloy_pin_lockstep.rs)

</details>


