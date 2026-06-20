# Repository File Map

> **Branch:** `feat/ferrous-foundation` &nbsp;&middot;&nbsp; **HEAD:** `cd8c9ae7` &nbsp;&middot;&nbsp; **Generated:** 2026-06-20  
> **Total tracked files:** **886** &nbsp;&middot;&nbsp; **Lines of code:** tokei 14.0.0

A navigable inventory of every file tracked by Git on this branch, grouped by the role each directory plays in the workspace. Use the table of contents to jump straight to a section; full file listings are collapsed by default so the high-level shape stays scannable.

`Lines` are physical line counts measured directly from each file, so they match `wc -l` exactly. `Code` comes from `tokei 14.0.0`, which separates executable code from blanks, comments, and documentation; `Comments` is the remainder (`Lines − Code − Blank`); for Rust, the per-crate `Doc` column isolates doc-comments (`///` / `//!`) from inline comments. Data and config files (JSON fixtures, schemas, YAML, TOML, lockfiles) are counted in the per-language `Code` column too, but tracked separately from hand-written Rust/TypeScript so the SDK figures aren't distorted.

---

## Table of contents

1. [At a glance](#at-a-glance)
2. [Top-level layout](#top-level-layout)
3. [File composition by extension](#file-composition-by-extension)
4. [Workspace crates (`crates/`)](#workspace-crates-crates)
5. [Source hotspots](#source-hotspots)
6. [Examples (`examples/`)](#examples-examples)
7. [End-to-end harnesses (`e2e/`)](#end-to-end-harnesses-e2e)
8. [Upstream parity (`parity/`)](#upstream-parity-parity)
9. [Documentation (`docs/`)](#documentation-docs)
10. [Fuzzing (`fuzz/`)](#fuzzing-fuzz)
11. [CI & repo-level configuration](#ci--repo-level-configuration)
12. [Full file index](#full-file-index)

---

## At a glance

**Lines of code** (tracked files only)

- **32,427 lines of Rust** across the 14 SDK crates, covered by **36,346 lines of tests** — a **1.1× test-to-code ratio** — plus **179 lines of benchmarks**.
- **12,147 doc-comment lines** documenting the public API (~37.5% of crate code), plus **764 inline comment lines**.
- **5,443 lines of TypeScript** across examples, e2e harnesses, and wasm bindings.
- **15,805 lines of Markdown prose** — ADRs, audit notes, and READMEs.
- **14,067 lines of data & config** — JSON schemas, parity fixtures, YAML, TOML, and lockfiles. Tracked and counted in the totals below; listed separately here because it's data, not hand-written code.

**Footprint** (tracked files)

- **544 files** live under `crates/` — 14 workspace member crates make up roughly 61% of the repo.
- **96 files** under `docs/` are mostly architecture decision records and audit notes.
- **41 files** under `parity/` are golden fixtures captured from upstream services to keep the Rust SDK byte-compatible.
- **45 files** under `fuzz/` cover cargo-fuzz targets and their seed corpora.
- **60 files** under `examples/` + `e2e/` are runnable demos and integration harnesses.
- **46 files** under `xtask/` are the maintenance automation crate (parity refresh, policy checks, doc generation).

---

## Top-level layout

| Path | Files | Lines | Code | Purpose |
|------|------:|------:|-----:|---------|
| `crates/` | 544 | 112,261 | 76,478 | Workspace member crates (the SDK itself) |
| `docs/` | 96 | 15,158 | 0 | Architecture decision records, audit notes, provider notes |
| `xtask/` | 46 | 9,691 | 8,176 | Cargo xtask automation crate (parity, policy, docs subcommands) |
| `fuzz/` | 45 | 9,425 | 3,886 | cargo-fuzz targets, corpora, and failure artifacts |
| `parity/` | 41 | 5,828 | 5,517 | Golden fixtures + pinned specs from upstream services |
| `examples/` | 34 | 3,543 | 2,473 | Runnable usage examples (Rust + TypeScript) |
| `e2e/` | 26 | 3,716 | 3,002 | End-to-end integration harnesses |
| `.github/` | 23 | 3,261 | 2,772 | GitHub Actions workflows and repo config |
| `tests/` | 12 | 1,124 | 969 | Workspace-level integration tests |
| `.cargo/` | 2 | 32 | 27 | Cargo configuration |
| `SECURITY.md` | 1 | 182 | 0 | Security policy |
| `rust-toolchain.toml` | 1 | 6 | 4 | Pinned Rust toolchain |
| `ROADMAP.md` | 1 | 67 | 0 | Roadmap document |
| `release.toml` | 1 | 56 | 11 |  |
| `README.md` | 1 | 252 | 0 | Top-level README |
| `PROPERTIES.md` | 1 | 260 | 0 | Property-based testing index |
| `.gitattributes` | 1 | 41 | 0 | Git attributes |
| `LICENSE` | 1 | 674 | 0 | License text |
| `Cargo.lock` | 1 | 5,660 | 0 | Workspace lockfile |
| `.githooks/` | 1 | 35 | 28 | Tracked git hook scripts |
| `.gitignore` | 1 | 28 | 0 | Top-level git ignore rules |
| `.yamllint` | 1 | 7 | 0 | YAML lint configuration |
| `CONTRIBUTING.md` | 1 | 283 | 0 | Contribution guide |
| `cliff.toml` | 1 | 69 | 48 |  |
| `CHANGELOG.md` | 1 | 64 | 0 | Release changelog |
| `llvm-cov-summary.txt` | 1 | 186 | 0 | Coverage summary snapshot |
| `Cargo.toml` | 1 | 121 | 106 | Workspace manifest |
| **Total** | **886** | **172,030** | **103,497** | |

---

## File composition by extension

| Extension | Files | Lines | Code | Comments | Blank | Typical role |
|-----------|------:|------:|-----:|---------:|------:|--------------|
| `.rs` | 557 | 112,050 | 83,987 | 17,789 | 10,274 | Rust source and tests |
| `.md` | 121 | 19,116 | 0 | 15,805 | 3,311 | Markdown docs (ADRs, audit notes, READMEs) |
| `.ts` | 47 | 13,202 | 5,443 | 6,868 | 891 | TypeScript (examples, e2e, wasm bindings) |
| `.json` | 45 | 2,831 | 2,831 | 0 | 0 | JSON schemas, parity fixtures, test vectors |
| `.toml` | 28 | 1,909 | 1,506 | 153 | 250 | Cargo manifests and tool configs |
| `.stderr` | 25 | 570 | 0 | 549 | 21 | trybuild compile-fail snapshots |
| `.yml` | 16 | 5,138 | 4,717 | 271 | 150 | CI workflows and config |
| `.yaml` | 11 | 4,693 | 3,855 | 28 | 810 | CI workflows, OpenAPI specs, config |
| `.txt` | 7 | 227 | 0 | 226 | 1 | Plain text fixtures / summaries |
| `.mjs` | 6 | 810 | 614 | 100 | 96 | JavaScript modules |
| `.sh` | 4 | 555 | 480 | 11 | 64 | Shell scripts |
| `(none)` | 3 | 1,383 | 28 | 1,107 | 248 |  |
| `.graphql` | 3 | 24 | 24 | 0 | 0 | GraphQL queries (subgraph) |
| `.bin` | 2 | 0 | 0 | 0 | 0 | Binary fixtures |
| `.lock` | 2 | 9,416 | 0 | 8,508 | 908 | Cargo / package lockfiles |
| `.keep` | 2 | 2 | 0 | 0 | 2 |  |
| `.gitignore` | 2 | 31 | 0 | 26 | 5 |  |
| `.html` | 1 | 12 | 12 | 0 | 0 | Static HTML for browser examples |
| `.gitattributes` | 1 | 41 | 0 | 35 | 6 |  |
| `.yamllint` | 1 | 7 | 0 | 6 | 1 |  |
| `.proptest-regressions` | 1 | 7 | 0 | 7 | 0 | proptest regression seeds |
| `.npmignore` | 1 | 6 | 0 | 6 | 0 |  |
| **Total** | **886** | **172,030** | **103,497** | **51,495** | **17,038** | |

> **Code + Comments + Blank = Lines** for every row. ``Comments`` is all non-code, non-blank content: inline + doc-comments in source, prose in Markdown/text, and raw content in formats tokei does not parse as code (lockfiles, ``.stderr``, snapshots). Rust doc-comments are isolated in the per-crate ``Doc`` column above.

---

## Workspace crates (`crates/`)

14 member crates compose the SDK. `Code` is Rust `src/` code; `Tests` and `Benches` are Rust lines under `tests/` and `benches/`; `Doc` is `src/` doc-comment lines (`///` / `//!`) — the public-API documentation surface; `T:C` is the test-to-code ratio. Descriptions are pulled live from each crate's `Cargo.toml`.

| Crate | Files | Code | Tests | Benches | Doc | T:C | Purpose |
|-------|------:|-----:|------:|--------:|----:|----:|---------|
| [`wasm`](crates/wasm) | 114 | 6,378 | 4,899 | 0 | 1,772 | 0.8× | TypeScript-callable wasm-bindgen leaf for the CoW Protocol Rust SDK; built to wasm32 and shipped to npm, not crates.io |
| [`core`](crates/core) | 75 | 5,566 | 4,085 | 0 | 2,359 | 0.7× | Shared CoW Protocol core types and validation primitives |
| [`trading`](crates/trading) | 58 | 5,278 | 7,032 | 46 | 1,881 | 1.3× | High-level CoW Protocol trading orchestration surface |
| [`orderbook`](crates/orderbook) | 42 | 4,329 | 5,607 | 14 | 1,801 | 1.3× | Typed CoW Protocol orderbook client models and decoding helpers |
| [`contracts`](crates/contracts) | 59 | 2,912 | 3,901 | 60 | 1,576 | 1.3× | CoW Protocol low-level contracts helpers for order hashing, signature codecs and verification, ABI bindings, and fail-closed on-chain event decoding |
| [`app-data`](crates/app-data) | 41 | 1,385 | 2,186 | 33 | 750 | 1.6× | CoW Protocol app-data encoding, validation, and CID compatibility |
| [`alloy-provider`](crates/alloy-provider) | 27 | 1,294 | 1,516 | 0 | 203 | 1.2× | Alloy-backed read-only Provider adapter for the CoW Protocol Rust SDK |
| [`subgraph`](crates/subgraph) | 26 | 1,236 | 2,154 | 0 | 483 | 1.7× | Typed CoW Protocol subgraph query primitives |
| [`signing`](crates/signing) | 25 | 915 | 1,593 | 26 | 324 | 1.7× | Deterministic CoW Protocol order hashing, EIP-712 signing, and UID helpers |
| [`test-utils`](crates/test-utils) | 10 | 800 | 143 | 0 | 236 | 0.2× | Internal, unpublished shared test helpers for the cow-rs workspace. |
| [`alloy`](crates/alloy) | 27 | 777 | 1,265 | 0 | 209 | 1.6× | Composed Alloy provider and signer adapter for the CoW Protocol Rust SDK |
| [`alloy-signer`](crates/alloy-signer) | 23 | 726 | 534 | 0 | 157 | 0.7× | Alloy-backed local private-key Signer adapter for the CoW Protocol Rust SDK |
| [`test`](crates/test) | 9 | 725 | 283 | 0 | 228 | 0.4× | In-memory test doubles for the cow-rs SDK public traits (OrderbookClient, Signer, Provider) so downstream applications can test their CoW Protocol integration without a live orderbook, RPC endpoint, or wallet. |
| [`sdk`](crates/sdk) | 8 | 106 | 1,148 | 0 | 168 | 10.8× | Facade crate for CoW Protocol Rust SDK surfaces |
| **Total** | **544** | **32,427** | **36,346** | **179** | **12,147** | **1.1×** | |

---

## Source hotspots

The 25 largest hand-written source files by code lines (Rust + TypeScript). This is where complexity — and review attention — concentrates.

| File | Lang | Kind | Code | Comments |
|------|------|------|-----:|---------:|
| [`xtask/src/parity/mod.rs`](xtask/src/parity/mod.rs) | Rust | src | 1,122 | 111 |
| [`crates/orderbook/tests/api_contract.rs`](crates/orderbook/tests/api_contract.rs) | Rust | test | 1,032 | 23 |
| [`crates/subgraph/tests/api_contract.rs`](crates/subgraph/tests/api_contract.rs) | Rust | test | 1,012 | 6 |
| [`crates/orderbook/tests/request_contract.rs`](crates/orderbook/tests/request_contract.rs) | Rust | test | 856 | 21 |
| [`crates/trading/tests/common/mod.rs`](crates/trading/tests/common/mod.rs) | Rust | test | 855 | 36 |
| [`crates/trading/tests/quote_contract.rs`](crates/trading/tests/quote_contract.rs) | Rust | test | 786 | 13 |
| [`crates/trading/src/types/params.rs`](crates/trading/src/types/params.rs) | Rust | src | 778 | 271 |
| [`crates/sdk/tests/error_redaction_contract.rs`](crates/sdk/tests/error_redaction_contract.rs) | Rust | test | 773 | 58 |
| [`crates/trading/tests/post_contract.rs`](crates/trading/tests/post_contract.rs) | Rust | test | 759 | 74 |
| [`crates/orderbook/src/types/quote.rs`](crates/orderbook/src/types/quote.rs) | Rust | src | 736 | 297 |
| [`crates/wasm/npm/src/default.ts`](crates/wasm/npm/src/default.ts) | TypeScript | src | 702 | 11 |
| [`xtask/src/parity/openapi_coverage.rs`](xtask/src/parity/openapi_coverage.rs) | Rust | src | 686 | 5 |
| [`crates/wasm/src/exports/orderbook.rs`](crates/wasm/src/exports/orderbook.rs) | Rust | src | 680 | 193 |
| [`crates/wasm/snapshots/raw/default.d.ts`](crates/wasm/snapshots/raw/default.d.ts) | TypeScript | src | 679 | 2,339 |
| [`crates/wasm/src/exports/trading.rs`](crates/wasm/src/exports/trading.rs) | Rust | src | 679 | 165 |
| [`crates/wasm/snapshots/raw/trading.d.ts`](crates/wasm/snapshots/raw/trading.d.ts) | TypeScript | src | 645 | 2,228 |
| [`crates/orderbook/src/types/order.rs`](crates/orderbook/src/types/order.rs) | Rust | src | 629 | 249 |
| [`crates/core/tests/transport_contract.rs`](crates/core/tests/transport_contract.rs) | Rust | test | 613 | 27 |
| [`crates/trading/src/post.rs`](crates/trading/src/post.rs) | Rust | src | 604 | 127 |
| [`crates/trading/tests/sdk_contract.rs`](crates/trading/tests/sdk_contract.rs) | Rust | test | 601 | 8 |
| [`crates/trading/src/slippage.rs`](crates/trading/src/slippage.rs) | Rust | src | 595 | 121 |
| [`crates/wasm/npm/src/trading.ts`](crates/wasm/npm/src/trading.ts) | TypeScript | src | 589 | 11 |
| [`crates/wasm/src/exports/signing.rs`](crates/wasm/src/exports/signing.rs) | Rust | src | 581 | 124 |
| [`crates/core/tests/policy_contract.rs`](crates/core/tests/policy_contract.rs) | Rust | test | 580 | 64 |
| [`crates/orderbook/src/api.rs`](crates/orderbook/src/api.rs) | Rust | src | 578 | 251 |

---

## Examples (`examples/`)

| Example | Files | Lines | Code | Purpose |
|---------|------:|------:|-----:|---------|
| [`native`](examples/native) | 33 | 3,500 | 2,473 | Native Rust scenario walkthroughs |
| **Total (listed)** | **33** | **3,500** | **2,473** | |

---

## End-to-end harnesses (`e2e/`)

| Harness | Files | Lines | Code | Purpose |
|---------|------:|------:|-----:|---------|
| [`wasm-typescript`](e2e/wasm-typescript) | 14 | 1,815 | 1,470 | Wasm + TypeScript integration harness |
| [`wasm-typescript-cf`](e2e/wasm-typescript-cf) | 12 | 1,901 | 1,532 | Wasm + TypeScript Cloudflare harness |
| **Total (listed)** | **26** | **3,716** | **3,002** | |

---

## Upstream parity (`parity/`)

| Subtree | Files | Lines | Code | Purpose |
|---------|------:|------:|-----:|---------|
| [`fixtures`](parity/fixtures) | 37 | 2,588 | 2,588 | Golden fixtures captured from upstream services |
| [`openapi`](parity/openapi) | 2 | 2,895 | 2,860 | OpenAPI specs pinned for parity |
| **Total (listed)** | **39** | **5,483** | **5,448** | |

---

## Documentation (`docs/`)

| Subtree | Files | Lines | Code | Purpose |
|---------|------:|------:|-----:|---------|
| [`adr`](docs/adr) | 56 | 5,720 | 0 | Architecture Decision Records |
| [`audit`](docs/audit) | 19 | 3,877 | 0 | Audit notes and review artifacts |
| [`providers`](docs/providers) | 2 | 279 | 0 | Provider integration notes |
| **Total (listed)** | **77** | **9,876** | **0** | |

---

## Fuzzing (`fuzz/`)

| Subtree | Files | Lines | Code | Purpose |
|---------|------:|------:|-----:|---------|
| [`fuzz_targets`](fuzz/fuzz_targets) | 42 | 5,163 | 3,610 | cargo-fuzz target sources |
| **Total (listed)** | **42** | **5,163** | **3,610** | |

---

## CI & repo-level configuration

| Path | Files | Purpose |
|------|------:|---------|
| `.github/workflows/` | 13 | GitHub Actions pipelines |
| `.github/config/`    | 8 | Shared CI config |
| `.githooks/`         | 1 | Tracked git hooks |
| `.cargo/`            | 2 | Cargo config (e.g. rustflags) |
| `tests/`             | 12 | Workspace-level integration tests |

---

## Full file index

Every tracked file, grouped by the directory it lives in. Each section is collapsed by default — click to expand. The number after each file is its total line count.

<details>
<summary><code>(repo root)</code> &mdash; 16 file(s)</summary>

- [`.gitattributes`](.gitattributes) &mdash; 41 lines
- [`.gitignore`](.gitignore) &mdash; 28 lines
- [`.yamllint`](.yamllint) &mdash; 7 lines
- [`Cargo.lock`](Cargo.lock) &mdash; 5,660 lines
- [`Cargo.toml`](Cargo.toml) &mdash; 121 lines
- [`CHANGELOG.md`](CHANGELOG.md) &mdash; 64 lines
- [`cliff.toml`](cliff.toml) &mdash; 69 lines
- [`CONTRIBUTING.md`](CONTRIBUTING.md) &mdash; 283 lines
- [`LICENSE`](LICENSE) &mdash; 674 lines
- [`llvm-cov-summary.txt`](llvm-cov-summary.txt) &mdash; 186 lines
- [`PROPERTIES.md`](PROPERTIES.md) &mdash; 260 lines
- [`README.md`](README.md) &mdash; 252 lines
- [`release.toml`](release.toml) &mdash; 56 lines
- [`ROADMAP.md`](ROADMAP.md) &mdash; 67 lines
- [`rust-toolchain.toml`](rust-toolchain.toml) &mdash; 6 lines
- [`SECURITY.md`](SECURITY.md) &mdash; 182 lines

</details>

<details>
<summary><code>.cargo/</code> &mdash; 2 file(s)</summary>

- [`config.toml`](.cargo/config.toml) &mdash; 29 lines
- [`mutants.toml`](.cargo/mutants.toml) &mdash; 3 lines

</details>

<details>
<summary><code>.githooks/</code> &mdash; 1 file(s)</summary>

- [`commit-msg`](.githooks/commit-msg) &mdash; 35 lines

</details>

<details>
<summary><code>.github/</code> &mdash; 1 file(s)</summary>

- [`commit-template.md`](.github/commit-template.md) &mdash; 12 lines

</details>

<details>
<summary><code>.github/codeql/</code> &mdash; 1 file(s)</summary>

- [`codeql-config.yml`](.github/codeql/codeql-config.yml) &mdash; 25 lines

</details>

<details>
<summary><code>.github/config/</code> &mdash; 8 file(s)</summary>

- [`audit-refresh-map.yml`](.github/config/audit-refresh-map.yml) &mdash; 30 lines
- [`deny-unknown-fields-allowlist.yaml`](.github/config/deny-unknown-fields-allowlist.yaml) &mdash; 20 lines
- [`deny.toml`](.github/config/deny.toml) &mdash; 152 lines
- [`enum-policy.yaml`](.github/config/enum-policy.yaml) &mdash; 477 lines
- [`nextest.toml`](.github/config/nextest.toml) &mdash; 38 lines
- [`panic-allowlist.yaml`](.github/config/panic-allowlist.yaml) &mdash; 89 lines
- [`principle-adr-map.yaml`](.github/config/principle-adr-map.yaml) &mdash; 110 lines
- [`typos.toml`](.github/config/typos.toml) &mdash; 30 lines

</details>

<details>
<summary><code>.github/workflows/</code> &mdash; 13 file(s)</summary>

- [`_quality-gate.yml`](.github/workflows/_quality-gate.yml) &mdash; 349 lines
- [`alloy-release-candidate.yml`](.github/workflows/alloy-release-candidate.yml) &mdash; 133 lines
- [`benchmarks.yml`](.github/workflows/benchmarks.yml) &mdash; 69 lines
- [`ci.yml`](.github/workflows/ci.yml) &mdash; 315 lines
- [`codeql.yml`](.github/workflows/codeql.yml) &mdash; 55 lines
- [`commit-format.yml`](.github/workflows/commit-format.yml) &mdash; 98 lines
- [`crate-checks.yml`](.github/workflows/crate-checks.yml) &mdash; 99 lines
- [`docs-quality.yml`](.github/workflows/docs-quality.yml) &mdash; 90 lines
- [`fuzz.yml`](.github/workflows/fuzz.yml) &mdash; 79 lines
- [`release-readiness.yml`](.github/workflows/release-readiness.yml) &mdash; 325 lines
- [`retry-soak.yml`](.github/workflows/retry-soak.yml) &mdash; 35 lines
- [`upstream-drift.yml`](.github/workflows/upstream-drift.yml) &mdash; 40 lines
- [`wasm.yml`](.github/workflows/wasm.yml) &mdash; 591 lines

</details>

<details>
<summary><code>crates/alloy/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/alloy/Cargo.toml) &mdash; 57 lines
- [`README.md`](crates/alloy/README.md) &mdash; 151 lines

</details>

<details>
<summary><code>crates/alloy-provider/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/alloy-provider/Cargo.toml) &mdash; 50 lines
- [`README.md`](crates/alloy-provider/README.md) &mdash; 131 lines

</details>

<details>
<summary><code>crates/alloy-provider/src/</code> &mdash; 8 file(s)</summary>

- [`builder.rs`](crates/alloy-provider/src/builder.rs) &mdash; 198 lines
- [`client.rs`](crates/alloy-provider/src/client.rs) &mdash; 29 lines
- [`conversion.rs`](crates/alloy-provider/src/conversion.rs) &mdash; 326 lines
- [`error.rs`](crates/alloy-provider/src/error.rs) &mdash; 262 lines
- [`lib.rs`](crates/alloy-provider/src/lib.rs) &mdash; 154 lines
- [`provider.rs`](crates/alloy-provider/src/provider.rs) &mdash; 147 lines
- [`read_contract.rs`](crates/alloy-provider/src/read_contract.rs) &mdash; 460 lines
- [`retry.rs`](crates/alloy-provider/src/retry.rs) &mdash; 87 lines

</details>

<details>
<summary><code>crates/alloy-provider/tests/</code> &mdash; 11 file(s)</summary>

- [`builder_contract.rs`](crates/alloy-provider/tests/builder_contract.rs) &mdash; 134 lines
- [`cancellation_contract.rs`](crates/alloy-provider/tests/cancellation_contract.rs) &mdash; 18 lines
- [`compile_fail.rs`](crates/alloy-provider/tests/compile_fail.rs) &mdash; 7 lines
- [`dependency_boundary_contract.rs`](crates/alloy-provider/tests/dependency_boundary_contract.rs) &mdash; 50 lines
- [`error_class_contract.rs`](crates/alloy-provider/tests/error_class_contract.rs) &mdash; 224 lines
- [`provider_contract.rs`](crates/alloy-provider/tests/provider_contract.rs) &mdash; 299 lines
- [`read_contract_no_panic.rs`](crates/alloy-provider/tests/read_contract_no_panic.rs) &mdash; 75 lines
- [`read_contract_parity.rs`](crates/alloy-provider/tests/read_contract_parity.rs) &mdash; 636 lines
- [`redaction_contract.rs`](crates/alloy-provider/tests/redaction_contract.rs) &mdash; 123 lines
- [`retry_contract.rs`](crates/alloy-provider/tests/retry_contract.rs) &mdash; 69 lines
- [`seam_contract.rs`](crates/alloy-provider/tests/seam_contract.rs) &mdash; 260 lines

</details>

<details>
<summary><code>crates/alloy-provider/tests/trybuild/</code> &mdash; 6 file(s)</summary>

- [`external_marker_construction_fails.rs`](crates/alloy-provider/tests/trybuild/external_marker_construction_fails.rs) &mdash; 9 lines
- [`external_marker_construction_fails.stderr`](crates/alloy-provider/tests/trybuild/external_marker_construction_fails.stderr) &mdash; 13 lines
- [`no_signer.rs`](crates/alloy-provider/tests/trybuild/no_signer.rs) &mdash; 7 lines
- [`no_signer.stderr`](crates/alloy-provider/tests/trybuild/no_signer.stderr) &mdash; 11 lines
- [`no_signing_provider.rs`](crates/alloy-provider/tests/trybuild/no_signing_provider.rs) &mdash; 7 lines
- [`no_signing_provider.stderr`](crates/alloy-provider/tests/trybuild/no_signing_provider.stderr) &mdash; 11 lines

</details>

<details>
<summary><code>crates/alloy-signer/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/alloy-signer/Cargo.toml) &mdash; 48 lines
- [`README.md`](crates/alloy-signer/README.md) &mdash; 136 lines

</details>

<details>
<summary><code>crates/alloy-signer/src/</code> &mdash; 5 file(s)</summary>

- [`builder.rs`](crates/alloy-signer/src/builder.rs) &mdash; 291 lines
- [`conversion.rs`](crates/alloy-signer/src/conversion.rs) &mdash; 295 lines
- [`error.rs`](crates/alloy-signer/src/error.rs) &mdash; 212 lines
- [`lib.rs`](crates/alloy-signer/src/lib.rs) &mdash; 65 lines
- [`signer.rs`](crates/alloy-signer/src/signer.rs) &mdash; 142 lines

</details>

<details>
<summary><code>crates/alloy-signer/tests/</code> &mdash; 9 file(s)</summary>

- [`cancellation_contract.rs`](crates/alloy-signer/tests/cancellation_contract.rs) &mdash; 28 lines
- [`compile_fail.rs`](crates/alloy-signer/tests/compile_fail.rs) &mdash; 9 lines
- [`dependency_boundary_contract.rs`](crates/alloy-signer/tests/dependency_boundary_contract.rs) &mdash; 55 lines
- [`eip191_reference_vectors.rs`](crates/alloy-signer/tests/eip191_reference_vectors.rs) &mdash; 42 lines
- [`eip712_reference_vectors.rs`](crates/alloy-signer/tests/eip712_reference_vectors.rs) &mdash; 40 lines
- [`proptests.rs`](crates/alloy-signer/tests/proptests.rs) &mdash; 102 lines
- [`redaction_contract.rs`](crates/alloy-signer/tests/redaction_contract.rs) &mdash; 133 lines
- [`signer_contract.rs`](crates/alloy-signer/tests/signer_contract.rs) &mdash; 127 lines
- [`signer_error_trait_contract.rs`](crates/alloy-signer/tests/signer_error_trait_contract.rs) &mdash; 33 lines

</details>

<details>
<summary><code>crates/alloy-signer/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/alloy-signer/tests/common/mod.rs) &mdash; 20 lines

</details>

<details>
<summary><code>crates/alloy-signer/tests/trybuild/</code> &mdash; 6 file(s)</summary>

- [`external_marker_construction_fails.rs`](crates/alloy-signer/tests/trybuild/external_marker_construction_fails.rs) &mdash; 32 lines
- [`external_marker_construction_fails.stderr`](crates/alloy-signer/tests/trybuild/external_marker_construction_fails.stderr) &mdash; 25 lines
- [`no_provider.rs`](crates/alloy-signer/tests/trybuild/no_provider.rs) &mdash; 15 lines
- [`no_provider.stderr`](crates/alloy-signer/tests/trybuild/no_provider.stderr) &mdash; 13 lines
- [`no_signing_provider.rs`](crates/alloy-signer/tests/trybuild/no_signing_provider.rs) &mdash; 15 lines
- [`no_signing_provider.stderr`](crates/alloy-signer/tests/trybuild/no_signing_provider.stderr) &mdash; 13 lines

</details>

<details>
<summary><code>crates/alloy/src/</code> &mdash; 6 file(s)</summary>

- [`builder.rs`](crates/alloy/src/builder.rs) &mdash; 387 lines
- [`client.rs`](crates/alloy/src/client.rs) &mdash; 231 lines
- [`conversion.rs`](crates/alloy/src/conversion.rs) &mdash; 17 lines
- [`error.rs`](crates/alloy/src/error.rs) &mdash; 257 lines
- [`handle.rs`](crates/alloy/src/handle.rs) &mdash; 131 lines
- [`lib.rs`](crates/alloy/src/lib.rs) &mdash; 72 lines

</details>

<details>
<summary><code>crates/alloy/tests/</code> &mdash; 15 file(s)</summary>

- [`builder_contract.rs`](crates/alloy/tests/builder_contract.rs) &mdash; 282 lines
- [`cancellation_contract.rs`](crates/alloy/tests/cancellation_contract.rs) &mdash; 28 lines
- [`chain_coherence_mismatch.rs`](crates/alloy/tests/chain_coherence_mismatch.rs) &mdash; 93 lines
- [`chain_coherence.rs`](crates/alloy/tests/chain_coherence.rs) &mdash; 35 lines
- [`compile_fail.rs`](crates/alloy/tests/compile_fail.rs) &mdash; 8 lines
- [`eip712_reference_vectors.rs`](crates/alloy/tests/eip712_reference_vectors.rs) &mdash; 92 lines
- [`error_contract.rs`](crates/alloy/tests/error_contract.rs) &mdash; 207 lines
- [`handle_survives_drop.rs`](crates/alloy/tests/handle_survives_drop.rs) &mdash; 32 lines
- [`log_provider_contract.rs`](crates/alloy/tests/log_provider_contract.rs) &mdash; 40 lines
- [`provider_contract.rs`](crates/alloy/tests/provider_contract.rs) &mdash; 197 lines
- [`read_contract_contract.rs`](crates/alloy/tests/read_contract_contract.rs) &mdash; 129 lines
- [`redaction_contract.rs`](crates/alloy/tests/redaction_contract.rs) &mdash; 182 lines
- [`send_transaction_does_not_wait_for_confirmation.rs`](crates/alloy/tests/send_transaction_does_not_wait_for_confirmation.rs) &mdash; 151 lines
- [`signer_error_trait_contract.rs`](crates/alloy/tests/signer_error_trait_contract.rs) &mdash; 39 lines
- [`signing_provider_contract.rs`](crates/alloy/tests/signing_provider_contract.rs) &mdash; 37 lines

</details>

<details>
<summary><code>crates/alloy/tests/trybuild/</code> &mdash; 4 file(s)</summary>

- [`no_provider_on_handle.rs`](crates/alloy/tests/trybuild/no_provider_on_handle.rs) &mdash; 8 lines
- [`no_provider_on_handle.stderr`](crates/alloy/tests/trybuild/no_provider_on_handle.stderr) &mdash; 21 lines
- [`no_signer_on_client.rs`](crates/alloy/tests/trybuild/no_signer_on_client.rs) &mdash; 8 lines
- [`no_signer_on_client.stderr`](crates/alloy/tests/trybuild/no_signer_on_client.stderr) &mdash; 21 lines

</details>

<details>
<summary><code>crates/app-data/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/app-data/Cargo.toml) &mdash; 50 lines
- [`README.md`](crates/app-data/README.md) &mdash; 161 lines

</details>

<details>
<summary><code>crates/app-data/benches/</code> &mdash; 1 file(s)</summary>

- [`stringify.rs`](crates/app-data/benches/stringify.rs) &mdash; 38 lines

</details>

<details>
<summary><code>crates/app-data/src/</code> &mdash; 6 file(s)</summary>

- [`cid.rs`](crates/app-data/src/cid.rs) &mdash; 143 lines
- [`errors.rs`](crates/app-data/src/errors.rs) &mdash; 217 lines
- [`fetch.rs`](crates/app-data/src/fetch.rs) &mdash; 211 lines
- [`info.rs`](crates/app-data/src/info.rs) &mdash; 349 lines
- [`lib.rs`](crates/app-data/src/lib.rs) &mdash; 72 lines
- [`schema.rs`](crates/app-data/src/schema.rs) &mdash; 158 lines

</details>

<details>
<summary><code>crates/app-data/src/metadata/</code> &mdash; 4 file(s)</summary>

- [`flashloan.rs`](crates/app-data/src/metadata/flashloan.rs) &mdash; 107 lines
- [`hooks.rs`](crates/app-data/src/metadata/hooks.rs) &mdash; 80 lines
- [`mod.rs`](crates/app-data/src/metadata/mod.rs) &mdash; 18 lines
- [`quote.rs`](crates/app-data/src/metadata/quote.rs) &mdash; 97 lines

</details>

<details>
<summary><code>crates/app-data/src/types/</code> &mdash; 6 file(s)</summary>

- [`doc.rs`](crates/app-data/src/types/doc.rs) &mdash; 126 lines
- [`ipfs.rs`](crates/app-data/src/types/ipfs.rs) &mdash; 13 lines
- [`mod.rs`](crates/app-data/src/types/mod.rs) &mdash; 19 lines
- [`params.rs`](crates/app-data/src/types/params.rs) &mdash; 327 lines
- [`partner_fee.rs`](crates/app-data/src/types/partner_fee.rs) &mdash; 385 lines
- [`validation.rs`](crates/app-data/src/types/validation.rs) &mdash; 31 lines

</details>

<details>
<summary><code>crates/app-data/tests/</code> &mdash; 18 file(s)</summary>

- [`app_data_info_contract.rs`](crates/app-data/tests/app_data_info_contract.rs) &mdash; 44 lines
- [`canonical_json_contract.rs`](crates/app-data/tests/canonical_json_contract.rs) &mdash; 44 lines
- [`cid_contract.rs`](crates/app-data/tests/cid_contract.rs) &mdash; 105 lines
- [`error_contract.rs`](crates/app-data/tests/error_contract.rs) &mdash; 13 lines
- [`error_variant_shape.rs`](crates/app-data/tests/error_variant_shape.rs) &mdash; 97 lines
- [`fetch_contract.rs`](crates/app-data/tests/fetch_contract.rs) &mdash; 243 lines
- [`fetch_telemetry_contract.rs`](crates/app-data/tests/fetch_telemetry_contract.rs) &mdash; 83 lines
- [`flashloan_contract.rs`](crates/app-data/tests/flashloan_contract.rs) &mdash; 295 lines
- [`hooks_contract.rs`](crates/app-data/tests/hooks_contract.rs) &mdash; 159 lines
- [`ipfs_config_redaction_contract.rs`](crates/app-data/tests/ipfs_config_redaction_contract.rs) &mdash; 52 lines
- [`json_recursion_contract.rs`](crates/app-data/tests/json_recursion_contract.rs) &mdash; 24 lines
- [`metadata_signer_contract.rs`](crates/app-data/tests/metadata_signer_contract.rs) &mdash; 171 lines
- [`partner_fee_contract.rs`](crates/app-data/tests/partner_fee_contract.rs) &mdash; 412 lines
- [`property_contract.rs`](crates/app-data/tests/property_contract.rs) &mdash; 326 lines
- [`schema_contract.rs`](crates/app-data/tests/schema_contract.rs) &mdash; 158 lines
- [`schema_drift_contract.rs`](crates/app-data/tests/schema_drift_contract.rs) &mdash; 218 lines
- [`typed_metadata_contract.rs`](crates/app-data/tests/typed_metadata_contract.rs) &mdash; 24 lines
- [`validated_shape_contract.rs`](crates/app-data/tests/validated_shape_contract.rs) &mdash; 142 lines

</details>

<details>
<summary><code>crates/app-data/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/app-data/tests/common/mod.rs) &mdash; 41 lines

</details>

<details>
<summary><code>crates/app-data/tests/proptest-regressions/</code> &mdash; 1 file(s)</summary>

- [`property_contract.txt`](crates/app-data/tests/proptest-regressions/property_contract.txt) &mdash; 6 lines

</details>

<details>
<summary><code>crates/app-data/tests/ui/</code> &mdash; 2 file(s)</summary>

- [`partner_fee_bps_width_witness.rs`](crates/app-data/tests/ui/partner_fee_bps_width_witness.rs) &mdash; 27 lines
- [`partner_fee_bps_width_witness.stderr`](crates/app-data/tests/ui/partner_fee_bps_width_witness.stderr) &mdash; 5 lines

</details>

<details>
<summary><code>crates/contracts/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/contracts/Cargo.toml) &mdash; 62 lines
- [`README.md`](crates/contracts/README.md) &mdash; 110 lines

</details>

<details>
<summary><code>crates/contracts/benches/</code> &mdash; 2 file(s)</summary>

- [`order_hashing.rs`](crates/contracts/benches/order_hashing.rs) &mdash; 27 lines
- [`uid_packing.rs`](crates/contracts/benches/uid_packing.rs) &mdash; 42 lines

</details>

<details>
<summary><code>crates/contracts/src/</code> &mdash; 13 file(s)</summary>

- [`deployments.rs`](crates/contracts/src/deployments.rs) &mdash; 420 lines
- [`errors.rs`](crates/contracts/src/errors.rs) &mdash; 216 lines
- [`eth_flow.rs`](crates/contracts/src/eth_flow.rs) &mdash; 617 lines
- [`hex_field.rs`](crates/contracts/src/hex_field.rs) &mdash; 234 lines
- [`interaction.rs`](crates/contracts/src/interaction.rs) &mdash; 116 lines
- [`lib.rs`](crates/contracts/src/lib.rs) &mdash; 85 lines
- [`onchain_orders.rs`](crates/contracts/src/onchain_orders.rs) &mdash; 326 lines
- [`order.rs`](crates/contracts/src/order.rs) &mdash; 529 lines
- [`primitives.rs`](crates/contracts/src/primitives.rs) &mdash; 254 lines
- [`settlement.rs`](crates/contracts/src/settlement.rs) &mdash; 258 lines
- [`signature.rs`](crates/contracts/src/signature.rs) &mdash; 609 lines
- [`tokens.rs`](crates/contracts/src/tokens.rs) &mdash; 98 lines
- [`verify.rs`](crates/contracts/src/verify.rs) &mdash; 244 lines

</details>

<details>
<summary><code>crates/contracts/src/cow_shed/</code> &mdash; 8 file(s)</summary>

- [`bindings.rs`](crates/contracts/src/cow_shed/bindings.rs) &mdash; 163 lines
- [`calls.rs`](crates/contracts/src/cow_shed/calls.rs) &mdash; 66 lines
- [`eip712.rs`](crates/contracts/src/cow_shed/eip712.rs) &mdash; 168 lines
- [`errors.rs`](crates/contracts/src/cow_shed/errors.rs) &mdash; 23 lines
- [`hooks.rs`](crates/contracts/src/cow_shed/hooks.rs) &mdash; 285 lines
- [`mod.rs`](crates/contracts/src/cow_shed/mod.rs) &mdash; 77 lines
- [`types.rs`](crates/contracts/src/cow_shed/types.rs) &mdash; 46 lines
- [`version.rs`](crates/contracts/src/cow_shed/version.rs) &mdash; 49 lines

</details>

<details>
<summary><code>crates/contracts/src/cow_shed/address/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/contracts/src/cow_shed/address/mod.rs) &mdash; 132 lines

</details>

<details>
<summary><code>crates/contracts/src/cow_shed/address/proxy-creation-code/</code> &mdash; 2 file(s)</summary>

- [`v1.0.0.bin`](crates/contracts/src/cow_shed/address/proxy-creation-code/v1.0.0.bin)
- [`v1.0.1.bin`](crates/contracts/src/cow_shed/address/proxy-creation-code/v1.0.1.bin)

</details>

<details>
<summary><code>crates/contracts/tests/</code> &mdash; 25 file(s)</summary>

- [`deployment_address_parity_contract.rs`](crates/contracts/tests/deployment_address_parity_contract.rs) &mdash; 58 lines
- [`domain_separator_parity_contract.rs`](crates/contracts/tests/domain_separator_parity_contract.rs) &mdash; 35 lines
- [`eip712_message_hash_parity_contract.rs`](crates/contracts/tests/eip712_message_hash_parity_contract.rs) &mdash; 113 lines
- [`eip712_type_hash_parity_contract.rs`](crates/contracts/tests/eip712_type_hash_parity_contract.rs) &mdash; 66 lines
- [`eoa_signature_byte_order_contract.rs`](crates/contracts/tests/eoa_signature_byte_order_contract.rs) &mdash; 77 lines
- [`error_contract.rs`](crates/contracts/tests/error_contract.rs) &mdash; 197 lines
- [`eth_flow_events_contract.rs`](crates/contracts/tests/eth_flow_events_contract.rs) &mdash; 143 lines
- [`interaction_contract.rs`](crates/contracts/tests/interaction_contract.rs) &mdash; 79 lines
- [`non_exhaustive_dto_contract.rs`](crates/contracts/tests/non_exhaustive_dto_contract.rs) &mdash; 119 lines
- [`onchain_orders.rs`](crates/contracts/tests/onchain_orders.rs) &mdash; 296 lines
- [`order_contract.rs`](crates/contracts/tests/order_contract.rs) &mdash; 126 lines
- [`order_digest_parity_contract.rs`](crates/contracts/tests/order_digest_parity_contract.rs) &mdash; 156 lines
- [`parity_contract.rs`](crates/contracts/tests/parity_contract.rs) &mdash; 556 lines
- [`property_contract.rs`](crates/contracts/tests/property_contract.rs) &mdash; 441 lines
- [`proxy_address_parity_contract.rs`](crates/contracts/tests/proxy_address_parity_contract.rs) &mdash; 134 lines
- [`recoverable_signature_contract.rs`](crates/contracts/tests/recoverable_signature_contract.rs) &mdash; 307 lines
- [`selector_parity_cow_shed_contract.rs`](crates/contracts/tests/selector_parity_cow_shed_contract.rs) &mdash; 148 lines
- [`settlement_events_contract.rs`](crates/contracts/tests/settlement_events_contract.rs) &mdash; 199 lines
- [`sign_telemetry_contract.rs`](crates/contracts/tests/sign_telemetry_contract.rs) &mdash; 58 lines
- [`signature_contract.rs`](crates/contracts/tests/signature_contract.rs) &mdash; 464 lines
- [`signed_calldata_parity_contract.rs`](crates/contracts/tests/signed_calldata_parity_contract.rs) &mdash; 164 lines
- [`tokens_contract.rs`](crates/contracts/tests/tokens_contract.rs) &mdash; 235 lines
- [`ui.rs`](crates/contracts/tests/ui.rs) &mdash; 11 lines
- [`v_normalization_contract.rs`](crates/contracts/tests/v_normalization_contract.rs) &mdash; 105 lines
- [`verify_telemetry_contract.rs`](crates/contracts/tests/verify_telemetry_contract.rs) &mdash; 204 lines

</details>

<details>
<summary><code>crates/contracts/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/contracts/tests/common/mod.rs) &mdash; 152 lines

</details>

<details>
<summary><code>crates/contracts/tests/cow_shed_common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/contracts/tests/cow_shed_common/mod.rs) &mdash; 49 lines

</details>

<details>
<summary><code>crates/contracts/tests/ui/</code> &mdash; 4 file(s)</summary>

- [`non_exhaustive_external_match.rs`](crates/contracts/tests/ui/non_exhaustive_external_match.rs) &mdash; 30 lines
- [`non_exhaustive_external_match.stderr`](crates/contracts/tests/ui/non_exhaustive_external_match.stderr) &mdash; 56 lines
- [`typestate_marker_sealing.rs`](crates/contracts/tests/ui/typestate_marker_sealing.rs) &mdash; 26 lines
- [`typestate_marker_sealing.stderr`](crates/contracts/tests/ui/typestate_marker_sealing.stderr) &mdash; 143 lines

</details>

<details>
<summary><code>crates/core/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/core/Cargo.toml) &mdash; 83 lines
- [`README.md`](crates/core/README.md) &mdash; 126 lines

</details>

<details>
<summary><code>crates/core/src/</code> &mdash; 4 file(s)</summary>

- [`cancellation.rs`](crates/core/src/cancellation.rs) &mdash; 122 lines
- [`errors.rs`](crates/core/src/errors.rs) &mdash; 250 lines
- [`lib.rs`](crates/core/src/lib.rs) &mdash; 158 lines
- [`validation.rs`](crates/core/src/validation.rs) &mdash; 116 lines

</details>

<details>
<summary><code>crates/core/src/config/</code> &mdash; 6 file(s)</summary>

- [`chains.rs`](crates/core/src/config/chains.rs) &mdash; 222 lines
- [`env.rs`](crates/core/src/config/env.rs) &mdash; 71 lines
- [`hosts.rs`](crates/core/src/config/hosts.rs) &mdash; 236 lines
- [`http.rs`](crates/core/src/config/http.rs) &mdash; 122 lines
- [`mod.rs`](crates/core/src/config/mod.rs) &mdash; 36 lines
- [`protocol.rs`](crates/core/src/config/protocol.rs) &mdash; 166 lines

</details>

<details>
<summary><code>crates/core/src/redaction/</code> &mdash; 3 file(s)</summary>

- [`body.rs`](crates/core/src/redaction/body.rs) &mdash; 397 lines
- [`mod.rs`](crates/core/src/redaction/mod.rs) &mdash; 21 lines
- [`wrappers.rs`](crates/core/src/redaction/wrappers.rs) &mdash; 327 lines

</details>

<details>
<summary><code>crates/core/src/traits/</code> &mdash; 5 file(s)</summary>

- [`mod.rs`](crates/core/src/traits/mod.rs) &mdash; 8 lines
- [`provider.rs`](crates/core/src/traits/provider.rs) &mdash; 238 lines
- [`signer.rs`](crates/core/src/traits/signer.rs) &mdash; 219 lines
- [`transaction.rs`](crates/core/src/traits/transaction.rs) &mdash; 209 lines
- [`typed_data.rs`](crates/core/src/traits/typed_data.rs) &mdash; 184 lines

</details>

<details>
<summary><code>crates/core/src/transport/</code> &mdash; 5 file(s)</summary>

- [`error.rs`](crates/core/src/transport/error.rs) &mdash; 77 lines
- [`fetch.rs`](crates/core/src/transport/fetch.rs) &mdash; 438 lines
- [`http.rs`](crates/core/src/transport/http.rs) &mdash; 284 lines
- [`mod.rs`](crates/core/src/transport/mod.rs) &mdash; 186 lines
- [`reqwest.rs`](crates/core/src/transport/reqwest.rs) &mdash; 508 lines

</details>

<details>
<summary><code>crates/core/src/transport/policy/</code> &mdash; 10 file(s)</summary>

- [`classify.rs`](crates/core/src/transport/policy/classify.rs) &mdash; 46 lines
- [`config.rs`](crates/core/src/transport/policy/config.rs) &mdash; 298 lines
- [`jitter.rs`](crates/core/src/transport/policy/jitter.rs) &mdash; 163 lines
- [`mod.rs`](crates/core/src/transport/policy/mod.rs) &mdash; 55 lines
- [`rate_limit.rs`](crates/core/src/transport/policy/rate_limit.rs) &mdash; 295 lines
- [`retry_after.rs`](crates/core/src/transport/policy/retry_after.rs) &mdash; 134 lines
- [`retry.rs`](crates/core/src/transport/policy/retry.rs) &mdash; 231 lines
- [`runner.rs`](crates/core/src/transport/policy/runner.rs) &mdash; 528 lines
- [`status.rs`](crates/core/src/transport/policy/status.rs) &mdash; 47 lines
- [`time.rs`](crates/core/src/transport/policy/time.rs) &mdash; 66 lines

</details>

<details>
<summary><code>crates/core/src/types/</code> &mdash; 8 file(s)</summary>

- [`amount.rs`](crates/core/src/types/amount.rs) &mdash; 506 lines
- [`app_code.rs`](crates/core/src/types/app_code.rs) &mdash; 206 lines
- [`identity.rs`](crates/core/src/types/identity.rs) &mdash; 917 lines
- [`logs.rs`](crates/core/src/types/logs.rs) &mdash; 281 lines
- [`mod.rs`](crates/core/src/types/mod.rs) &mdash; 72 lines
- [`order.rs`](crates/core/src/types/order.rs) &mdash; 207 lines
- [`quote.rs`](crates/core/src/types/quote.rs) &mdash; 156 lines
- [`validity.rs`](crates/core/src/types/validity.rs) &mdash; 101 lines

</details>

<details>
<summary><code>crates/core/tests/</code> &mdash; 21 file(s)</summary>

- [`address_literal_ui.rs`](crates/core/tests/address_literal_ui.rs) &mdash; 17 lines
- [`amount_arithmetic_ui.rs`](crates/core/tests/amount_arithmetic_ui.rs) &mdash; 30 lines
- [`cancellation_contract.rs`](crates/core/tests/cancellation_contract.rs) &mdash; 126 lines
- [`cancellation_coverage_validator.rs`](crates/core/tests/cancellation_coverage_validator.rs) &mdash; 232 lines
- [`classify_contract.rs`](crates/core/tests/classify_contract.rs) &mdash; 48 lines
- [`config_contract.rs`](crates/core/tests/config_contract.rs) &mdash; 239 lines
- [`policy_contract.rs`](crates/core/tests/policy_contract.rs) &mdash; 731 lines
- [`property_contract.rs`](crates/core/tests/property_contract.rs) &mdash; 585 lines
- [`provider_capability_split_contract.rs`](crates/core/tests/provider_capability_split_contract.rs) &mdash; 250 lines
- [`redaction_contract.rs`](crates/core/tests/redaction_contract.rs) &mdash; 209 lines
- [`retry_after_contract.proptest-regressions`](crates/core/tests/retry_after_contract.proptest-regressions) &mdash; 7 lines
- [`retry_after_contract.rs`](crates/core/tests/retry_after_contract.rs) &mdash; 295 lines
- [`retry_after_fixture_contract.rs`](crates/core/tests/retry_after_fixture_contract.rs) &mdash; 118 lines
- [`supported_networks_parity.rs`](crates/core/tests/supported_networks_parity.rs) &mdash; 69 lines
- [`token_balance_parity.rs`](crates/core/tests/token_balance_parity.rs) &mdash; 89 lines
- [`token_balance_ui.rs`](crates/core/tests/token_balance_ui.rs) &mdash; 20 lines
- [`traits_contract.rs`](crates/core/tests/traits_contract.rs) &mdash; 480 lines
- [`transport_contract.rs`](crates/core/tests/transport_contract.rs) &mdash; 726 lines
- [`types_contract.rs`](crates/core/tests/types_contract.rs) &mdash; 607 lines
- [`wasm_sleep_zero_delay_contract.rs`](crates/core/tests/wasm_sleep_zero_delay_contract.rs) &mdash; 35 lines
- [`wire_format_preservation_contract.rs`](crates/core/tests/wire_format_preservation_contract.rs) &mdash; 294 lines

</details>

<details>
<summary><code>crates/core/tests/proptest-regressions/</code> &mdash; 1 file(s)</summary>

- [`property_contract.txt`](crates/core/tests/proptest-regressions/property_contract.txt) &mdash; 9 lines

</details>

<details>
<summary><code>crates/core/tests/ui/</code> &mdash; 10 file(s)</summary>

- [`address_literal_empty.rs`](crates/core/tests/ui/address_literal_empty.rs) &mdash; 9 lines
- [`address_literal_empty.stderr`](crates/core/tests/ui/address_literal_empty.stderr) &mdash; 11 lines
- [`address_literal_non_string.rs`](crates/core/tests/ui/address_literal_non_string.rs) &mdash; 8 lines
- [`address_literal_non_string.stderr`](crates/core/tests/ui/address_literal_non_string.stderr) &mdash; 26 lines
- [`amount_arithmetic_operators_removed.rs`](crates/core/tests/ui/amount_arithmetic_operators_removed.rs) &mdash; 19 lines
- [`amount_arithmetic_operators_removed.stderr`](crates/core/tests/ui/amount_arithmetic_operators_removed.stderr) &mdash; 47 lines
- [`amount_string_conversion_rejected.rs`](crates/core/tests/ui/amount_string_conversion_rejected.rs) &mdash; 15 lines
- [`amount_string_conversion_rejected.stderr`](crates/core/tests/ui/amount_string_conversion_rejected.stderr) &mdash; 27 lines
- [`token_balance_split_cross_side.rs`](crates/core/tests/ui/token_balance_split_cross_side.rs) &mdash; 43 lines
- [`token_balance_split_cross_side.stderr`](crates/core/tests/ui/token_balance_split_cross_side.stderr) &mdash; 22 lines

</details>

<details>
<summary><code>crates/orderbook/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/orderbook/Cargo.toml) &mdash; 50 lines
- [`README.md`](crates/orderbook/README.md) &mdash; 117 lines

</details>

<details>
<summary><code>crates/orderbook/benches/</code> &mdash; 1 file(s)</summary>

- [`quote_cost.rs`](crates/orderbook/benches/quote_cost.rs) &mdash; 17 lines

</details>

<details>
<summary><code>crates/orderbook/src/</code> &mdash; 7 file(s)</summary>

- [`api.rs`](crates/orderbook/src/api.rs) &mdash; 891 lines
- [`builder.rs`](crates/orderbook/src/builder.rs) &mdash; 480 lines
- [`error.rs`](crates/orderbook/src/error.rs) &mdash; 585 lines
- [`lib.rs`](crates/orderbook/src/lib.rs) &mdash; 295 lines
- [`rejection.rs`](crates/orderbook/src/rejection.rs) &mdash; 575 lines
- [`request.rs`](crates/orderbook/src/request.rs) &mdash; 710 lines
- [`transform.rs`](crates/orderbook/src/transform.rs) &mdash; 79 lines

</details>

<details>
<summary><code>crates/orderbook/src/types/</code> &mdash; 8 file(s)</summary>

- [`app_data.rs`](crates/orderbook/src/types/app_data.rs) &mdash; 159 lines
- [`auction.rs`](crates/orderbook/src/types/auction.rs) &mdash; 345 lines
- [`enums.rs`](crates/orderbook/src/types/enums.rs) &mdash; 163 lines
- [`lists.rs`](crates/orderbook/src/types/lists.rs) &mdash; 194 lines
- [`mod.rs`](crates/orderbook/src/types/mod.rs) &mdash; 117 lines
- [`order.rs`](crates/orderbook/src/types/order.rs) &mdash; 926 lines
- [`prices.rs`](crates/orderbook/src/types/prices.rs) &mdash; 60 lines
- [`quote.rs`](crates/orderbook/src/types/quote.rs) &mdash; 1,110 lines

</details>

<details>
<summary><code>crates/orderbook/tests/</code> &mdash; 16 file(s)</summary>

- [`api_contract.rs`](crates/orderbook/tests/api_contract.rs) &mdash; 1,184 lines
- [`builder_contract.rs`](crates/orderbook/tests/builder_contract.rs) &mdash; 251 lines
- [`cancellation_composition_contract.rs`](crates/orderbook/tests/cancellation_composition_contract.rs) &mdash; 477 lines
- [`ecdsa_scheme_conversion_contract.rs`](crates/orderbook/tests/ecdsa_scheme_conversion_contract.rs) &mdash; 61 lines
- [`error_variant_shape.rs`](crates/orderbook/tests/error_variant_shape.rs) &mdash; 112 lines
- [`fee_amount_is_not_a_public_builder_setter.rs`](crates/orderbook/tests/fee_amount_is_not_a_public_builder_setter.rs) &mdash; 198 lines
- [`host_policy_contract.rs`](crates/orderbook/tests/host_policy_contract.rs) &mdash; 112 lines
- [`invariant_contract.rs`](crates/orderbook/tests/invariant_contract.rs) &mdash; 326 lines
- [`order_creation_fee_deserialize.rs`](crates/orderbook/tests/order_creation_fee_deserialize.rs) &mdash; 153 lines
- [`quote_echo_contract.rs`](crates/orderbook/tests/quote_echo_contract.rs) &mdash; 469 lines
- [`rejection_category_contract.rs`](crates/orderbook/tests/rejection_category_contract.rs) &mdash; 81 lines
- [`rejection_contract.rs`](crates/orderbook/tests/rejection_contract.rs) &mdash; 635 lines
- [`request_contract.rs`](crates/orderbook/tests/request_contract.rs) &mdash; 963 lines
- [`transform_contract.rs`](crates/orderbook/tests/transform_contract.rs) &mdash; 435 lines
- [`types_contract.rs`](crates/orderbook/tests/types_contract.rs) &mdash; 600 lines
- [`wire_contract.rs`](crates/orderbook/tests/wire_contract.rs) &mdash; 277 lines

</details>

<details>
<summary><code>crates/orderbook/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/orderbook/tests/common/mod.rs) &mdash; 241 lines

</details>

<details>
<summary><code>crates/orderbook/tests/proptest-regressions/</code> &mdash; 1 file(s)</summary>

- [`order_creation_fee_deserialize.txt`](crates/orderbook/tests/proptest-regressions/order_creation_fee_deserialize.txt) &mdash; 6 lines

</details>

<details>
<summary><code>crates/orderbook/tests/ui/</code> &mdash; 6 file(s)</summary>

- [`build_on_empty_builder.rs`](crates/orderbook/tests/ui/build_on_empty_builder.rs) &mdash; 9 lines
- [`build_on_empty_builder.stderr`](crates/orderbook/tests/ui/build_on_empty_builder.stderr) &mdash; 9 lines
- [`build_without_chain.rs`](crates/orderbook/tests/ui/build_without_chain.rs) &mdash; 10 lines
- [`build_without_chain.stderr`](crates/orderbook/tests/ui/build_without_chain.stderr) &mdash; 9 lines
- [`build_without_environment.rs`](crates/orderbook/tests/ui/build_without_environment.rs) &mdash; 12 lines
- [`build_without_environment.stderr`](crates/orderbook/tests/ui/build_without_environment.stderr) &mdash; 14 lines

</details>

<details>
<summary><code>crates/sdk/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/sdk/Cargo.toml) &mdash; 84 lines
- [`README.md`](crates/sdk/README.md) &mdash; 173 lines

</details>

<details>
<summary><code>crates/sdk/src/</code> &mdash; 1 file(s)</summary>

- [`lib.rs`](crates/sdk/src/lib.rs) &mdash; 300 lines

</details>

<details>
<summary><code>crates/sdk/tests/</code> &mdash; 5 file(s)</summary>

- [`error_class_contract.rs`](crates/sdk/tests/error_class_contract.rs) &mdash; 285 lines
- [`error_redaction_contract.rs`](crates/sdk/tests/error_redaction_contract.rs) &mdash; 898 lines
- [`public_api_default_features_only.rs`](crates/sdk/tests/public_api_default_features_only.rs) &mdash; 25 lines
- [`public_api_with_all_features.rs`](crates/sdk/tests/public_api_with_all_features.rs) &mdash; 26 lines
- [`public_api.rs`](crates/sdk/tests/public_api.rs) &mdash; 121 lines

</details>

<details>
<summary><code>crates/signing/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/signing/Cargo.toml) &mdash; 63 lines
- [`README.md`](crates/signing/README.md) &mdash; 142 lines

</details>

<details>
<summary><code>crates/signing/benches/</code> &mdash; 1 file(s)</summary>

- [`typed_data.rs`](crates/signing/benches/typed_data.rs) &mdash; 30 lines

</details>

<details>
<summary><code>crates/signing/src/</code> &mdash; 6 file(s)</summary>

- [`cache.rs`](crates/signing/src/cache.rs) &mdash; 239 lines
- [`cancellation.rs`](crates/signing/src/cancellation.rs) &mdash; 195 lines
- [`domain.rs`](crates/signing/src/domain.rs) &mdash; 211 lines
- [`errors.rs`](crates/signing/src/errors.rs) &mdash; 75 lines
- [`lib.rs`](crates/signing/src/lib.rs) &mdash; 52 lines
- [`order_signing.rs`](crates/signing/src/order_signing.rs) &mdash; 469 lines

</details>

<details>
<summary><code>crates/signing/src/eip1271/</code> &mdash; 4 file(s)</summary>

- [`error.rs`](crates/signing/src/eip1271/error.rs) &mdash; 27 lines
- [`mod.rs`](crates/signing/src/eip1271/mod.rs) &mdash; 9 lines
- [`provider.rs`](crates/signing/src/eip1271/provider.rs) &mdash; 44 lines
- [`sol_types.rs`](crates/signing/src/eip1271/sol_types.rs) &mdash; 59 lines

</details>

<details>
<summary><code>crates/signing/tests/</code> &mdash; 8 file(s)</summary>

- [`cancellation_contract.rs`](crates/signing/tests/cancellation_contract.rs) &mdash; 193 lines
- [`domain_contract.rs`](crates/signing/tests/domain_contract.rs) &mdash; 106 lines
- [`eip1271_cache_contract.rs`](crates/signing/tests/eip1271_cache_contract.rs) &mdash; 569 lines
- [`eip1271_contract.rs`](crates/signing/tests/eip1271_contract.rs) &mdash; 145 lines
- [`order_signing_contract.rs`](crates/signing/tests/order_signing_contract.rs) &mdash; 305 lines
- [`property_contract.rs`](crates/signing/tests/property_contract.rs) &mdash; 468 lines
- [`ui.rs`](crates/signing/tests/ui.rs) &mdash; 5 lines
- [`wasm_cache_contract.rs`](crates/signing/tests/wasm_cache_contract.rs) &mdash; 65 lines

</details>

<details>
<summary><code>crates/signing/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/signing/tests/common/mod.rs) &mdash; 17 lines

</details>

<details>
<summary><code>crates/signing/tests/proptest-regressions/</code> &mdash; 1 file(s)</summary>

- [`property_contract.txt`](crates/signing/tests/proptest-regressions/property_contract.txt) &mdash; 6 lines

</details>

<details>
<summary><code>crates/signing/tests/ui/</code> &mdash; 2 file(s)</summary>

- [`eip1271_error_match_requires_wildcard.rs`](crates/signing/tests/ui/eip1271_error_match_requires_wildcard.rs) &mdash; 31 lines
- [`eip1271_error_match_requires_wildcard.stderr`](crates/signing/tests/ui/eip1271_error_match_requires_wildcard.stderr) &mdash; 18 lines

</details>

<details>
<summary><code>crates/subgraph/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/subgraph/Cargo.toml) &mdash; 42 lines
- [`README.md`](crates/subgraph/README.md) &mdash; 95 lines

</details>

<details>
<summary><code>crates/subgraph/src/</code> &mdash; 6 file(s)</summary>

- [`api.rs`](crates/subgraph/src/api.rs) &mdash; 734 lines
- [`builder.rs`](crates/subgraph/src/builder.rs) &mdash; 402 lines
- [`error.rs`](crates/subgraph/src/error.rs) &mdash; 358 lines
- [`lib.rs`](crates/subgraph/src/lib.rs) &mdash; 36 lines
- [`queries.rs`](crates/subgraph/src/queries.rs) &mdash; 12 lines
- [`types.rs`](crates/subgraph/src/types.rs) &mdash; 320 lines

</details>

<details>
<summary><code>crates/subgraph/src/query_documents/</code> &mdash; 3 file(s)</summary>

- [`last_days_volume.graphql`](crates/subgraph/src/query_documents/last_days_volume.graphql) &mdash; 6 lines
- [`last_hours_volume.graphql`](crates/subgraph/src/query_documents/last_hours_volume.graphql) &mdash; 6 lines
- [`totals.graphql`](crates/subgraph/src/query_documents/totals.graphql) &mdash; 12 lines

</details>

<details>
<summary><code>crates/subgraph/tests/</code> &mdash; 8 file(s)</summary>

- [`api_contract.rs`](crates/subgraph/tests/api_contract.rs) &mdash; 1,128 lines
- [`builder_contract.rs`](crates/subgraph/tests/builder_contract.rs) &mdash; 233 lines
- [`cancellation_composition_contract.rs`](crates/subgraph/tests/cancellation_composition_contract.rs) &mdash; 206 lines
- [`error_contract.rs`](crates/subgraph/tests/error_contract.rs) &mdash; 255 lines
- [`error_redaction_contract.rs`](crates/subgraph/tests/error_redaction_contract.rs) &mdash; 104 lines
- [`host_policy_contract.rs`](crates/subgraph/tests/host_policy_contract.rs) &mdash; 105 lines
- [`query_contract.rs`](crates/subgraph/tests/query_contract.rs) &mdash; 175 lines
- [`types_contract.rs`](crates/subgraph/tests/types_contract.rs) &mdash; 184 lines

</details>

<details>
<summary><code>crates/subgraph/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/subgraph/tests/common/mod.rs) &mdash; 51 lines

</details>

<details>
<summary><code>crates/subgraph/tests/ui/</code> &mdash; 6 file(s)</summary>

- [`build_on_empty_builder.rs`](crates/subgraph/tests/ui/build_on_empty_builder.rs) &mdash; 9 lines
- [`build_on_empty_builder.stderr`](crates/subgraph/tests/ui/build_on_empty_builder.stderr) &mdash; 9 lines
- [`build_without_api_key.rs`](crates/subgraph/tests/ui/build_without_api_key.rs) &mdash; 12 lines
- [`build_without_api_key.stderr`](crates/subgraph/tests/ui/build_without_api_key.stderr) &mdash; 14 lines
- [`build_without_chain.rs`](crates/subgraph/tests/ui/build_without_chain.rs) &mdash; 9 lines
- [`build_without_chain.stderr`](crates/subgraph/tests/ui/build_without_chain.stderr) &mdash; 9 lines

</details>

<details>
<summary><code>crates/test/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/test/Cargo.toml) &mdash; 30 lines
- [`README.md`](crates/test/README.md) &mdash; 83 lines

</details>

<details>
<summary><code>crates/test-utils/</code> &mdash; 1 file(s)</summary>

- [`Cargo.toml`](crates/test-utils/Cargo.toml) &mdash; 35 lines

</details>

<details>
<summary><code>crates/test-utils/src/</code> &mdash; 8 file(s)</summary>

- [`arb.rs`](crates/test-utils/src/arb.rs) &mdash; 55 lines
- [`builders.rs`](crates/test-utils/src/builders.rs) &mdash; 173 lines
- [`consts.rs`](crates/test-utils/src/consts.rs) &mdash; 14 lines
- [`eip712.rs`](crates/test-utils/src/eip712.rs) &mdash; 110 lines
- [`fixtures.rs`](crates/test-utils/src/fixtures.rs) &mdash; 67 lines
- [`lib.rs`](crates/test-utils/src/lib.rs) &mdash; 20 lines
- [`mocks.rs`](crates/test-utils/src/mocks.rs) &mdash; 350 lines
- [`trace.rs`](crates/test-utils/src/trace.rs) &mdash; 361 lines

</details>

<details>
<summary><code>crates/test-utils/tests/</code> &mdash; 1 file(s)</summary>

- [`smoke.rs`](crates/test-utils/tests/smoke.rs) &mdash; 170 lines

</details>

<details>
<summary><code>crates/test/src/</code> &mdash; 6 file(s)</summary>

- [`defaults.rs`](crates/test/src/defaults.rs) &mdash; 97 lines
- [`error.rs`](crates/test/src/error.rs) &mdash; 95 lines
- [`lib.rs`](crates/test/src/lib.rs) &mdash; 98 lines
- [`orderbook.rs`](crates/test/src/orderbook.rs) &mdash; 220 lines
- [`provider.rs`](crates/test/src/provider.rs) &mdash; 235 lines
- [`signer.rs`](crates/test/src/signer.rs) &mdash; 332 lines

</details>

<details>
<summary><code>crates/test/tests/</code> &mdash; 1 file(s)</summary>

- [`contract.rs`](crates/test/tests/contract.rs) &mdash; 339 lines

</details>

<details>
<summary><code>crates/trading/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/trading/Cargo.toml) &mdash; 70 lines
- [`README.md`](crates/trading/README.md) &mdash; 282 lines

</details>

<details>
<summary><code>crates/trading/benches/</code> &mdash; 1 file(s)</summary>

- [`order_build.rs`](crates/trading/benches/order_build.rs) &mdash; 51 lines

</details>

<details>
<summary><code>crates/trading/src/</code> &mdash; 14 file(s)</summary>

- [`allowance.rs`](crates/trading/src/allowance.rs) &mdash; 150 lines
- [`app_data.rs`](crates/trading/src/app_data.rs) &mdash; 275 lines
- [`cancel.rs`](crates/trading/src/cancel.rs) &mdash; 65 lines
- [`error.rs`](crates/trading/src/error.rs) &mdash; 245 lines
- [`lib.rs`](crates/trading/src/lib.rs) &mdash; 117 lines
- [`onchain.rs`](crates/trading/src/onchain.rs) &mdash; 481 lines
- [`order.rs`](crates/trading/src/order.rs) &mdash; 337 lines
- [`params.rs`](crates/trading/src/params.rs) &mdash; 109 lines
- [`post.rs`](crates/trading/src/post.rs) &mdash; 780 lines
- [`quote.rs`](crates/trading/src/quote.rs) &mdash; 429 lines
- [`slippage.rs`](crates/trading/src/slippage.rs) &mdash; 782 lines
- [`validation.rs`](crates/trading/src/validation.rs) &mdash; 267 lines
- [`wait.rs`](crates/trading/src/wait.rs) &mdash; 393 lines
- [`wrap.rs`](crates/trading/src/wrap.rs) &mdash; 93 lines

</details>

<details>
<summary><code>crates/trading/src/client/</code> &mdash; 6 file(s)</summary>

- [`builder.rs`](crates/trading/src/client/builder.rs) &mdash; 265 lines
- [`helpers.rs`](crates/trading/src/client/helpers.rs) &mdash; 152 lines
- [`limit.rs`](crates/trading/src/client/limit.rs) &mdash; 288 lines
- [`methods.rs`](crates/trading/src/client/methods.rs) &mdash; 597 lines
- [`mod.rs`](crates/trading/src/client/mod.rs) &mdash; 151 lines
- [`swap.rs`](crates/trading/src/client/swap.rs) &mdash; 330 lines

</details>

<details>
<summary><code>crates/trading/src/types/</code> &mdash; 4 file(s)</summary>

- [`mod.rs`](crates/trading/src/types/mod.rs) &mdash; 16 lines
- [`params.rs`](crates/trading/src/types/params.rs) &mdash; 1,147 lines
- [`result.rs`](crates/trading/src/types/result.rs) &mdash; 256 lines
- [`seams.rs`](crates/trading/src/types/seams.rs) &mdash; 102 lines

</details>

<details>
<summary><code>crates/trading/tests/</code> &mdash; 24 file(s)</summary>

- [`allowance_contract.rs`](crates/trading/tests/allowance_contract.rs) &mdash; 145 lines
- [`app_code_contract.rs`](crates/trading/tests/app_code_contract.rs) &mdash; 43 lines
- [`app_data_merge_contract.rs`](crates/trading/tests/app_data_merge_contract.rs) &mdash; 661 lines
- [`cancel_contract.rs`](crates/trading/tests/cancel_contract.rs) &mdash; 87 lines
- [`cancellation_composition_contract.rs`](crates/trading/tests/cancellation_composition_contract.rs) &mdash; 519 lines
- [`error_variant_shape.rs`](crates/trading/tests/error_variant_shape.rs) &mdash; 113 lines
- [`invariant_contract.rs`](crates/trading/tests/invariant_contract.rs) &mdash; 516 lines
- [`limit_from_quote_contract.rs`](crates/trading/tests/limit_from_quote_contract.rs) &mdash; 103 lines
- [`limit_lifecycle_contract.rs`](crates/trading/tests/limit_lifecycle_contract.rs) &mdash; 118 lines
- [`onchain_contract.rs`](crates/trading/tests/onchain_contract.rs) &mdash; 422 lines
- [`order_contract.rs`](crates/trading/tests/order_contract.rs) &mdash; 185 lines
- [`parameters_contract.rs`](crates/trading/tests/parameters_contract.rs) &mdash; 133 lines
- [`post_contract.rs`](crates/trading/tests/post_contract.rs) &mdash; 930 lines
- [`property_contract.rs`](crates/trading/tests/property_contract.rs) &mdash; 212 lines
- [`quote_contract.rs`](crates/trading/tests/quote_contract.rs) &mdash; 863 lines
- [`quote_projection_parity.rs`](crates/trading/tests/quote_projection_parity.rs) &mdash; 154 lines
- [`sdk_contract.rs`](crates/trading/tests/sdk_contract.rs) &mdash; 672 lines
- [`slippage_contract.rs`](crates/trading/tests/slippage_contract.rs) &mdash; 250 lines
- [`swap_lifecycle_contract.rs`](crates/trading/tests/swap_lifecycle_contract.rs) &mdash; 147 lines
- [`types_contract.rs`](crates/trading/tests/types_contract.rs) &mdash; 300 lines
- [`ui.rs`](crates/trading/tests/ui.rs) &mdash; 11 lines
- [`validation_contract.rs`](crates/trading/tests/validation_contract.rs) &mdash; 342 lines
- [`wait_helper_contract.rs`](crates/trading/tests/wait_helper_contract.rs) &mdash; 190 lines
- [`wait_telemetry_contract.rs`](crates/trading/tests/wait_telemetry_contract.rs) &mdash; 85 lines

</details>

<details>
<summary><code>crates/trading/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/trading/tests/common/mod.rs) &mdash; 1,007 lines

</details>

<details>
<summary><code>crates/trading/tests/proptest-regressions/</code> &mdash; 2 file(s)</summary>

- [`invariant_contract.txt`](crates/trading/tests/proptest-regressions/invariant_contract.txt) &mdash; 8 lines
- [`property_contract.txt`](crates/trading/tests/proptest-regressions/property_contract.txt) &mdash; 6 lines

</details>

<details>
<summary><code>crates/trading/tests/ui/</code> &mdash; 4 file(s)</summary>

- [`client_rejection_external_match_requires_wildcard.rs`](crates/trading/tests/ui/client_rejection_external_match_requires_wildcard.rs) &mdash; 18 lines
- [`client_rejection_external_match_requires_wildcard.stderr`](crates/trading/tests/ui/client_rejection_external_match_requires_wildcard.stderr) &mdash; 18 lines
- [`trading_sdk_no_free_constructors.rs`](crates/trading/tests/ui/trading_sdk_no_free_constructors.rs) &mdash; 9 lines
- [`trading_sdk_no_free_constructors.stderr`](crates/trading/tests/ui/trading_sdk_no_free_constructors.stderr) &mdash; 5 lines

</details>

<details>
<summary><code>crates/wasm/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/wasm/Cargo.toml) &mdash; 104 lines
- [`README.md`](crates/wasm/README.md) &mdash; 157 lines

</details>

<details>
<summary><code>crates/wasm/npm/</code> &mdash; 10 file(s)</summary>

- [`.gitignore`](crates/wasm/npm/.gitignore) &mdash; 3 lines
- [`.npmignore`](crates/wasm/npm/.npmignore) &mdash; 6 lines
- [`flavours.json`](crates/wasm/npm/flavours.json) &mdash; 77 lines
- [`LICENSE`](crates/wasm/npm/LICENSE) &mdash; 674 lines
- [`package.template.json`](crates/wasm/npm/package.template.json) &mdash; 54 lines
- [`pnpm-lock.yaml`](crates/wasm/npm/pnpm-lock.yaml) &mdash; 771 lines
- [`README.md`](crates/wasm/npm/README.md) &mdash; 325 lines
- [`tsconfig.facade.json`](crates/wasm/npm/tsconfig.facade.json) &mdash; 5 lines
- [`tsconfig.json`](crates/wasm/npm/tsconfig.json) &mdash; 24 lines
- [`vitest.config.ts`](crates/wasm/npm/vitest.config.ts) &mdash; 9 lines

</details>

<details>
<summary><code>crates/wasm/npm/scripts/</code> &mdash; 10 file(s)</summary>

- [`build.sh`](crates/wasm/npm/scripts/build.sh) &mdash; 212 lines
- [`compile-facade.sh`](crates/wasm/npm/scripts/compile-facade.sh) &mdash; 252 lines
- [`dedupe-target-wasm.mjs`](crates/wasm/npm/scripts/dedupe-target-wasm.mjs) &mdash; 171 lines
- [`measure-wasm-size.mjs`](crates/wasm/npm/scripts/measure-wasm-size.mjs) &mdash; 175 lines
- [`pack-and-resolve-tarball.sh`](crates/wasm/npm/scripts/pack-and-resolve-tarball.sh) &mdash; 22 lines
- [`render-package-json.mjs`](crates/wasm/npm/scripts/render-package-json.mjs) &mdash; 177 lines
- [`verify-exports.mjs`](crates/wasm/npm/scripts/verify-exports.mjs) &mdash; 150 lines
- [`verify-facade-denylist.mjs`](crates/wasm/npm/scripts/verify-facade-denylist.mjs) &mdash; 81 lines
- [`verify-no-raw-exports.mjs`](crates/wasm/npm/scripts/verify-no-raw-exports.mjs) &mdash; 56 lines
- [`verify-package-resolution.sh`](crates/wasm/npm/scripts/verify-package-resolution.sh) &mdash; 69 lines

</details>

<details>
<summary><code>crates/wasm/npm/src/</code> &mdash; 10 file(s)</summary>

- [`callbacks.ts`](crates/wasm/npm/src/callbacks.ts) &mdash; 113 lines
- [`default.ts`](crates/wasm/npm/src/default.ts) &mdash; 800 lines
- [`envelope.ts`](crates/wasm/npm/src/envelope.ts) &mdash; 6 lines
- [`errors.ts`](crates/wasm/npm/src/errors.ts) &mdash; 382 lines
- [`index.ts`](crates/wasm/npm/src/index.ts) &mdash; 1 lines
- [`internal.ts`](crates/wasm/npm/src/internal.ts) &mdash; 160 lines
- [`options.ts`](crates/wasm/npm/src/options.ts) &mdash; 79 lines
- [`orderbook.ts`](crates/wasm/npm/src/orderbook.ts) &mdash; 458 lines
- [`signing.ts`](crates/wasm/npm/src/signing.ts) &mdash; 178 lines
- [`trading.ts`](crates/wasm/npm/src/trading.ts) &mdash; 670 lines

</details>

<details>
<summary><code>crates/wasm/npm/src/raw/</code> &mdash; 4 file(s)</summary>

- [`default.ts`](crates/wasm/npm/src/raw/default.ts) &mdash; 43 lines
- [`orderbook.ts`](crates/wasm/npm/src/raw/orderbook.ts) &mdash; 35 lines
- [`signing.ts`](crates/wasm/npm/src/raw/signing.ts) &mdash; 28 lines
- [`trading.ts`](crates/wasm/npm/src/raw/trading.ts) &mdash; 40 lines

</details>

<details>
<summary><code>crates/wasm/npm/tests/</code> &mdash; 8 file(s)</summary>

- [`facade-cancellation.test.ts`](crates/wasm/npm/tests/facade-cancellation.test.ts) &mdash; 28 lines
- [`facade-default.test.ts`](crates/wasm/npm/tests/facade-default.test.ts) &mdash; 34 lines
- [`facade-error-normalization.test.ts`](crates/wasm/npm/tests/facade-error-normalization.test.ts) &mdash; 154 lines
- [`facade-error-shape.test.ts`](crates/wasm/npm/tests/facade-error-shape.test.ts) &mdash; 65 lines
- [`facade-orderbook.test.ts`](crates/wasm/npm/tests/facade-orderbook.test.ts) &mdash; 20 lines
- [`facade-resource-cleanup.test.ts`](crates/wasm/npm/tests/facade-resource-cleanup.test.ts) &mdash; 42 lines
- [`facade-signing.test.ts`](crates/wasm/npm/tests/facade-signing.test.ts) &mdash; 19 lines
- [`fixtures.ts`](crates/wasm/npm/tests/fixtures.ts) &mdash; 34 lines

</details>

<details>
<summary><code>crates/wasm/snapshots/facade/</code> &mdash; 5 file(s)</summary>

- [`.keep`](crates/wasm/snapshots/facade/.keep) &mdash; 1 lines
- [`default.d.ts`](crates/wasm/snapshots/facade/default.d.ts) &mdash; 117 lines
- [`orderbook.d.ts`](crates/wasm/snapshots/facade/orderbook.d.ts) &mdash; 77 lines
- [`signing.d.ts`](crates/wasm/snapshots/facade/signing.d.ts) &mdash; 41 lines
- [`trading.d.ts`](crates/wasm/snapshots/facade/trading.d.ts) &mdash; 98 lines

</details>

<details>
<summary><code>crates/wasm/snapshots/raw/</code> &mdash; 5 file(s)</summary>

- [`.keep`](crates/wasm/snapshots/raw/.keep) &mdash; 1 lines
- [`default.d.ts`](crates/wasm/snapshots/raw/default.d.ts) &mdash; 3,170 lines
- [`orderbook.d.ts`](crates/wasm/snapshots/raw/orderbook.d.ts) &mdash; 2,050 lines
- [`signing.d.ts`](crates/wasm/snapshots/raw/signing.d.ts) &mdash; 678 lines
- [`trading.d.ts`](crates/wasm/snapshots/raw/trading.d.ts) &mdash; 3,016 lines

</details>

<details>
<summary><code>crates/wasm/src/</code> &mdash; 1 file(s)</summary>

- [`lib.rs`](crates/wasm/src/lib.rs) &mdash; 43 lines

</details>

<details>
<summary><code>crates/wasm/src/exports/</code> &mdash; 15 file(s)</summary>

- [`callbacks.rs`](crates/wasm/src/exports/callbacks.rs) &mdash; 134 lines
- [`cancel.rs`](crates/wasm/src/exports/cancel.rs) &mdash; 245 lines
- [`chains.rs`](crates/wasm/src/exports/chains.rs) &mdash; 264 lines
- [`eip1271.rs`](crates/wasm/src/exports/eip1271.rs) &mdash; 204 lines
- [`envelope.rs`](crates/wasm/src/exports/envelope.rs) &mdash; 37 lines
- [`errors.rs`](crates/wasm/src/exports/errors.rs) &mdash; 730 lines
- [`events.rs`](crates/wasm/src/exports/events.rs) &mdash; 64 lines
- [`ipfs.rs`](crates/wasm/src/exports/ipfs.rs) &mdash; 255 lines
- [`mod.rs`](crates/wasm/src/exports/mod.rs) &mdash; 135 lines
- [`orderbook.rs`](crates/wasm/src/exports/orderbook.rs) &mdash; 923 lines
- [`registry.rs`](crates/wasm/src/exports/registry.rs) &mdash; 95 lines
- [`signing.rs`](crates/wasm/src/exports/signing.rs) &mdash; 742 lines
- [`subgraph.rs`](crates/wasm/src/exports/subgraph.rs) &mdash; 241 lines
- [`trading.rs`](crates/wasm/src/exports/trading.rs) &mdash; 891 lines
- [`transport.rs`](crates/wasm/src/exports/transport.rs) &mdash; 488 lines

</details>

<details>
<summary><code>crates/wasm/src/exports/dto/</code> &mdash; 12 file(s)</summary>

- [`app_data.rs`](crates/wasm/src/exports/dto/app_data.rs) &mdash; 104 lines
- [`contracts.rs`](crates/wasm/src/exports/dto/contracts.rs) &mdash; 134 lines
- [`core.rs`](crates/wasm/src/exports/dto/core.rs) &mdash; 144 lines
- [`events.rs`](crates/wasm/src/exports/dto/events.rs) &mdash; 298 lines
- [`mod.rs`](crates/wasm/src/exports/dto/mod.rs) &mdash; 102 lines
- [`order.rs`](crates/wasm/src/exports/dto/order.rs) &mdash; 239 lines
- [`orderbook.rs`](crates/wasm/src/exports/dto/orderbook.rs) &mdash; 552 lines
- [`quote.rs`](crates/wasm/src/exports/dto/quote.rs) &mdash; 267 lines
- [`signing.rs`](crates/wasm/src/exports/dto/signing.rs) &mdash; 206 lines
- [`subgraph.rs`](crates/wasm/src/exports/dto/subgraph.rs) &mdash; 19 lines
- [`trading.rs`](crates/wasm/src/exports/dto/trading.rs) &mdash; 318 lines
- [`transport.rs`](crates/wasm/src/exports/dto/transport.rs) &mdash; 310 lines

</details>

<details>
<summary><code>crates/wasm/src/helpers/</code> &mdash; 6 file(s)</summary>

- [`app_data.rs`](crates/wasm/src/helpers/app_data.rs) &mdash; 84 lines
- [`chains.rs`](crates/wasm/src/helpers/chains.rs) &mdash; 100 lines
- [`dto.rs`](crates/wasm/src/helpers/dto.rs) &mdash; 334 lines
- [`errors.rs`](crates/wasm/src/helpers/errors.rs) &mdash; 51 lines
- [`mod.rs`](crates/wasm/src/helpers/mod.rs) &mdash; 17 lines
- [`signing.rs`](crates/wasm/src/helpers/signing.rs) &mdash; 41 lines

</details>

<details>
<summary><code>crates/wasm/tests/</code> &mdash; 25 file(s)</summary>

- [`host_pure_helpers.rs`](crates/wasm/tests/host_pure_helpers.rs) &mdash; 295 lines
- [`no_ffi_helpers.rs`](crates/wasm/tests/no_ffi_helpers.rs) &mdash; 63 lines
- [`transport_fetch_contract.rs`](crates/wasm/tests/transport_fetch_contract.rs) &mdash; 376 lines
- [`transport_fetch_smoke.rs`](crates/wasm/tests/transport_fetch_smoke.rs) &mdash; 22 lines
- [`transport_parity_contract.rs`](crates/wasm/tests/transport_parity_contract.rs) &mdash; 539 lines
- [`wasm_callback_contract.rs`](crates/wasm/tests/wasm_callback_contract.rs) &mdash; 391 lines
- [`wasm_callback_lifetime_contract.rs`](crates/wasm/tests/wasm_callback_lifetime_contract.rs) &mdash; 55 lines
- [`wasm_callback_transport_contract.rs`](crates/wasm/tests/wasm_callback_transport_contract.rs) &mdash; 135 lines
- [`wasm_cancellation_contract.rs`](crates/wasm/tests/wasm_cancellation_contract.rs) &mdash; 239 lines
- [`wasm_dto_parity_contract.rs`](crates/wasm/tests/wasm_dto_parity_contract.rs) &mdash; 134 lines
- [`wasm_eip1271_contract.rs`](crates/wasm/tests/wasm_eip1271_contract.rs) &mdash; 248 lines
- [`wasm_envelope_contract.rs`](crates/wasm/tests/wasm_envelope_contract.rs) &mdash; 33 lines
- [`wasm_error_abi_contract.rs`](crates/wasm/tests/wasm_error_abi_contract.rs) &mdash; 224 lines
- [`wasm_facade_coverage_contract.rs`](crates/wasm/tests/wasm_facade_coverage_contract.rs) &mdash; 230 lines
- [`wasm_facade_snapshot_contract.rs`](crates/wasm/tests/wasm_facade_snapshot_contract.rs) &mdash; 161 lines
- [`wasm_fail_closed_contract.rs`](crates/wasm/tests/wasm_fail_closed_contract.rs) &mdash; 227 lines
- [`wasm_flavour_reachability_contract.rs`](crates/wasm/tests/wasm_flavour_reachability_contract.rs) &mdash; 217 lines
- [`wasm_ipfs_contract.rs`](crates/wasm/tests/wasm_ipfs_contract.rs) &mdash; 181 lines
- [`wasm_redaction_contract.rs`](crates/wasm/tests/wasm_redaction_contract.rs) &mdash; 123 lines
- [`wasm_retry_runner_contract.rs`](crates/wasm/tests/wasm_retry_runner_contract.rs) &mdash; 69 lines
- [`wasm_snapshot_surface_contract.rs`](crates/wasm/tests/wasm_snapshot_surface_contract.rs) &mdash; 437 lines
- [`wasm_surface_contract.rs`](crates/wasm/tests/wasm_surface_contract.rs) &mdash; 228 lines
- [`wasm_telemetry_contract.rs`](crates/wasm/tests/wasm_telemetry_contract.rs) &mdash; 54 lines
- [`wasm_transport_policy_contract.rs`](crates/wasm/tests/wasm_transport_policy_contract.rs) &mdash; 320 lines
- [`wasm_workflow_coverage_contract.rs`](crates/wasm/tests/wasm_workflow_coverage_contract.rs) &mdash; 507 lines

</details>

<details>
<summary><code>crates/wasm/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/wasm/tests/common/mod.rs) &mdash; 195 lines

</details>

<details>
<summary><code>docs/</code> &mdash; 19 file(s)</summary>

- [`alloy-doctrine.md`](docs/alloy-doctrine.md) &mdash; 178 lines
- [`alloy-major-release-runbook.md`](docs/alloy-major-release-runbook.md) &mdash; 63 lines
- [`architecture.md`](docs/architecture.md) &mdash; 462 lines
- [`code-of-conduct.md`](docs/code-of-conduct.md) &mdash; 71 lines
- [`comparison-with-typescript-sdk.md`](docs/comparison-with-typescript-sdk.md) &mdash; 87 lines
- [`deployments.md`](docs/deployments.md) &mdash; 102 lines
- [`examples.md`](docs/examples.md) &mdash; 104 lines
- [`getting-started.md`](docs/getting-started.md) &mdash; 781 lines
- [`integrations.md`](docs/integrations.md) &mdash; 391 lines
- [`msrv-policy.md`](docs/msrv-policy.md) &mdash; 39 lines
- [`observability.md`](docs/observability.md) &mdash; 488 lines
- [`parity.md`](docs/parity.md) &mdash; 490 lines
- [`performance.md`](docs/performance.md) &mdash; 276 lines
- [`principles.md`](docs/principles.md) &mdash; 242 lines
- [`publication-handoff.md`](docs/publication-handoff.md) &mdash; 117 lines
- [`README.md`](docs/README.md) &mdash; 124 lines
- [`release-checklist.md`](docs/release-checklist.md) &mdash; 454 lines
- [`transport.md`](docs/transport.md) &mdash; 486 lines
- [`verification.md`](docs/verification.md) &mdash; 327 lines

</details>

<details>
<summary><code>docs/adr/</code> &mdash; 56 file(s)</summary>

- [`0000-template.md`](docs/adr/0000-template.md) &mdash; 44 lines
- [`0001-multi-crate-sdk-family-with-thin-facade.md`](docs/adr/0001-multi-crate-sdk-family-with-thin-facade.md) &mdash; 69 lines
- [`0002-dedicated-trading-orchestration-crate.md`](docs/adr/0002-dedicated-trading-orchestration-crate.md) &mdash; 44 lines
- [`0003-separate-read-only-subgraph-crate.md`](docs/adr/0003-separate-read-only-subgraph-crate.md) &mdash; 65 lines
- [`0005-boundary-specific-runtime-contracts-and-strong-domain-types.md`](docs/adr/0005-boundary-specific-runtime-contracts-and-strong-domain-types.md) &mdash; 68 lines
- [`0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md`](docs/adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md) &mdash; 51 lines
- [`0010-runtime-neutral-async-and-transport-posture.md`](docs/adr/0010-runtime-neutral-async-and-transport-posture.md) &mdash; 91 lines
- [`0011-typed-amount-boundary-and-typestate-ready-state-construction.md`](docs/adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md) &mdash; 134 lines
- [`0012-alloy-sol-bindings-and-registry-authority.md`](docs/adr/0012-alloy-sol-bindings-and-registry-authority.md) &mdash; 107 lines
- [`0013-http-transport-injection-and-typestate-builders.md`](docs/adr/0013-http-transport-injection-and-typestate-builders.md) &mdash; 94 lines
- [`0014-eip1271-verification-cache.md`](docs/adr/0014-eip1271-verification-cache.md) &mdash; 108 lines
- [`0015-client-side-order-bounds-validator.md`](docs/adr/0015-client-side-order-bounds-validator.md) &mdash; 121 lines
- [`0016-split-sell-and-buy-token-balance-enums.md`](docs/adr/0016-split-sell-and-buy-token-balance-enums.md) &mdash; 89 lines
- [`0017-typed-orderbook-rejection-parser.md`](docs/adr/0017-typed-orderbook-rejection-parser.md) &mdash; 135 lines
- [`0018-typed-app-data-merge.md`](docs/adr/0018-typed-app-data-merge.md) &mdash; 134 lines
- [`0020-ethflow-owner-threading.md`](docs/adr/0020-ethflow-owner-threading.md) &mdash; 159 lines
- [`0021-orderbook-total-fee-policy.md`](docs/adr/0021-orderbook-total-fee-policy.md) &mdash; 110 lines
- [`0022-ecdsa-signature-v-normalization.md`](docs/adr/0022-ecdsa-signature-v-normalization.md) &mdash; 135 lines
- [`0024-asyncprovider-asyncsigningprovider-capability-split.md`](docs/adr/0024-asyncprovider-asyncsigningprovider-capability-split.md) &mdash; 70 lines
- [`0025-workspace-url-redaction-convention.md`](docs/adr/0025-workspace-url-redaction-convention.md) &mdash; 60 lines
- [`0026-alloy-major-release-absorption-plan.md`](docs/adr/0026-alloy-major-release-absorption-plan.md) &mdash; 96 lines
- [`0027-post-quantum-signing-absorption-plan.md`](docs/adr/0027-post-quantum-signing-absorption-plan.md) &mdash; 77 lines
- [`0028-account-abstraction-integration-plan.md`](docs/adr/0028-account-abstraction-integration-plan.md) &mdash; 82 lines
- [`0030-workspace-locked-versioning-tag-baseline.md`](docs/adr/0030-workspace-locked-versioning-tag-baseline.md) &mdash; 78 lines
- [`0031-wire-dto-openapi-driven-with-order-auction-order-split.md`](docs/adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md) &mdash; 77 lines
- [`0032-deployment-authority-machine-readable-provenance.md`](docs/adr/0032-deployment-authority-machine-readable-provenance.md) &mdash; 103 lines
- [`0033-minimum-viable-panic-surface.md`](docs/adr/0033-minimum-viable-panic-surface.md) &mdash; 71 lines
- [`0035-alloy-provider-adapter.md`](docs/adr/0035-alloy-provider-adapter.md) &mdash; 152 lines
- [`0038-transaction-lifecycle-types.md`](docs/adr/0038-transaction-lifecycle-types.md) &mdash; 72 lines
- [`0039-typescript-callable-wasm-sdk-surface.md`](docs/adr/0039-typescript-callable-wasm-sdk-surface.md) &mdash; 154 lines
- [`0040-wallet-provider-callback-boundary-for-js-consumers.md`](docs/adr/0040-wallet-provider-callback-boundary-for-js-consumers.md) &mdash; 71 lines
- [`0041-transport-policy-l3-layering.md`](docs/adr/0041-transport-policy-l3-layering.md) &mdash; 94 lines
- [`0044-bundle-size-profile-and-flavor-builds.md`](docs/adr/0044-bundle-size-profile-and-flavor-builds.md) &mdash; 108 lines
- [`0045-async-signer-trait-narrowing.md`](docs/adr/0045-async-signer-trait-narrowing.md) &mdash; 54 lines
- [`0048-composable-conditional-order-framework.md`](docs/adr/0048-composable-conditional-order-framework.md) &mdash; 208 lines
- [`0049-cow-shed-account-abstraction-proxy.md`](docs/adr/0049-cow-shed-account-abstraction-proxy.md) &mdash; 144 lines
- [`0050-eip1271-signature-blob-encoding.md`](docs/adr/0050-eip1271-signature-blob-encoding.md) &mdash; 172 lines
- [`0051-signing-owned-eip1271-signature-provider-trait.md`](docs/adr/0051-signing-owned-eip1271-signature-provider-trait.md) &mdash; 134 lines
- [`0052-alloy-primitives-canonical-primitive-layer.md`](docs/adr/0052-alloy-primitives-canonical-primitive-layer.md) &mdash; 126 lines
- [`0053-typed-signer-rejection-classification.md`](docs/adr/0053-typed-signer-rejection-classification.md) &mdash; 155 lines
- [`0054-onchain-order-event-decoding-is-fail-closed.md`](docs/adr/0054-onchain-order-event-decoding-is-fail-closed.md) &mdash; 85 lines
- [`0055-bounded-response-reads.md`](docs/adr/0055-bounded-response-reads.md) &mdash; 94 lines
- [`0057-log-provider-capability-trait.md`](docs/adr/0057-log-provider-capability-trait.md) &mdash; 125 lines
- [`0058-typed-quote-request-response-surface.md`](docs/adr/0058-typed-quote-request-response-surface.md) &mdash; 200 lines
- [`0059-hash-concrete-orderdata-directly.md`](docs/adr/0059-hash-concrete-orderdata-directly.md) &mdash; 75 lines
- [`0060-uniform-error-classification.md`](docs/adr/0060-uniform-error-classification.md) &mdash; 117 lines
- [`0061-wasm-abi-receiver-pay-to-owner.md`](docs/adr/0061-wasm-abi-receiver-pay-to-owner.md) &mdash; 79 lines
- [`0062-internal-shared-test-support-crate.md`](docs/adr/0062-internal-shared-test-support-crate.md) &mdash; 60 lines
- [`0063-published-consumer-test-doubles-crate.md`](docs/adr/0063-published-consumer-test-doubles-crate.md) &mdash; 93 lines
- [`0064-app-data-typed-validation.md`](docs/adr/0064-app-data-typed-validation.md) &mdash; 90 lines
- [`0066-trading-slippage-and-suggestion-policy.md`](docs/adr/0066-trading-slippage-and-suggestion-policy.md) &mdash; 59 lines
- [`0067-idiomatic-accessor-naming.md`](docs/adr/0067-idiomatic-accessor-naming.md) &mdash; 68 lines
- [`0068-payload-only-typed-data-signing.md`](docs/adr/0068-payload-only-typed-data-signing.md) &mdash; 80 lines
- [`0069-layered-trading-operation-surface-and-signing-free-transport.md`](docs/adr/0069-layered-trading-operation-surface-and-signing-free-transport.md) &mdash; 85 lines
- [`0070-onchain-transaction-helper-boundary.md`](docs/adr/0070-onchain-transaction-helper-boundary.md) &mdash; 72 lines
- [`README.md`](docs/adr/README.md) &mdash; 222 lines

</details>

<details>
<summary><code>docs/audit/</code> &mdash; 19 file(s)</summary>

- [`alloy-adapters-audit.md`](docs/audit/alloy-adapters-audit.md) &mdash; 191 lines
- [`bounded-response-reads-audit.md`](docs/audit/bounded-response-reads-audit.md) &mdash; 125 lines
- [`contract-bindings-parity-audit.md`](docs/audit/contract-bindings-parity-audit.md) &mdash; 241 lines
- [`cow-shed-contract-bindings-audit.md`](docs/audit/cow-shed-contract-bindings-audit.md) &mdash; 245 lines
- [`credential-redaction-audit.md`](docs/audit/credential-redaction-audit.md) &mdash; 286 lines
- [`dependency-gate-audit.md`](docs/audit/dependency-gate-audit.md) &mdash; 354 lines
- [`deployment-registry-audit.md`](docs/audit/deployment-registry-audit.md) &mdash; 138 lines
- [`ecdsa-signature-normalization-audit.md`](docs/audit/ecdsa-signature-normalization-audit.md) &mdash; 135 lines
- [`eip1271-verification-cache-audit.md`](docs/audit/eip1271-verification-cache-audit.md) &mdash; 238 lines
- [`error-classification-audit.md`](docs/audit/error-classification-audit.md) &mdash; 197 lines
- [`event-log-decoding-audit.md`](docs/audit/event-log-decoding-audit.md) &mdash; 87 lines
- [`fuzz-coverage-audit.md`](docs/audit/fuzz-coverage-audit.md) &mdash; 209 lines
- [`http-transport-contract-audit.md`](docs/audit/http-transport-contract-audit.md) &mdash; 350 lines
- [`panic-free-public-surface-audit.md`](docs/audit/panic-free-public-surface-audit.md) &mdash; 124 lines
- [`README.md`](docs/audit/README.md) &mdash; 91 lines
- [`source-lock-provenance-audit.md`](docs/audit/source-lock-provenance-audit.md) &mdash; 218 lines
- [`trading-order-integrity-audit.md`](docs/audit/trading-order-integrity-audit.md) &mdash; 155 lines
- [`wasm-surface-audit.md`](docs/audit/wasm-surface-audit.md) &mdash; 356 lines
- [`workflow-security-audit.md`](docs/audit/workflow-security-audit.md) &mdash; 137 lines

</details>

<details>
<summary><code>docs/providers/</code> &mdash; 2 file(s)</summary>

- [`adapting-alloy.md`](docs/providers/adapting-alloy.md) &mdash; 202 lines
- [`README.md`](docs/providers/README.md) &mdash; 77 lines

</details>

<details>
<summary><code>e2e/wasm-typescript/</code> &mdash; 8 file(s)</summary>

- [`index.html`](e2e/wasm-typescript/index.html) &mdash; 12 lines
- [`package.json`](e2e/wasm-typescript/package.json) &mdash; 28 lines
- [`playwright.config.ts`](e2e/wasm-typescript/playwright.config.ts) &mdash; 16 lines
- [`pnpm-lock.yaml`](e2e/wasm-typescript/pnpm-lock.yaml) &mdash; 1,365 lines
- [`pnpm-workspace.yaml`](e2e/wasm-typescript/pnpm-workspace.yaml) &mdash; 3 lines
- [`tsconfig.json`](e2e/wasm-typescript/tsconfig.json) &mdash; 14 lines
- [`vite.config.ts`](e2e/wasm-typescript/vite.config.ts) &mdash; 16 lines
- [`vitest.config.ts`](e2e/wasm-typescript/vitest.config.ts) &mdash; 11 lines

</details>

<details>
<summary><code>e2e/wasm-typescript-cf/</code> &mdash; 7 file(s)</summary>

- [`package.json`](e2e/wasm-typescript-cf/package.json) &mdash; 27 lines
- [`pnpm-lock.yaml`](e2e/wasm-typescript-cf/pnpm-lock.yaml) &mdash; 1,684 lines
- [`pnpm-workspace.yaml`](e2e/wasm-typescript-cf/pnpm-workspace.yaml) &mdash; 4 lines
- [`tsconfig.json`](e2e/wasm-typescript-cf/tsconfig.json) &mdash; 14 lines
- [`vitest.config.ts`](e2e/wasm-typescript-cf/vitest.config.ts) &mdash; 15 lines
- [`worker-configuration.d.ts`](e2e/wasm-typescript-cf/worker-configuration.d.ts) &mdash; 15 lines
- [`wrangler.toml`](e2e/wasm-typescript-cf/wrangler.toml) &mdash; 8 lines

</details>

<details>
<summary><code>e2e/wasm-typescript-cf/src/</code> &mdash; 2 file(s)</summary>

- [`wasm.d.ts`](e2e/wasm-typescript-cf/src/wasm.d.ts) &mdash; 9 lines
- [`worker.ts`](e2e/wasm-typescript-cf/src/worker.ts) &mdash; 61 lines

</details>

<details>
<summary><code>e2e/wasm-typescript-cf/tests/</code> &mdash; 3 file(s)</summary>

- [`forbidden-instantiation.spec.ts`](e2e/wasm-typescript-cf/tests/forbidden-instantiation.spec.ts) &mdash; 17 lines
- [`init-once.spec.ts`](e2e/wasm-typescript-cf/tests/init-once.spec.ts) &mdash; 13 lines
- [`orderbook.spec.ts`](e2e/wasm-typescript-cf/tests/orderbook.spec.ts) &mdash; 34 lines

</details>

<details>
<summary><code>e2e/wasm-typescript/src/</code> &mdash; 1 file(s)</summary>

- [`index.ts`](e2e/wasm-typescript/src/index.ts) &mdash; 59 lines

</details>

<details>
<summary><code>e2e/wasm-typescript/tests/</code> &mdash; 4 file(s)</summary>

- [`eip1271.spec.ts`](e2e/wasm-typescript/tests/eip1271.spec.ts) &mdash; 40 lines
- [`orderbook.spec.ts`](e2e/wasm-typescript/tests/orderbook.spec.ts) &mdash; 93 lines
- [`signing.spec.ts`](e2e/wasm-typescript/tests/signing.spec.ts) &mdash; 81 lines
- [`transport.spec.ts`](e2e/wasm-typescript/tests/transport.spec.ts) &mdash; 64 lines

</details>

<details>
<summary><code>e2e/wasm-typescript/tests/browser/</code> &mdash; 1 file(s)</summary>

- [`browser.spec.ts`](e2e/wasm-typescript/tests/browser/browser.spec.ts) &mdash; 13 lines

</details>

<details>
<summary><code>examples/</code> &mdash; 1 file(s)</summary>

- [`README.md`](examples/README.md) &mdash; 43 lines

</details>

<details>
<summary><code>examples/native/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](examples/native/Cargo.toml) &mdash; 146 lines
- [`README.md`](examples/native/README.md) &mdash; 159 lines

</details>

<details>
<summary><code>examples/native/scenarios/</code> &mdash; 28 file(s)</summary>

- [`alloy_custom_traits.rs`](examples/native/scenarios/alloy_custom_traits.rs) &mdash; 164 lines
- [`alloy_provider.rs`](examples/native/scenarios/alloy_provider.rs) &mdash; 42 lines
- [`alloy_quickstart.rs`](examples/native/scenarios/alloy_quickstart.rs) &mdash; 47 lines
- [`alloy_signer.rs`](examples/native/scenarios/alloy_signer.rs) &mdash; 71 lines
- [`alloy_trading_full_flow.rs`](examples/native/scenarios/alloy_trading_full_flow.rs) &mdash; 102 lines
- [`app_data.rs`](examples/native/scenarios/app_data.rs) &mdash; 48 lines
- [`cancel_in_flight.rs`](examples/native/scenarios/cancel_in_flight.rs) &mdash; 89 lines
- [`eip1271_signer.rs`](examples/native/scenarios/eip1271_signer.rs) &mdash; 71 lines
- [`error_classification.rs`](examples/native/scenarios/error_classification.rs) &mdash; 283 lines
- [`ethflow_checker.rs`](examples/native/scenarios/ethflow_checker.rs) &mdash; 105 lines
- [`ethflow.rs`](examples/native/scenarios/ethflow.rs) &mdash; 107 lines
- [`facade_surface.rs`](examples/native/scenarios/facade_surface.rs) &mdash; 39 lines
- [`limit_order.rs`](examples/native/scenarios/limit_order.rs) &mdash; 89 lines
- [`onchain_actions.rs`](examples/native/scenarios/onchain_actions.rs) &mdash; 162 lines
- [`order_history.rs`](examples/native/scenarios/order_history.rs) &mdash; 106 lines
- [`order_lifecycle.rs`](examples/native/scenarios/order_lifecycle.rs) &mdash; 59 lines
- [`orderbook_live.rs`](examples/native/scenarios/orderbook_live.rs) &mdash; 63 lines
- [`orderbook_transport.rs`](examples/native/scenarios/orderbook_transport.rs) &mdash; 128 lines
- [`quote.rs`](examples/native/scenarios/quote.rs) &mdash; 67 lines
- [`receipt_lifecycle.rs`](examples/native/scenarios/receipt_lifecycle.rs) &mdash; 82 lines
- [`sign_order.rs`](examples/native/scenarios/sign_order.rs) &mdash; 64 lines
- [`slippage_suggester.rs`](examples/native/scenarios/slippage_suggester.rs) &mdash; 71 lines
- [`subgraph_live.rs`](examples/native/scenarios/subgraph_live.rs) &mdash; 48 lines
- [`subgraph_query.rs`](examples/native/scenarios/subgraph_query.rs) &mdash; 177 lines
- [`swap_quickstart.rs`](examples/native/scenarios/swap_quickstart.rs) &mdash; 68 lines
- [`token_balance.rs`](examples/native/scenarios/token_balance.rs) &mdash; 75 lines
- [`trading_full_cycle.rs`](examples/native/scenarios/trading_full_cycle.rs) &mdash; 107 lines
- [`transaction_lifecycle.rs`](examples/native/scenarios/transaction_lifecycle.rs) &mdash; 76 lines

</details>

<details>
<summary><code>examples/native/src/</code> &mdash; 2 file(s)</summary>

- [`lib.rs`](examples/native/src/lib.rs) &mdash; 18 lines
- [`support.rs`](examples/native/src/support.rs) &mdash; 361 lines

</details>

<details>
<summary><code>examples/native/tests/</code> &mdash; 1 file(s)</summary>

- [`scenario_contract.rs`](examples/native/tests/scenario_contract.rs) &mdash; 206 lines

</details>

<details>
<summary><code>fuzz/</code> &mdash; 3 file(s)</summary>

- [`Cargo.lock`](fuzz/Cargo.lock) &mdash; 3,756 lines
- [`Cargo.toml`](fuzz/Cargo.toml) &mdash; 324 lines
- [`README.md`](fuzz/README.md) &mdash; 182 lines

</details>

<details>
<summary><code>fuzz/fuzz_targets/</code> &mdash; 42 file(s)</summary>

- [`fuzz_amount_parse_units.rs`](fuzz/fuzz_targets/fuzz_amount_parse_units.rs) &mdash; 62 lines
- [`fuzz_amount_parse.rs`](fuzz/fuzz_targets/fuzz_amount_parse.rs) &mdash; 75 lines
- [`fuzz_app_data_cid_roundtrip.rs`](fuzz/fuzz_targets/fuzz_app_data_cid_roundtrip.rs) &mdash; 91 lines
- [`fuzz_app_data_merge.rs`](fuzz/fuzz_targets/fuzz_app_data_merge.rs) &mdash; 309 lines
- [`fuzz_app_data_params_from_doc.rs`](fuzz/fuzz_targets/fuzz_app_data_params_from_doc.rs) &mdash; 362 lines
- [`fuzz_app_data_size_limit.rs`](fuzz/fuzz_targets/fuzz_app_data_size_limit.rs) &mdash; 158 lines
- [`fuzz_calculate_total_fee.rs`](fuzz/fuzz_targets/fuzz_calculate_total_fee.rs) &mdash; 96 lines
- [`fuzz_cid_to_app_data_hex.rs`](fuzz/fuzz_targets/fuzz_cid_to_app_data_hex.rs) &mdash; 90 lines
- [`fuzz_core_identity_validators.rs`](fuzz/fuzz_targets/fuzz_core_identity_validators.rs) &mdash; 195 lines
- [`fuzz_decode_magic_value_response.rs`](fuzz/fuzz_targets/fuzz_decode_magic_value_response.rs) &mdash; 214 lines
- [`fuzz_decoded_body_canonical_status_text.rs`](fuzz/fuzz_targets/fuzz_decoded_body_canonical_status_text.rs) &mdash; 243 lines
- [`fuzz_ecdsa_v_normalization.rs`](fuzz/fuzz_targets/fuzz_ecdsa_v_normalization.rs) &mdash; 54 lines
- [`fuzz_eip1271_signature_data_codec.rs`](fuzz/fuzz_targets/fuzz_eip1271_signature_data_codec.rs) &mdash; 56 lines
- [`fuzz_eth_flow_event_log_decode.rs`](fuzz/fuzz_targets/fuzz_eth_flow_event_log_decode.rs) &mdash; 52 lines
- [`fuzz_ethflow_create_order_encode.rs`](fuzz/fuzz_targets/fuzz_ethflow_create_order_encode.rs) &mdash; 115 lines
- [`fuzz_flashloan_hints.rs`](fuzz/fuzz_targets/fuzz_flashloan_hints.rs) &mdash; 111 lines
- [`fuzz_hash_order_cancellations.rs`](fuzz/fuzz_targets/fuzz_hash_order_cancellations.rs) &mdash; 158 lines
- [`fuzz_hook_list_deserialize.rs`](fuzz/fuzz_targets/fuzz_hook_list_deserialize.rs) &mdash; 96 lines
- [`fuzz_jitter_delay_for_attempt.rs`](fuzz/fuzz_targets/fuzz_jitter_delay_for_attempt.rs) &mdash; 115 lines
- [`fuzz_onchain_order_log_decode.rs`](fuzz/fuzz_targets/fuzz_onchain_order_log_decode.rs) &mdash; 61 lines
- [`fuzz_order_bounds_validator.rs`](fuzz/fuzz_targets/fuzz_order_bounds_validator.rs) &mdash; 272 lines
- [`fuzz_order_signature_classify.rs`](fuzz/fuzz_targets/fuzz_order_signature_classify.rs) &mdash; 83 lines
- [`fuzz_order_uid_pack_unpack.rs`](fuzz/fuzz_targets/fuzz_order_uid_pack_unpack.rs) &mdash; 56 lines
- [`fuzz_orderbook_rejection_code.rs`](fuzz/fuzz_targets/fuzz_orderbook_rejection_code.rs) &mdash; 87 lines
- [`fuzz_orderbook_rejection_decode.rs`](fuzz/fuzz_targets/fuzz_orderbook_rejection_decode.rs) &mdash; 52 lines
- [`fuzz_parse_retry_after.rs`](fuzz/fuzz_targets/fuzz_parse_retry_after.rs) &mdash; 51 lines
- [`fuzz_partner_fee_from_value.rs`](fuzz/fuzz_targets/fuzz_partner_fee_from_value.rs) &mdash; 78 lines
- [`fuzz_recover_ecdsa_address.rs`](fuzz/fuzz_targets/fuzz_recover_ecdsa_address.rs) &mdash; 88 lines
- [`fuzz_recoverable_signature_differential.rs`](fuzz/fuzz_targets/fuzz_recoverable_signature_differential.rs) &mdash; 92 lines
- [`fuzz_recoverable_signature_parse_hex.rs`](fuzz/fuzz_targets/fuzz_recoverable_signature_parse_hex.rs) &mdash; 61 lines
- [`fuzz_redact_response_body.rs`](fuzz/fuzz_targets/fuzz_redact_response_body.rs) &mdash; 84 lines
- [`fuzz_retry_policy_delay.rs`](fuzz/fuzz_targets/fuzz_retry_policy_delay.rs) &mdash; 153 lines
- [`fuzz_schema_version_is_semver.rs`](fuzz/fuzz_targets/fuzz_schema_version_is_semver.rs) &mdash; 92 lines
- [`fuzz_settlement_event_log_decode.rs`](fuzz/fuzz_targets/fuzz_settlement_event_log_decode.rs) &mdash; 54 lines
- [`fuzz_signing_domain_separator.rs`](fuzz/fuzz_targets/fuzz_signing_domain_separator.rs) &mdash; 126 lines
- [`fuzz_slippage_amounts.rs`](fuzz/fuzz_targets/fuzz_slippage_amounts.rs) &mdash; 160 lines
- [`fuzz_slippage_policy_helpers.rs`](fuzz/fuzz_targets/fuzz_slippage_policy_helpers.rs) &mdash; 182 lines
- [`fuzz_stringify_deterministic.rs`](fuzz/fuzz_targets/fuzz_stringify_deterministic.rs) &mdash; 73 lines
- [`fuzz_subgraph_graphql_error_decode.rs`](fuzz/fuzz_targets/fuzz_subgraph_graphql_error_decode.rs) &mdash; 93 lines
- [`fuzz_transport_error_classify.rs`](fuzz/fuzz_targets/fuzz_transport_error_classify.rs) &mdash; 282 lines
- [`fuzz_typed_data_digest.rs`](fuzz/fuzz_targets/fuzz_typed_data_digest.rs) &mdash; 142 lines
- [`fuzz_valid_to_relative.rs`](fuzz/fuzz_targets/fuzz_valid_to_relative.rs) &mdash; 89 lines

</details>

<details>
<summary><code>parity/</code> &mdash; 2 file(s)</summary>

- [`README.md`](parity/README.md) &mdash; 265 lines
- [`source-lock.yaml`](parity/source-lock.yaml) &mdash; 80 lines

</details>

<details>
<summary><code>parity/fixtures/</code> &mdash; 1 file(s)</summary>

- [`contracts.json`](parity/fixtures/contracts.json) &mdash; 252 lines

</details>

<details>
<summary><code>parity/fixtures/app_data/</code> &mdash; 3 file(s)</summary>

- [`canonical_json_utf16.json`](parity/fixtures/app_data/canonical_json_utf16.json) &mdash; 22 lines
- [`flashloan_v1.7.0.json`](parity/fixtures/app_data/flashloan_v1.7.0.json) &mdash; 19 lines
- [`hooks_v1.14.0.json`](parity/fixtures/app_data/hooks_v1.14.0.json) &mdash; 37 lines

</details>

<details>
<summary><code>parity/fixtures/app_data/schemas/</code> &mdash; 5 file(s)</summary>

- [`app-data-document-v1.15.0.json`](parity/fixtures/app_data/schemas/app-data-document-v1.15.0.json) &mdash; 92 lines
- [`flashloan.json`](parity/fixtures/app_data/schemas/flashloan.json) &mdash; 60 lines
- [`hook-v0.2.0.json`](parity/fixtures/app_data/schemas/hook-v0.2.0.json) &mdash; 55 lines
- [`partner-fee-v1.1.0.json`](parity/fixtures/app_data/schemas/partner-fee-v1.1.0.json) &mdash; 118 lines
- [`quote-v1.1.0.json`](parity/fixtures/app_data/schemas/quote-v1.1.0.json) &mdash; 38 lines

</details>

<details>
<summary><code>parity/fixtures/chains/</code> &mdash; 1 file(s)</summary>

- [`supported_networks.json`](parity/fixtures/chains/supported_networks.json) &mdash; 25 lines

</details>

<details>
<summary><code>parity/fixtures/cow_shed/</code> &mdash; 7 file(s)</summary>

- [`canonical_selectors.json`](parity/fixtures/cow_shed/canonical_selectors.json) &mdash; 132 lines
- [`deployments.json`](parity/fixtures/cow_shed/deployments.json) &mdash; 33 lines
- [`domain_separator.json`](parity/fixtures/cow_shed/domain_separator.json) &mdash; 35 lines
- [`eoa_signature_byte_order.json`](parity/fixtures/cow_shed/eoa_signature_byte_order.json) &mdash; 42 lines
- [`execute_hooks_calldata.json`](parity/fixtures/cow_shed/execute_hooks_calldata.json) &mdash; 129 lines
- [`execute_hooks_digest.json`](parity/fixtures/cow_shed/execute_hooks_digest.json) &mdash; 55 lines
- [`proxy_addresses.json`](parity/fixtures/cow_shed/proxy_addresses.json) &mdash; 94 lines

</details>

<details>
<summary><code>parity/fixtures/ecdsa/</code> &mdash; 1 file(s)</summary>

- [`v_normalization.json`](parity/fixtures/ecdsa/v_normalization.json) &mdash; 155 lines

</details>

<details>
<summary><code>parity/fixtures/eip712/</code> &mdash; 2 file(s)</summary>

- [`order_digests.json`](parity/fixtures/eip712/order_digests.json) &mdash; 242 lines
- [`settlement_domain_separator.json`](parity/fixtures/eip712/settlement_domain_separator.json) &mdash; 22 lines

</details>

<details>
<summary><code>parity/fixtures/orderbook/</code> &mdash; 9 file(s)</summary>

- [`onchain_order_data.json`](parity/fixtures/orderbook/onchain_order_data.json) &mdash; 16 lines
- [`order_quote_response.json`](parity/fixtures/orderbook/order_quote_response.json) &mdash; 38 lines
- [`order_with_full_metadata.json`](parity/fixtures/orderbook/order_with_full_metadata.json) &mdash; 86 lines
- [`rejection_error_types.json`](parity/fixtures/orderbook/rejection_error_types.json) &mdash; 61 lines
- [`solver_competition_response.json`](parity/fixtures/orderbook/solver_competition_response.json) &mdash; 56 lines
- [`solver_execution.json`](parity/fixtures/orderbook/solver_execution.json) &mdash; 19 lines
- [`stored_order_quote.json`](parity/fixtures/orderbook/stored_order_quote.json) &mdash; 25 lines
- [`total_surplus.json`](parity/fixtures/orderbook/total_surplus.json) &mdash; 16 lines
- [`trade.json`](parity/fixtures/orderbook/trade.json) &mdash; 36 lines

</details>

<details>
<summary><code>parity/fixtures/orderbook-requests/</code> &mdash; 2 file(s)</summary>

- [`order_cancellations.json`](parity/fixtures/orderbook-requests/order_cancellations.json) &mdash; 25 lines
- [`order_creation.json`](parity/fixtures/orderbook-requests/order_creation.json) &mdash; 79 lines

</details>

<details>
<summary><code>parity/fixtures/retry_after/</code> &mdash; 3 file(s)</summary>

- [`imf_fixdate_accept.json`](parity/fixtures/retry_after/imf_fixdate_accept.json) &mdash; 109 lines
- [`imf_fixdate_reject.json`](parity/fixtures/retry_after/imf_fixdate_reject.json) &mdash; 79 lines
- [`legacy_rfc850.json`](parity/fixtures/retry_after/legacy_rfc850.json) &mdash; 59 lines

</details>

<details>
<summary><code>parity/fixtures/signing/</code> &mdash; 2 file(s)</summary>

- [`eip1271_typescript_vector.json`](parity/fixtures/signing/eip1271_typescript_vector.json) &mdash; 30 lines
- [`eth_sign_typed_data_request.json`](parity/fixtures/signing/eth_sign_typed_data_request.json) &mdash; 158 lines

</details>

<details>
<summary><code>parity/fixtures/trading/</code> &mdash; 1 file(s)</summary>

- [`protocol_fee_partner_fee_composition.json`](parity/fixtures/trading/protocol_fee_partner_fee_composition.json) &mdash; 39 lines

</details>

<details>
<summary><code>parity/openapi/</code> &mdash; 2 file(s)</summary>

- [`coverage.yaml`](parity/openapi/coverage.yaml) &mdash; 90 lines
- [`services-orderbook.yml`](parity/openapi/services-orderbook.yml) &mdash; 2,805 lines

</details>

<details>
<summary><code>tests/</code> &mdash; 11 file(s)</summary>

- [`alloy_read_contract_parity_invariant.rs`](tests/alloy_read_contract_parity_invariant.rs) &mdash; 104 lines
- [`alloy_two_family_lockfile_invariant.rs`](tests/alloy_two_family_lockfile_invariant.rs) &mdash; 112 lines
- [`alloy_umbrella_composition.rs`](tests/alloy_umbrella_composition.rs) &mdash; 100 lines
- [`Cargo.toml`](tests/Cargo.toml) &mdash; 68 lines
- [`cow_shed_typed_data_digest.rs`](tests/cow_shed_typed_data_digest.rs) &mdash; 77 lines
- [`dependency_default_features_audit.rs`](tests/dependency_default_features_audit.rs) &mdash; 82 lines
- [`msrv_consistency.rs`](tests/msrv_consistency.rs) &mdash; 37 lines
- [`supported_chains_doc_table.rs`](tests/supported_chains_doc_table.rs) &mdash; 103 lines
- [`transaction_lifecycle_cross_adapter_invariant.rs`](tests/transaction_lifecycle_cross_adapter_invariant.rs) &mdash; 134 lines
- [`wasm_dependency_invariant.rs`](tests/wasm_dependency_invariant.rs) &mdash; 70 lines
- [`workspace_alloy_pin_lockstep.rs`](tests/workspace_alloy_pin_lockstep.rs) &mdash; 126 lines

</details>

<details>
<summary><code>tests/support/</code> &mdash; 1 file(s)</summary>

- [`rpc.rs`](tests/support/rpc.rs) &mdash; 111 lines

</details>

<details>
<summary><code>xtask/</code> &mdash; 1 file(s)</summary>

- [`Cargo.toml`](xtask/Cargo.toml) &mdash; 31 lines

</details>

<details>
<summary><code>xtask/src/</code> &mdash; 4 file(s)</summary>

- [`changelog.rs`](xtask/src/changelog.rs) &mdash; 298 lines
- [`lib.rs`](xtask/src/lib.rs) &mdash; 22 lines
- [`main.rs`](xtask/src/main.rs) &mdash; 272 lines
- [`version_surface.rs`](xtask/src/version_surface.rs) &mdash; 280 lines

</details>

<details>
<summary><code>xtask/src/docs/</code> &mdash; 3 file(s)</summary>

- [`agree.rs`](xtask/src/docs/agree.rs) &mdash; 264 lines
- [`audit_index.rs`](xtask/src/docs/audit_index.rs) &mdash; 138 lines
- [`mod.rs`](xtask/src/docs/mod.rs) &mdash; 9 lines

</details>

<details>
<summary><code>xtask/src/parity/</code> &mdash; 5 file(s)</summary>

- [`mod.rs`](xtask/src/parity/mod.rs) &mdash; 1,375 lines
- [`openapi_coverage.rs`](xtask/src/parity/openapi_coverage.rs) &mdash; 750 lines
- [`registry_confirm.rs`](xtask/src/parity/registry_confirm.rs) &mdash; 364 lines
- [`sync.rs`](xtask/src/parity/sync.rs) &mdash; 565 lines
- [`vendor_openapi.rs`](xtask/src/parity/vendor_openapi.rs) &mdash; 67 lines

</details>

<details>
<summary><code>xtask/src/policy/</code> &mdash; 20 file(s)</summary>

- [`check_adr_coverage.rs`](xtask/src/policy/check_adr_coverage.rs) &mdash; 220 lines
- [`check_alloy_family_pins.rs`](xtask/src/policy/check_alloy_family_pins.rs) &mdash; 237 lines
- [`check_chain_patch_eligibility.rs`](xtask/src/policy/check_chain_patch_eligibility.rs) &mdash; 202 lines
- [`check_deny_unknown_fields.rs`](xtask/src/policy/check_deny_unknown_fields.rs) &mdash; 126 lines
- [`check_enum_policy.rs`](xtask/src/policy/check_enum_policy.rs) &mdash; 142 lines
- [`check_msrv_notice.rs`](xtask/src/policy/check_msrv_notice.rs) &mdash; 163 lines
- [`check_panic_allowlist.rs`](xtask/src/policy/check_panic_allowlist.rs) &mdash; 356 lines
- [`check_property_citations.rs`](xtask/src/policy/check_property_citations.rs) &mdash; 152 lines
- [`check_readme_include.rs`](xtask/src/policy/check_readme_include.rs) &mdash; 100 lines
- [`check_shell_wrappers.rs`](xtask/src/policy/check_shell_wrappers.rs) &mdash; 90 lines
- [`check_wasm_invariant.rs`](xtask/src/policy/check_wasm_invariant.rs) &mdash; 271 lines
- [`check_workflow_security.rs`](xtask/src/policy/check_workflow_security.rs) &mdash; 131 lines
- [`check_workspace_versions.rs`](xtask/src/policy/check_workspace_versions.rs) &mdash; 185 lines
- [`classify_release.rs`](xtask/src/policy/classify_release.rs) &mdash; 188 lines
- [`dependency_invariant.rs`](xtask/src/policy/dependency_invariant.rs) &mdash; 177 lines
- [`fences.rs`](xtask/src/policy/fences.rs) &mdash; 472 lines
- [`fixtures.rs`](xtask/src/policy/fixtures.rs) &mdash; 13 lines
- [`mod.rs`](xtask/src/policy/mod.rs) &mdash; 102 lines
- [`run_deterministic_examples.rs`](xtask/src/policy/run_deterministic_examples.rs) &mdash; 175 lines
- [`workspace.rs`](xtask/src/policy/workspace.rs) &mdash; 543 lines

</details>

<details>
<summary><code>xtask/tests/</code> &mdash; 12 file(s)</summary>

- [`check_adr_coverage.rs`](xtask/tests/check_adr_coverage.rs) &mdash; 51 lines
- [`check_chain_patch_eligibility.rs`](xtask/tests/check_chain_patch_eligibility.rs) &mdash; 45 lines
- [`check_deny_unknown_fields.rs`](xtask/tests/check_deny_unknown_fields.rs) &mdash; 46 lines
- [`check_enum_policy.rs`](xtask/tests/check_enum_policy.rs) &mdash; 50 lines
- [`check_msrv_notice.rs`](xtask/tests/check_msrv_notice.rs) &mdash; 37 lines
- [`check_panic_allowlist.rs`](xtask/tests/check_panic_allowlist.rs) &mdash; 192 lines
- [`check_property_citations.rs`](xtask/tests/check_property_citations.rs) &mdash; 81 lines
- [`check_workspace_versions.rs`](xtask/tests/check_workspace_versions.rs) &mdash; 25 lines
- [`classify_release.rs`](xtask/tests/classify_release.rs) &mdash; 103 lines
- [`openapi_coverage.rs`](xtask/tests/openapi_coverage.rs) &mdash; 162 lines
- [`registry_confirm.rs`](xtask/tests/registry_confirm.rs) &mdash; 157 lines
- [`vendor_openapi.rs`](xtask/tests/vendor_openapi.rs) &mdash; 104 lines

</details>

<details>
<summary><code>xtask/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](xtask/tests/common/mod.rs) &mdash; 158 lines

</details>


