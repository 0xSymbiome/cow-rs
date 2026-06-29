# Repository File Map

> **Branch:** `feat/ferrous-foundation` &nbsp;&middot;&nbsp; **HEAD:** `47308c8a` &nbsp;&middot;&nbsp; **Generated:** 2026-06-29  
> **Total tracked files:** **885** &nbsp;&middot;&nbsp; **Lines of code:** tokei 14.0.0

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

- **36,199 lines of Rust** across the 15 SDK crates, covered by **35,577 lines of tests** — a **1.0× test-to-code ratio** — plus **164 lines of benchmarks**.
- **12,390 doc-comment lines** documenting the public API (~34.2% of crate code), plus **968 inline comment lines**.
- **5,462 lines of TypeScript** across examples, e2e harnesses, and wasm bindings.
- **9,146 lines of Markdown prose** — ADRs, audit notes, and READMEs.
- **14,290 lines of data & config** — JSON schemas, parity fixtures, YAML, TOML, and lockfiles. Tracked and counted in the totals below; listed separately here because it's data, not hand-written code.

**Footprint** (tracked files)

- **562 files** live under `crates/` — 15 workspace member crates make up roughly 64% of the repo.
- **76 files** under `docs/` are mostly architecture decision records and audit notes.
- **40 files** under `parity/` are golden fixtures captured from upstream services to keep the Rust SDK byte-compatible.
- **45 files** under `fuzz/` cover cargo-fuzz targets and their seed corpora.
- **62 files** under `examples/` + `e2e/` are runnable demos and integration harnesses.
- **46 files** under `xtask/` are the maintenance automation crate (parity refresh, policy checks, doc generation).

---

## Top-level layout

| Path | Files | Lines | Code | Purpose |
|------|------:|------:|-----:|---------|
| `crates/` | 562 | 117,602 | 79,566 | Workspace member crates (the SDK itself) |
| `docs/` | 76 | 7,165 | 0 | Architecture decision records, audit notes, provider notes |
| `xtask/` | 46 | 10,016 | 8,430 | Cargo xtask automation crate (parity, policy, docs subcommands) |
| `fuzz/` | 45 | 9,396 | 3,874 | cargo-fuzz targets, corpora, and failure artifacts |
| `parity/` | 40 | 5,771 | 5,455 | Golden fixtures + pinned specs from upstream services |
| `examples/` | 36 | 3,703 | 2,574 | Runnable usage examples (Rust + TypeScript) |
| `e2e/` | 26 | 3,694 | 2,984 | End-to-end integration harnesses |
| `.github/` | 24 | 3,523 | 2,983 | GitHub Actions workflows and repo config |
| `tests/` | 12 | 1,129 | 958 | Workspace-level integration tests |
| `.cargo/` | 2 | 35 | 30 | Cargo configuration |
| `SECURITY.md` | 1 | 183 | 0 | Security policy |
| `rust-toolchain.toml` | 1 | 6 | 4 | Pinned Rust toolchain |
| `ROADMAP.md` | 1 | 82 | 0 | Roadmap document |
| `release.toml` | 1 | 56 | 11 |  |
| `README.md` | 1 | 254 | 0 | Top-level README |
| `.gitignore` | 1 | 28 | 0 | Top-level git ignore rules |
| `llvm-cov-summary.txt` | 1 | 186 | 0 | Coverage summary snapshot |
| `LICENSE` | 1 | 674 | 0 | License text |
| `.yamllint` | 1 | 7 | 0 | YAML lint configuration |
| `.githooks/` | 1 | 35 | 28 | Tracked git hook scripts |
| `Cargo.lock` | 1 | 5,938 | 0 | Workspace lockfile |
| `.gitattributes` | 1 | 21 | 0 | Git attributes |
| `CONTRIBUTING.md` | 1 | 283 | 0 | Contribution guide |
| `cliff.toml` | 1 | 69 | 48 |  |
| `Cargo.toml` | 1 | 122 | 107 | Workspace manifest |
| `CHANGELOG.md` | 1 | 78 | 0 | Release changelog |
| **Total** | **885** | **170,056** | **107,052** | |

---

## File composition by extension

| Extension | Files | Lines | Code | Comments | Blank | Typical role |
|-----------|------:|------:|-----:|---------:|------:|--------------|
| `.rs` | 572 | 116,114 | 87,300 | 18,302 | 10,512 | Rust source and tests |
| `.md` | 101 | 10,958 | 0 | 9,146 | 1,812 | Markdown docs (ADRs, audit notes, READMEs) |
| `.ts` | 49 | 13,866 | 5,462 | 7,508 | 896 | TypeScript (examples, e2e, wasm bindings) |
| `.json` | 44 | 2,673 | 2,673 | 0 | 0 | JSON schemas, parity fixtures, test vectors |
| `.toml` | 29 | 2,167 | 1,580 | 328 | 259 | Cargo manifests and tool configs |
| `.stderr` | 25 | 570 | 0 | 549 | 21 | trybuild compile-fail snapshots |
| `.yml` | 17 | 5,416 | 4,960 | 300 | 156 | CI workflows and config |
| `.yaml` | 11 | 4,778 | 3,919 | 48 | 811 | CI workflows, OpenAPI specs, config |
| `.txt` | 7 | 227 | 0 | 226 | 1 | Plain text fixtures / summaries |
| `.mjs` | 6 | 810 | 614 | 100 | 96 | JavaScript modules |
| `.sh` | 4 | 555 | 480 | 11 | 64 | Shell scripts |
| `(none)` | 3 | 1,383 | 28 | 1,107 | 248 |  |
| `.graphql` | 3 | 24 | 24 | 0 | 0 | GraphQL queries (subgraph) |
| `.bin` | 2 | 0 | 0 | 0 | 0 | Binary fixtures |
| `.lock` | 2 | 9,694 | 0 | 8,764 | 930 | Cargo / package lockfiles |
| `.keep` | 2 | 2 | 0 | 0 | 2 |  |
| `.gitignore` | 2 | 31 | 0 | 26 | 5 |  |
| `.html` | 1 | 12 | 12 | 0 | 0 | Static HTML for browser examples |
| `.wit` | 1 | 735 | 0 | 652 | 83 |  |
| `.gitattributes` | 1 | 21 | 0 | 18 | 3 |  |
| `.yamllint` | 1 | 7 | 0 | 6 | 1 |  |
| `.npmignore` | 1 | 6 | 0 | 6 | 0 |  |
| `.proptest-regressions` | 1 | 7 | 0 | 7 | 0 | proptest regression seeds |
| **Total** | **885** | **170,056** | **107,052** | **47,104** | **15,900** | |

> **Code + Comments + Blank = Lines** for every row. ``Comments`` is all non-code, non-blank content: inline + doc-comments in source, prose in Markdown/text, and raw content in formats tokei does not parse as code (lockfiles, ``.stderr``, snapshots). Rust doc-comments are isolated in the per-crate ``Doc`` column above.

---

## Workspace crates (`crates/`)

15 member crates compose the SDK. `Code` is Rust `src/` code; `Tests` and `Benches` are Rust lines under `tests/` and `benches/`; `Doc` is `src/` doc-comment lines (`///` / `//!`) — the public-API documentation surface; `T:C` is the test-to-code ratio. Descriptions are pulled live from each crate's `Cargo.toml`.

| Crate | Files | Code | Tests | Benches | Doc | T:C | Purpose |
|-------|------:|-----:|------:|--------:|----:|----:|---------|
| [`core`](crates/core) | 75 | 5,768 | 4,103 | 0 | 2,366 | 0.7× | Shared CoW Protocol core types and validation primitives |
| [`js`](crates/js) | 115 | 5,578 | 4,800 | 0 | 1,369 | 0.9× | The CoW Protocol Rust SDK compiled to wasm for JavaScript and TypeScript with wasm-bindgen; shipped to npm, not crates.io |
| [`trading`](crates/trading) | 57 | 5,315 | 7,054 | 46 | 1,916 | 1.3× | High-level CoW Protocol trading orchestration surface |
| [`orderbook`](crates/orderbook) | 41 | 4,653 | 5,558 | 0 | 1,834 | 1.2× | Typed CoW Protocol orderbook client models and decoding helpers |
| [`contracts`](crates/contracts) | 64 | 4,128 | 3,900 | 59 | 1,997 | 0.9× | CoW Protocol low-level contracts helpers for order hashing, signature codecs and verification, ABI bindings, and fail-closed on-chain event decoding |
| [`component`](crates/component) | 16 | 2,937 | 0 | 0 | 240 | 0.0× | The CoW Protocol Rust SDK as a WebAssembly Component (WASI 0.2 and 0.3); built to wasm32-wasip2 and distributed via OCI, not crates.io |
| [`app-data`](crates/app-data) | 41 | 1,444 | 2,181 | 33 | 742 | 1.5× | CoW Protocol app-data encoding, validation, and CID compatibility |
| [`alloy-provider`](crates/alloy-provider) | 27 | 1,290 | 1,516 | 0 | 209 | 1.2× | Alloy-backed read-only Provider adapter for the CoW Protocol Rust SDK |
| [`subgraph`](crates/subgraph) | 26 | 1,236 | 2,154 | 0 | 483 | 1.7× | Typed CoW Protocol subgraph query primitives |
| [`test-utils`](crates/test-utils) | 10 | 800 | 143 | 0 | 236 | 0.2× | Internal, unpublished shared test helpers for the cow-rs workspace. |
| [`signing`](crates/signing) | 23 | 787 | 1,065 | 26 | 231 | 1.4× | Deterministic CoW Protocol order hashing, EIP-712 signing, and UID helpers |
| [`test`](crates/test) | 9 | 725 | 283 | 0 | 228 | 0.4× | In-memory test doubles for the cow-rs SDK public traits (OrderbookClient, Signer, Provider) so downstream applications can test their CoW Protocol integration without a live orderbook, RPC endpoint, or wallet. |
| [`alloy`](crates/alloy) | 27 | 719 | 1,140 | 0 | 205 | 1.6× | Composed Alloy provider and signer adapter for the CoW Protocol Rust SDK |
| [`alloy-signer`](crates/alloy-signer) | 23 | 710 | 534 | 0 | 162 | 0.8× | Alloy-backed local private-key Signer adapter for the CoW Protocol Rust SDK |
| [`sdk`](crates/sdk) | 8 | 109 | 1,146 | 0 | 172 | 10.5× | Facade crate for CoW Protocol Rust SDK surfaces |
| **Total** | **562** | **36,199** | **35,577** | **164** | **12,390** | **1.0×** | |

---

## Source hotspots

The 25 largest hand-written source files by code lines (Rust + TypeScript). This is where complexity — and review attention — concentrates.

| File | Lang | Kind | Code | Comments |
|------|------|------|-----:|---------:|
| [`xtask/src/parity/mod.rs`](xtask/src/parity/mod.rs) | Rust | src | 1,190 | 126 |
| [`crates/orderbook/tests/api_contract.rs`](crates/orderbook/tests/api_contract.rs) | Rust | test | 1,032 | 23 |
| [`crates/subgraph/tests/api_contract.rs`](crates/subgraph/tests/api_contract.rs) | Rust | test | 1,012 | 6 |
| [`crates/trading/tests/common/mod.rs`](crates/trading/tests/common/mod.rs) | Rust | test | 857 | 36 |
| [`crates/orderbook/tests/request_contract.rs`](crates/orderbook/tests/request_contract.rs) | Rust | test | 856 | 21 |
| [`crates/trading/src/types/params.rs`](crates/trading/src/types/params.rs) | Rust | src | 851 | 281 |
| [`crates/trading/tests/quote_contract.rs`](crates/trading/tests/quote_contract.rs) | Rust | test | 788 | 14 |
| [`crates/orderbook/src/types/quote.rs`](crates/orderbook/src/types/quote.rs) | Rust | src | 775 | 299 |
| [`xtask/src/parity/openapi_coverage.rs`](xtask/src/parity/openapi_coverage.rs) | Rust | src | 771 | 17 |
| [`crates/sdk/tests/error_redaction_contract.rs`](crates/sdk/tests/error_redaction_contract.rs) | Rust | test | 771 | 58 |
| [`crates/trading/tests/post_contract.rs`](crates/trading/tests/post_contract.rs) | Rust | test | 759 | 74 |
| [`crates/js/src/exports/trading.rs`](crates/js/src/exports/trading.rs) | Rust | src | 705 | 185 |
| [`crates/orderbook/src/types/order.rs`](crates/orderbook/src/types/order.rs) | Rust | src | 701 | 270 |
| [`crates/js/npm/src/default.ts`](crates/js/npm/src/default.ts) | TypeScript | src | 697 | 22 |
| [`crates/js/snapshots/raw/default.d.ts`](crates/js/snapshots/raw/default.d.ts) | TypeScript | src | 648 | 2,484 |
| [`crates/js/src/exports/orderbook.rs`](crates/js/src/exports/orderbook.rs) | Rust | src | 626 | 193 |
| [`crates/js/snapshots/raw/trading.d.ts`](crates/js/snapshots/raw/trading.d.ts) | TypeScript | src | 619 | 2,383 |
| [`crates/trading/src/slippage.rs`](crates/trading/src/slippage.rs) | Rust | src | 615 | 144 |
| [`crates/core/tests/transport_contract.rs`](crates/core/tests/transport_contract.rs) | Rust | test | 613 | 27 |
| [`crates/trading/src/post.rs`](crates/trading/src/post.rs) | Rust | src | 604 | 127 |
| [`crates/trading/tests/sdk_contract.rs`](crates/trading/tests/sdk_contract.rs) | Rust | test | 601 | 8 |
| [`crates/js/src/exports/errors.rs`](crates/js/src/exports/errors.rs) | Rust | src | 597 | 150 |
| [`crates/core/tests/policy_contract.rs`](crates/core/tests/policy_contract.rs) | Rust | test | 580 | 64 |
| [`crates/js/npm/src/trading.ts`](crates/js/npm/src/trading.ts) | TypeScript | src | 580 | 22 |
| [`crates/orderbook/src/api.rs`](crates/orderbook/src/api.rs) | Rust | src | 578 | 252 |

---

## Examples (`examples/`)

| Example | Files | Lines | Code | Purpose |
|---------|------:|------:|-----:|---------|
| [`native`](examples/native) | 35 | 3,660 | 2,574 | Native Rust scenario walkthroughs |
| **Total (listed)** | **35** | **3,660** | **2,574** | |

---

## End-to-end harnesses (`e2e/`)

| Harness | Files | Lines | Code | Purpose |
|---------|------:|------:|-----:|---------|
| [`wasm-typescript`](e2e/wasm-typescript) | 14 | 1,793 | 1,452 | Wasm + TypeScript integration harness |
| [`wasm-typescript-cf`](e2e/wasm-typescript-cf) | 12 | 1,901 | 1,532 | Wasm + TypeScript Cloudflare harness |
| **Total (listed)** | **26** | **3,694** | **2,984** | |

---

## Upstream parity (`parity/`)

| Subtree | Files | Lines | Code | Purpose |
|---------|------:|------:|-----:|---------|
| [`fixtures`](parity/fixtures) | 36 | 2,430 | 2,430 | Golden fixtures captured from upstream services |
| [`openapi`](parity/openapi) | 2 | 2,986 | 2,946 | OpenAPI specs pinned for parity |
| **Total (listed)** | **38** | **5,416** | **5,376** | |

---

## Documentation (`docs/`)

| Subtree | Files | Lines | Code | Purpose |
|---------|------:|------:|-----:|---------|
| [`adr`](docs/adr) | 56 | 6,015 | 0 | Architecture Decision Records |
| [`audit`](docs/audit) | 18 | 870 | 0 | Audit notes and review artifacts |
| [`providers`](docs/providers) | 1 | 209 | 0 | Provider integration notes |
| **Total (listed)** | **75** | **7,094** | **0** | |

---

## Fuzzing (`fuzz/`)

| Subtree | Files | Lines | Code | Purpose |
|---------|------:|------:|-----:|---------|
| [`fuzz_targets`](fuzz/fuzz_targets) | 42 | 5,134 | 3,598 | cargo-fuzz target sources |
| **Total (listed)** | **42** | **5,134** | **3,598** | |

---

## CI & repo-level configuration

| Path | Files | Purpose |
|------|------:|---------|
| `.github/workflows/` | 14 | GitHub Actions pipelines |
| `.github/config/`    | 8 | Shared CI config |
| `.githooks/`         | 1 | Tracked git hooks |
| `.cargo/`            | 2 | Cargo config (e.g. rustflags) |
| `tests/`             | 12 | Workspace-level integration tests |

---

## Full file index

Every tracked file, grouped by the directory it lives in. Each section is collapsed by default — click to expand. The number after each file is its total line count.

<details>
<summary><code>(repo root)</code> &mdash; 15 file(s)</summary>

- [`.gitattributes`](.gitattributes) &mdash; 21 lines
- [`.gitignore`](.gitignore) &mdash; 28 lines
- [`.yamllint`](.yamllint) &mdash; 7 lines
- [`Cargo.lock`](Cargo.lock) &mdash; 5,938 lines
- [`Cargo.toml`](Cargo.toml) &mdash; 122 lines
- [`CHANGELOG.md`](CHANGELOG.md) &mdash; 78 lines
- [`cliff.toml`](cliff.toml) &mdash; 69 lines
- [`CONTRIBUTING.md`](CONTRIBUTING.md) &mdash; 283 lines
- [`LICENSE`](LICENSE) &mdash; 674 lines
- [`llvm-cov-summary.txt`](llvm-cov-summary.txt) &mdash; 186 lines
- [`README.md`](README.md) &mdash; 254 lines
- [`release.toml`](release.toml) &mdash; 56 lines
- [`ROADMAP.md`](ROADMAP.md) &mdash; 82 lines
- [`rust-toolchain.toml`](rust-toolchain.toml) &mdash; 6 lines
- [`SECURITY.md`](SECURITY.md) &mdash; 183 lines

</details>

<details>
<summary><code>.cargo/</code> &mdash; 2 file(s)</summary>

- [`config.toml`](.cargo/config.toml) &mdash; 32 lines
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

- [`audit-refresh-map.yml`](.github/config/audit-refresh-map.yml) &mdash; 141 lines
- [`deny-unknown-fields-allowlist.yaml`](.github/config/deny-unknown-fields-allowlist.yaml) &mdash; 20 lines
- [`deny.toml`](.github/config/deny.toml) &mdash; 152 lines
- [`enum-policy.yaml`](.github/config/enum-policy.yaml) &mdash; 448 lines
- [`nextest.toml`](.github/config/nextest.toml) &mdash; 38 lines
- [`panic-allowlist.yaml`](.github/config/panic-allowlist.yaml) &mdash; 80 lines
- [`principle-adr-map.yaml`](.github/config/principle-adr-map.yaml) &mdash; 132 lines
- [`typos.toml`](.github/config/typos.toml) &mdash; 30 lines

</details>

<details>
<summary><code>.github/workflows/</code> &mdash; 14 file(s)</summary>

- [`_quality-gate.yml`](.github/workflows/_quality-gate.yml) &mdash; 349 lines
- [`alloy-release-candidate.yml`](.github/workflows/alloy-release-candidate.yml) &mdash; 133 lines
- [`benchmarks.yml`](.github/workflows/benchmarks.yml) &mdash; 69 lines
- [`ci.yml`](.github/workflows/ci.yml) &mdash; 315 lines
- [`codeql.yml`](.github/workflows/codeql.yml) &mdash; 55 lines
- [`commit-format.yml`](.github/workflows/commit-format.yml) &mdash; 98 lines
- [`component.yml`](.github/workflows/component.yml) &mdash; 113 lines
- [`crate-checks.yml`](.github/workflows/crate-checks.yml) &mdash; 99 lines
- [`docs-quality.yml`](.github/workflows/docs-quality.yml) &mdash; 88 lines
- [`fuzz.yml`](.github/workflows/fuzz.yml) &mdash; 79 lines
- [`release-readiness.yml`](.github/workflows/release-readiness.yml) &mdash; 349 lines
- [`retry-soak.yml`](.github/workflows/retry-soak.yml) &mdash; 35 lines
- [`upstream-drift.yml`](.github/workflows/upstream-drift.yml) &mdash; 62 lines
- [`wasm.yml`](.github/workflows/wasm.yml) &mdash; 601 lines

</details>

<details>
<summary><code>crates/alloy/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/alloy/Cargo.toml) &mdash; 49 lines
- [`README.md`](crates/alloy/README.md) &mdash; 147 lines

</details>

<details>
<summary><code>crates/alloy-provider/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/alloy-provider/Cargo.toml) &mdash; 50 lines
- [`README.md`](crates/alloy-provider/README.md) &mdash; 135 lines

</details>

<details>
<summary><code>crates/alloy-provider/src/</code> &mdash; 8 file(s)</summary>

- [`builder.rs`](crates/alloy-provider/src/builder.rs) &mdash; 198 lines
- [`client.rs`](crates/alloy-provider/src/client.rs) &mdash; 29 lines
- [`conversion.rs`](crates/alloy-provider/src/conversion.rs) &mdash; 326 lines
- [`error.rs`](crates/alloy-provider/src/error.rs) &mdash; 264 lines
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

- [`Cargo.toml`](crates/alloy-signer/Cargo.toml) &mdash; 44 lines
- [`README.md`](crates/alloy-signer/README.md) &mdash; 136 lines

</details>

<details>
<summary><code>crates/alloy-signer/src/</code> &mdash; 5 file(s)</summary>

- [`builder.rs`](crates/alloy-signer/src/builder.rs) &mdash; 291 lines
- [`conversion.rs`](crates/alloy-signer/src/conversion.rs) &mdash; 295 lines
- [`error.rs`](crates/alloy-signer/src/error.rs) &mdash; 211 lines
- [`lib.rs`](crates/alloy-signer/src/lib.rs) &mdash; 65 lines
- [`signer.rs`](crates/alloy-signer/src/signer.rs) &mdash; 130 lines

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
- [`error.rs`](crates/alloy/src/error.rs) &mdash; 201 lines
- [`handle.rs`](crates/alloy/src/handle.rs) &mdash; 120 lines
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
- [`error_contract.rs`](crates/alloy/tests/error_contract.rs) &mdash; 125 lines
- [`handle_survives_drop.rs`](crates/alloy/tests/handle_survives_drop.rs) &mdash; 32 lines
- [`log_provider_contract.rs`](crates/alloy/tests/log_provider_contract.rs) &mdash; 40 lines
- [`provider_contract.rs`](crates/alloy/tests/provider_contract.rs) &mdash; 197 lines
- [`read_contract_contract.rs`](crates/alloy/tests/read_contract_contract.rs) &mdash; 129 lines
- [`redaction_contract.rs`](crates/alloy/tests/redaction_contract.rs) &mdash; 127 lines
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

- [`Cargo.toml`](crates/app-data/Cargo.toml) &mdash; 70 lines
- [`README.md`](crates/app-data/README.md) &mdash; 160 lines

</details>

<details>
<summary><code>crates/app-data/benches/</code> &mdash; 1 file(s)</summary>

- [`stringify.rs`](crates/app-data/benches/stringify.rs) &mdash; 38 lines

</details>

<details>
<summary><code>crates/app-data/src/</code> &mdash; 6 file(s)</summary>

- [`cid.rs`](crates/app-data/src/cid.rs) &mdash; 143 lines
- [`errors.rs`](crates/app-data/src/errors.rs) &mdash; 217 lines
- [`fetch.rs`](crates/app-data/src/fetch.rs) &mdash; 209 lines
- [`info.rs`](crates/app-data/src/info.rs) &mdash; 349 lines
- [`lib.rs`](crates/app-data/src/lib.rs) &mdash; 63 lines
- [`schema.rs`](crates/app-data/src/schema.rs) &mdash; 158 lines

</details>

<details>
<summary><code>crates/app-data/src/metadata/</code> &mdash; 4 file(s)</summary>

- [`flashloan.rs`](crates/app-data/src/metadata/flashloan.rs) &mdash; 109 lines
- [`hooks.rs`](crates/app-data/src/metadata/hooks.rs) &mdash; 80 lines
- [`mod.rs`](crates/app-data/src/metadata/mod.rs) &mdash; 18 lines
- [`quote.rs`](crates/app-data/src/metadata/quote.rs) &mdash; 90 lines

</details>

<details>
<summary><code>crates/app-data/src/types/</code> &mdash; 6 file(s)</summary>

- [`doc.rs`](crates/app-data/src/types/doc.rs) &mdash; 126 lines
- [`ipfs.rs`](crates/app-data/src/types/ipfs.rs) &mdash; 13 lines
- [`mod.rs`](crates/app-data/src/types/mod.rs) &mdash; 19 lines
- [`params.rs`](crates/app-data/src/types/params.rs) &mdash; 327 lines
- [`partner_fee.rs`](crates/app-data/src/types/partner_fee.rs) &mdash; 452 lines
- [`validation.rs`](crates/app-data/src/types/validation.rs) &mdash; 39 lines

</details>

<details>
<summary><code>crates/app-data/tests/</code> &mdash; 18 file(s)</summary>

- [`app_data_info_contract.rs`](crates/app-data/tests/app_data_info_contract.rs) &mdash; 44 lines
- [`canonical_json_contract.rs`](crates/app-data/tests/canonical_json_contract.rs) &mdash; 44 lines
- [`cid_contract.rs`](crates/app-data/tests/cid_contract.rs) &mdash; 105 lines
- [`error_contract.rs`](crates/app-data/tests/error_contract.rs) &mdash; 13 lines
- [`error_variant_shape.rs`](crates/app-data/tests/error_variant_shape.rs) &mdash; 97 lines
- [`fetch_contract.rs`](crates/app-data/tests/fetch_contract.rs) &mdash; 241 lines
- [`fetch_telemetry_contract.rs`](crates/app-data/tests/fetch_telemetry_contract.rs) &mdash; 80 lines
- [`flashloan_contract.rs`](crates/app-data/tests/flashloan_contract.rs) &mdash; 295 lines
- [`hooks_contract.rs`](crates/app-data/tests/hooks_contract.rs) &mdash; 159 lines
- [`ipfs_config_redaction_contract.rs`](crates/app-data/tests/ipfs_config_redaction_contract.rs) &mdash; 51 lines
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
<summary><code>crates/component/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/component/Cargo.toml) &mdash; 102 lines
- [`README.md`](crates/component/README.md) &mdash; 42 lines

</details>

<details>
<summary><code>crates/component/src/</code> &mdash; 1 file(s)</summary>

- [`lib.rs`](crates/component/src/lib.rs) &mdash; 68 lines

</details>

<details>
<summary><code>crates/component/src/client/</code> &mdash; 5 file(s)</summary>

- [`async.rs`](crates/component/src/client/async.rs) &mdash; 634 lines
- [`core.rs`](crates/component/src/client/core.rs) &mdash; 583 lines
- [`mod.rs`](crates/component/src/client/mod.rs) &mdash; 37 lines
- [`orderbook.rs`](crates/component/src/client/orderbook.rs) &mdash; 574 lines
- [`sync.rs`](crates/component/src/client/sync.rs) &mdash; 593 lines

</details>

<details>
<summary><code>crates/component/src/engine/</code> &mdash; 7 file(s)</summary>

- [`composable.rs`](crates/component/src/engine/composable.rs) &mdash; 105 lines
- [`events.rs`](crates/component/src/engine/events.rs) &mdash; 34 lines
- [`mod.rs`](crates/component/src/engine/mod.rs) &mdash; 67 lines
- [`signing.rs`](crates/component/src/engine/signing.rs) &mdash; 73 lines
- [`tests.rs`](crates/component/src/engine/tests.rs) &mdash; 204 lines
- [`tx.rs`](crates/component/src/engine/tx.rs) &mdash; 131 lines
- [`world.rs`](crates/component/src/engine/world.rs) &mdash; 428 lines

</details>

<details>
<summary><code>crates/component/wit/</code> &mdash; 1 file(s)</summary>

- [`world.wit`](crates/component/wit/world.wit) &mdash; 735 lines

</details>

<details>
<summary><code>crates/contracts/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/contracts/Cargo.toml) &mdash; 66 lines
- [`README.md`](crates/contracts/README.md) &mdash; 119 lines

</details>

<details>
<summary><code>crates/contracts/benches/</code> &mdash; 2 file(s)</summary>

- [`order_hashing.rs`](crates/contracts/benches/order_hashing.rs) &mdash; 26 lines
- [`uid_packing.rs`](crates/contracts/benches/uid_packing.rs) &mdash; 42 lines

</details>

<details>
<summary><code>crates/contracts/src/</code> &mdash; 14 file(s)</summary>

- [`deployments.rs`](crates/contracts/src/deployments.rs) &mdash; 420 lines
- [`errors.rs`](crates/contracts/src/errors.rs) &mdash; 237 lines
- [`eth_flow.rs`](crates/contracts/src/eth_flow.rs) &mdash; 617 lines
- [`hex_field.rs`](crates/contracts/src/hex_field.rs) &mdash; 234 lines
- [`interaction.rs`](crates/contracts/src/interaction.rs) &mdash; 133 lines
- [`lib.rs`](crates/contracts/src/lib.rs) &mdash; 105 lines
- [`onchain_orders.rs`](crates/contracts/src/onchain_orders.rs) &mdash; 326 lines
- [`order.rs`](crates/contracts/src/order.rs) &mdash; 505 lines
- [`primitives.rs`](crates/contracts/src/primitives.rs) &mdash; 254 lines
- [`settlement.rs`](crates/contracts/src/settlement.rs) &mdash; 347 lines
- [`signature.rs`](crates/contracts/src/signature.rs) &mdash; 594 lines
- [`tokens.rs`](crates/contracts/src/tokens.rs) &mdash; 234 lines
- [`tx.rs`](crates/contracts/src/tx.rs) &mdash; 277 lines
- [`verify.rs`](crates/contracts/src/verify.rs) &mdash; 241 lines

</details>

<details>
<summary><code>crates/contracts/src/composable/</code> &mdash; 4 file(s)</summary>

- [`mod.rs`](crates/contracts/src/composable/mod.rs) &mdash; 48 lines
- [`multiplexer.rs`](crates/contracts/src/composable/multiplexer.rs) &mdash; 265 lines
- [`registry.rs`](crates/contracts/src/composable/registry.rs) &mdash; 202 lines
- [`twap.rs`](crates/contracts/src/composable/twap.rs) &mdash; 785 lines

</details>

<details>
<summary><code>crates/contracts/src/cow_shed/</code> &mdash; 8 file(s)</summary>

- [`bindings.rs`](crates/contracts/src/cow_shed/bindings.rs) &mdash; 163 lines
- [`calls.rs`](crates/contracts/src/cow_shed/calls.rs) &mdash; 66 lines
- [`eip712.rs`](crates/contracts/src/cow_shed/eip712.rs) &mdash; 168 lines
- [`errors.rs`](crates/contracts/src/cow_shed/errors.rs) &mdash; 29 lines
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
- [`error_contract.rs`](crates/contracts/tests/error_contract.rs) &mdash; 228 lines
- [`eth_flow_events_contract.rs`](crates/contracts/tests/eth_flow_events_contract.rs) &mdash; 143 lines
- [`interaction_contract.rs`](crates/contracts/tests/interaction_contract.rs) &mdash; 79 lines
- [`non_exhaustive_dto_contract.rs`](crates/contracts/tests/non_exhaustive_dto_contract.rs) &mdash; 119 lines
- [`onchain_orders.rs`](crates/contracts/tests/onchain_orders.rs) &mdash; 296 lines
- [`order_contract.rs`](crates/contracts/tests/order_contract.rs) &mdash; 125 lines
- [`order_digest_parity_contract.rs`](crates/contracts/tests/order_digest_parity_contract.rs) &mdash; 154 lines
- [`parity_contract.rs`](crates/contracts/tests/parity_contract.rs) &mdash; 550 lines
- [`property_contract.rs`](crates/contracts/tests/property_contract.rs) &mdash; 441 lines
- [`proxy_address_parity_contract.rs`](crates/contracts/tests/proxy_address_parity_contract.rs) &mdash; 134 lines
- [`recoverable_signature_contract.rs`](crates/contracts/tests/recoverable_signature_contract.rs) &mdash; 286 lines
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

- [`Cargo.toml`](crates/core/Cargo.toml) &mdash; 127 lines
- [`README.md`](crates/core/README.md) &mdash; 126 lines

</details>

<details>
<summary><code>crates/core/src/</code> &mdash; 4 file(s)</summary>

- [`cancellation.rs`](crates/core/src/cancellation.rs) &mdash; 122 lines
- [`errors.rs`](crates/core/src/errors.rs) &mdash; 244 lines
- [`lib.rs`](crates/core/src/lib.rs) &mdash; 157 lines
- [`validation.rs`](crates/core/src/validation.rs) &mdash; 116 lines

</details>

<details>
<summary><code>crates/core/src/config/</code> &mdash; 6 file(s)</summary>

- [`chains.rs`](crates/core/src/config/chains.rs) &mdash; 222 lines
- [`env.rs`](crates/core/src/config/env.rs) &mdash; 79 lines
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
- [`provider.rs`](crates/core/src/traits/provider.rs) &mdash; 246 lines
- [`signer.rs`](crates/core/src/traits/signer.rs) &mdash; 219 lines
- [`transaction.rs`](crates/core/src/traits/transaction.rs) &mdash; 225 lines
- [`typed_data.rs`](crates/core/src/traits/typed_data.rs) &mdash; 244 lines

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
- [`config.rs`](crates/core/src/transport/policy/config.rs) &mdash; 300 lines
- [`jitter.rs`](crates/core/src/transport/policy/jitter.rs) &mdash; 163 lines
- [`mod.rs`](crates/core/src/transport/policy/mod.rs) &mdash; 55 lines
- [`rate_limit.rs`](crates/core/src/transport/policy/rate_limit.rs) &mdash; 292 lines
- [`retry_after.rs`](crates/core/src/transport/policy/retry_after.rs) &mdash; 134 lines
- [`retry.rs`](crates/core/src/transport/policy/retry.rs) &mdash; 231 lines
- [`runner.rs`](crates/core/src/transport/policy/runner.rs) &mdash; 528 lines
- [`status.rs`](crates/core/src/transport/policy/status.rs) &mdash; 47 lines
- [`time.rs`](crates/core/src/transport/policy/time.rs) &mdash; 66 lines

</details>

<details>
<summary><code>crates/core/src/types/</code> &mdash; 8 file(s)</summary>

- [`amount.rs`](crates/core/src/types/amount.rs) &mdash; 521 lines
- [`app_code.rs`](crates/core/src/types/app_code.rs) &mdash; 206 lines
- [`identity.rs`](crates/core/src/types/identity.rs) &mdash; 932 lines
- [`logs.rs`](crates/core/src/types/logs.rs) &mdash; 281 lines
- [`mod.rs`](crates/core/src/types/mod.rs) &mdash; 72 lines
- [`order.rs`](crates/core/src/types/order.rs) &mdash; 305 lines
- [`quote.rs`](crates/core/src/types/quote.rs) &mdash; 176 lines
- [`validity.rs`](crates/core/src/types/validity.rs) &mdash; 82 lines

</details>

<details>
<summary><code>crates/core/tests/</code> &mdash; 21 file(s)</summary>

- [`address_literal_ui.rs`](crates/core/tests/address_literal_ui.rs) &mdash; 17 lines
- [`amount_arithmetic_ui.rs`](crates/core/tests/amount_arithmetic_ui.rs) &mdash; 30 lines
- [`cancellation_contract.rs`](crates/core/tests/cancellation_contract.rs) &mdash; 126 lines
- [`cancellation_coverage_validator.rs`](crates/core/tests/cancellation_coverage_validator.rs) &mdash; 232 lines
- [`classify_contract.rs`](crates/core/tests/classify_contract.rs) &mdash; 48 lines
- [`config_contract.rs`](crates/core/tests/config_contract.rs) &mdash; 238 lines
- [`policy_contract.rs`](crates/core/tests/policy_contract.rs) &mdash; 731 lines
- [`property_contract.rs`](crates/core/tests/property_contract.rs) &mdash; 583 lines
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
- [`types_contract.rs`](crates/core/tests/types_contract.rs) &mdash; 637 lines
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
<summary><code>crates/js/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/js/Cargo.toml) &mdash; 150 lines
- [`README.md`](crates/js/README.md) &mdash; 157 lines

</details>

<details>
<summary><code>crates/js/npm/</code> &mdash; 10 file(s)</summary>

- [`.gitignore`](crates/js/npm/.gitignore) &mdash; 3 lines
- [`.npmignore`](crates/js/npm/.npmignore) &mdash; 6 lines
- [`flavours.json`](crates/js/npm/flavours.json) &mdash; 77 lines
- [`LICENSE`](crates/js/npm/LICENSE) &mdash; 674 lines
- [`package.template.json`](crates/js/npm/package.template.json) &mdash; 54 lines
- [`pnpm-lock.yaml`](crates/js/npm/pnpm-lock.yaml) &mdash; 771 lines
- [`README.md`](crates/js/npm/README.md) &mdash; 336 lines
- [`tsconfig.facade.json`](crates/js/npm/tsconfig.facade.json) &mdash; 5 lines
- [`tsconfig.json`](crates/js/npm/tsconfig.json) &mdash; 24 lines
- [`vitest.config.ts`](crates/js/npm/vitest.config.ts) &mdash; 9 lines

</details>

<details>
<summary><code>crates/js/npm/scripts/</code> &mdash; 10 file(s)</summary>

- [`build.sh`](crates/js/npm/scripts/build.sh) &mdash; 212 lines
- [`compile-facade.sh`](crates/js/npm/scripts/compile-facade.sh) &mdash; 252 lines
- [`dedupe-target-wasm.mjs`](crates/js/npm/scripts/dedupe-target-wasm.mjs) &mdash; 171 lines
- [`measure-wasm-size.mjs`](crates/js/npm/scripts/measure-wasm-size.mjs) &mdash; 175 lines
- [`pack-and-resolve-tarball.sh`](crates/js/npm/scripts/pack-and-resolve-tarball.sh) &mdash; 22 lines
- [`render-package-json.mjs`](crates/js/npm/scripts/render-package-json.mjs) &mdash; 177 lines
- [`verify-exports.mjs`](crates/js/npm/scripts/verify-exports.mjs) &mdash; 150 lines
- [`verify-facade-denylist.mjs`](crates/js/npm/scripts/verify-facade-denylist.mjs) &mdash; 81 lines
- [`verify-no-raw-exports.mjs`](crates/js/npm/scripts/verify-no-raw-exports.mjs) &mdash; 56 lines
- [`verify-package-resolution.sh`](crates/js/npm/scripts/verify-package-resolution.sh) &mdash; 69 lines

</details>

<details>
<summary><code>crates/js/npm/src/</code> &mdash; 11 file(s)</summary>

- [`callbacks.ts`](crates/js/npm/src/callbacks.ts) &mdash; 106 lines
- [`default.ts`](crates/js/npm/src/default.ts) &mdash; 809 lines
- [`envelope.ts`](crates/js/npm/src/envelope.ts) &mdash; 6 lines
- [`errors.ts`](crates/js/npm/src/errors.ts) &mdash; 387 lines
- [`index.ts`](crates/js/npm/src/index.ts) &mdash; 1 lines
- [`internal.ts`](crates/js/npm/src/internal.ts) &mdash; 160 lines
- [`options.ts`](crates/js/npm/src/options.ts) &mdash; 79 lines
- [`orderbook.ts`](crates/js/npm/src/orderbook.ts) &mdash; 433 lines
- [`retry.ts`](crates/js/npm/src/retry.ts) &mdash; 83 lines
- [`signing.ts`](crates/js/npm/src/signing.ts) &mdash; 165 lines
- [`trading.ts`](crates/js/npm/src/trading.ts) &mdash; 675 lines

</details>

<details>
<summary><code>crates/js/npm/src/raw/</code> &mdash; 4 file(s)</summary>

- [`default.ts`](crates/js/npm/src/raw/default.ts) &mdash; 44 lines
- [`orderbook.ts`](crates/js/npm/src/raw/orderbook.ts) &mdash; 33 lines
- [`signing.ts`](crates/js/npm/src/raw/signing.ts) &mdash; 27 lines
- [`trading.ts`](crates/js/npm/src/raw/trading.ts) &mdash; 41 lines

</details>

<details>
<summary><code>crates/js/npm/tests/</code> &mdash; 9 file(s)</summary>

- [`facade-cancellation.test.ts`](crates/js/npm/tests/facade-cancellation.test.ts) &mdash; 28 lines
- [`facade-default.test.ts`](crates/js/npm/tests/facade-default.test.ts) &mdash; 34 lines
- [`facade-error-normalization.test.ts`](crates/js/npm/tests/facade-error-normalization.test.ts) &mdash; 154 lines
- [`facade-error-shape.test.ts`](crates/js/npm/tests/facade-error-shape.test.ts) &mdash; 65 lines
- [`facade-orderbook.test.ts`](crates/js/npm/tests/facade-orderbook.test.ts) &mdash; 20 lines
- [`facade-resource-cleanup.test.ts`](crates/js/npm/tests/facade-resource-cleanup.test.ts) &mdash; 42 lines
- [`facade-retry.test.ts`](crates/js/npm/tests/facade-retry.test.ts) &mdash; 123 lines
- [`facade-signing.test.ts`](crates/js/npm/tests/facade-signing.test.ts) &mdash; 19 lines
- [`fixtures.ts`](crates/js/npm/tests/fixtures.ts) &mdash; 34 lines

</details>

<details>
<summary><code>crates/js/snapshots/facade/</code> &mdash; 5 file(s)</summary>

- [`.keep`](crates/js/snapshots/facade/.keep) &mdash; 1 lines
- [`default.d.ts`](crates/js/snapshots/facade/default.d.ts) &mdash; 134 lines
- [`orderbook.d.ts`](crates/js/snapshots/facade/orderbook.d.ts) &mdash; 75 lines
- [`signing.d.ts`](crates/js/snapshots/facade/signing.d.ts) &mdash; 41 lines
- [`trading.d.ts`](crates/js/snapshots/facade/trading.d.ts) &mdash; 115 lines

</details>

<details>
<summary><code>crates/js/snapshots/raw/</code> &mdash; 5 file(s)</summary>

- [`.keep`](crates/js/snapshots/raw/.keep) &mdash; 1 lines
- [`default.d.ts`](crates/js/snapshots/raw/default.d.ts) &mdash; 3,281 lines
- [`orderbook.d.ts`](crates/js/snapshots/raw/orderbook.d.ts) &mdash; 2,186 lines
- [`signing.d.ts`](crates/js/snapshots/raw/signing.d.ts) &mdash; 779 lines
- [`trading.d.ts`](crates/js/snapshots/raw/trading.d.ts) &mdash; 3,143 lines

</details>

<details>
<summary><code>crates/js/src/</code> &mdash; 1 file(s)</summary>

- [`lib.rs`](crates/js/src/lib.rs) &mdash; 40 lines

</details>

<details>
<summary><code>crates/js/src/dto/</code> &mdash; 10 file(s)</summary>

- [`app_data.rs`](crates/js/src/dto/app_data.rs) &mdash; 109 lines
- [`chains.rs`](crates/js/src/dto/chains.rs) &mdash; 72 lines
- [`composable.rs`](crates/js/src/dto/composable.rs) &mdash; 67 lines
- [`contracts.rs`](crates/js/src/dto/contracts.rs) &mdash; 30 lines
- [`events.rs`](crates/js/src/dto/events.rs) &mdash; 337 lines
- [`mod.rs`](crates/js/src/dto/mod.rs) &mdash; 184 lines
- [`orderbook.rs`](crates/js/src/dto/orderbook.rs) &mdash; 339 lines
- [`signing.rs`](crates/js/src/dto/signing.rs) &mdash; 131 lines
- [`trading.rs`](crates/js/src/dto/trading.rs) &mdash; 154 lines
- [`transport.rs`](crates/js/src/dto/transport.rs) &mdash; 383 lines

</details>

<details>
<summary><code>crates/js/src/exports/</code> &mdash; 16 file(s)</summary>

- [`callbacks.rs`](crates/js/src/exports/callbacks.rs) &mdash; 128 lines
- [`cancel.rs`](crates/js/src/exports/cancel.rs) &mdash; 245 lines
- [`chains.rs`](crates/js/src/exports/chains.rs) &mdash; 270 lines
- [`composable.rs`](crates/js/src/exports/composable.rs) &mdash; 87 lines
- [`eip1271.rs`](crates/js/src/exports/eip1271.rs) &mdash; 197 lines
- [`envelope.rs`](crates/js/src/exports/envelope.rs) &mdash; 37 lines
- [`errors.rs`](crates/js/src/exports/errors.rs) &mdash; 798 lines
- [`events.rs`](crates/js/src/exports/events.rs) &mdash; 61 lines
- [`ipfs.rs`](crates/js/src/exports/ipfs.rs) &mdash; 253 lines
- [`mod.rs`](crates/js/src/exports/mod.rs) &mdash; 139 lines
- [`orderbook.rs`](crates/js/src/exports/orderbook.rs) &mdash; 868 lines
- [`registry.rs`](crates/js/src/exports/registry.rs) &mdash; 95 lines
- [`signing.rs`](crates/js/src/exports/signing.rs) &mdash; 553 lines
- [`subgraph.rs`](crates/js/src/exports/subgraph.rs) &mdash; 231 lines
- [`trading.rs`](crates/js/src/exports/trading.rs) &mdash; 946 lines
- [`transport.rs`](crates/js/src/exports/transport.rs) &mdash; 488 lines

</details>

<details>
<summary><code>crates/js/src/helpers/</code> &mdash; 6 file(s)</summary>

- [`app_data.rs`](crates/js/src/helpers/app_data.rs) &mdash; 87 lines
- [`chains.rs`](crates/js/src/helpers/chains.rs) &mdash; 99 lines
- [`dto.rs`](crates/js/src/helpers/dto.rs) &mdash; 77 lines
- [`errors.rs`](crates/js/src/helpers/errors.rs) &mdash; 51 lines
- [`mod.rs`](crates/js/src/helpers/mod.rs) &mdash; 17 lines
- [`signing.rs`](crates/js/src/helpers/signing.rs) &mdash; 41 lines

</details>

<details>
<summary><code>crates/js/tests/</code> &mdash; 25 file(s)</summary>

- [`host_pure_helpers.rs`](crates/js/tests/host_pure_helpers.rs) &mdash; 261 lines
- [`no_ffi_helpers.rs`](crates/js/tests/no_ffi_helpers.rs) &mdash; 63 lines
- [`transport_fetch_contract.rs`](crates/js/tests/transport_fetch_contract.rs) &mdash; 376 lines
- [`transport_fetch_smoke.rs`](crates/js/tests/transport_fetch_smoke.rs) &mdash; 22 lines
- [`transport_parity_contract.rs`](crates/js/tests/transport_parity_contract.rs) &mdash; 539 lines
- [`wasm_callback_contract.rs`](crates/js/tests/wasm_callback_contract.rs) &mdash; 336 lines
- [`wasm_callback_lifetime_contract.rs`](crates/js/tests/wasm_callback_lifetime_contract.rs) &mdash; 55 lines
- [`wasm_callback_transport_contract.rs`](crates/js/tests/wasm_callback_transport_contract.rs) &mdash; 135 lines
- [`wasm_cancellation_contract.rs`](crates/js/tests/wasm_cancellation_contract.rs) &mdash; 239 lines
- [`wasm_eip1271_contract.rs`](crates/js/tests/wasm_eip1271_contract.rs) &mdash; 245 lines
- [`wasm_envelope_contract.rs`](crates/js/tests/wasm_envelope_contract.rs) &mdash; 33 lines
- [`wasm_error_abi_contract.rs`](crates/js/tests/wasm_error_abi_contract.rs) &mdash; 224 lines
- [`wasm_errortype_drift_contract.rs`](crates/js/tests/wasm_errortype_drift_contract.rs) &mdash; 97 lines
- [`wasm_facade_coverage_contract.rs`](crates/js/tests/wasm_facade_coverage_contract.rs) &mdash; 231 lines
- [`wasm_facade_snapshot_contract.rs`](crates/js/tests/wasm_facade_snapshot_contract.rs) &mdash; 161 lines
- [`wasm_fail_closed_contract.rs`](crates/js/tests/wasm_fail_closed_contract.rs) &mdash; 237 lines
- [`wasm_flavour_reachability_contract.rs`](crates/js/tests/wasm_flavour_reachability_contract.rs) &mdash; 217 lines
- [`wasm_ipfs_contract.rs`](crates/js/tests/wasm_ipfs_contract.rs) &mdash; 181 lines
- [`wasm_redaction_contract.rs`](crates/js/tests/wasm_redaction_contract.rs) &mdash; 123 lines
- [`wasm_retry_runner_contract.rs`](crates/js/tests/wasm_retry_runner_contract.rs) &mdash; 69 lines
- [`wasm_snapshot_surface_contract.rs`](crates/js/tests/wasm_snapshot_surface_contract.rs) &mdash; 446 lines
- [`wasm_surface_contract.rs`](crates/js/tests/wasm_surface_contract.rs) &mdash; 228 lines
- [`wasm_telemetry_contract.rs`](crates/js/tests/wasm_telemetry_contract.rs) &mdash; 54 lines
- [`wasm_transport_policy_contract.rs`](crates/js/tests/wasm_transport_policy_contract.rs) &mdash; 320 lines
- [`wasm_workflow_coverage_contract.rs`](crates/js/tests/wasm_workflow_coverage_contract.rs) &mdash; 496 lines

</details>

<details>
<summary><code>crates/js/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/js/tests/common/mod.rs) &mdash; 195 lines

</details>

<details>
<summary><code>crates/orderbook/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/orderbook/Cargo.toml) &mdash; 74 lines
- [`README.md`](crates/orderbook/README.md) &mdash; 117 lines

</details>

<details>
<summary><code>crates/orderbook/src/</code> &mdash; 7 file(s)</summary>

- [`api.rs`](crates/orderbook/src/api.rs) &mdash; 892 lines
- [`builder.rs`](crates/orderbook/src/builder.rs) &mdash; 476 lines
- [`error.rs`](crates/orderbook/src/error.rs) &mdash; 589 lines
- [`lib.rs`](crates/orderbook/src/lib.rs) &mdash; 331 lines
- [`rejection.rs`](crates/orderbook/src/rejection.rs) &mdash; 654 lines
- [`request.rs`](crates/orderbook/src/request.rs) &mdash; 710 lines
- [`transform.rs`](crates/orderbook/src/transform.rs) &mdash; 32 lines

</details>

<details>
<summary><code>crates/orderbook/src/types/</code> &mdash; 8 file(s)</summary>

- [`app_data.rs`](crates/orderbook/src/types/app_data.rs) &mdash; 159 lines
- [`auction.rs`](crates/orderbook/src/types/auction.rs) &mdash; 433 lines
- [`enums.rs`](crates/orderbook/src/types/enums.rs) &mdash; 200 lines
- [`lists.rs`](crates/orderbook/src/types/lists.rs) &mdash; 206 lines
- [`mod.rs`](crates/orderbook/src/types/mod.rs) &mdash; 117 lines
- [`order.rs`](crates/orderbook/src/types/order.rs) &mdash; 1,019 lines
- [`prices.rs`](crates/orderbook/src/types/prices.rs) &mdash; 84 lines
- [`quote.rs`](crates/orderbook/src/types/quote.rs) &mdash; 1,151 lines

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
- [`invariant_contract.rs`](crates/orderbook/tests/invariant_contract.rs) &mdash; 303 lines
- [`order_creation_fee_deserialize.rs`](crates/orderbook/tests/order_creation_fee_deserialize.rs) &mdash; 153 lines
- [`quote_echo_contract.rs`](crates/orderbook/tests/quote_echo_contract.rs) &mdash; 469 lines
- [`rejection_category_contract.rs`](crates/orderbook/tests/rejection_category_contract.rs) &mdash; 81 lines
- [`rejection_contract.rs`](crates/orderbook/tests/rejection_contract.rs) &mdash; 635 lines
- [`request_contract.rs`](crates/orderbook/tests/request_contract.rs) &mdash; 963 lines
- [`transform_contract.rs`](crates/orderbook/tests/transform_contract.rs) &mdash; 399 lines
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

- [`Cargo.toml`](crates/sdk/Cargo.toml) &mdash; 81 lines
- [`README.md`](crates/sdk/README.md) &mdash; 172 lines

</details>

<details>
<summary><code>crates/sdk/src/</code> &mdash; 1 file(s)</summary>

- [`lib.rs`](crates/sdk/src/lib.rs) &mdash; 307 lines

</details>

<details>
<summary><code>crates/sdk/tests/</code> &mdash; 5 file(s)</summary>

- [`error_class_contract.rs`](crates/sdk/tests/error_class_contract.rs) &mdash; 285 lines
- [`error_redaction_contract.rs`](crates/sdk/tests/error_redaction_contract.rs) &mdash; 896 lines
- [`public_api_default_features_only.rs`](crates/sdk/tests/public_api_default_features_only.rs) &mdash; 25 lines
- [`public_api_with_all_features.rs`](crates/sdk/tests/public_api_with_all_features.rs) &mdash; 26 lines
- [`public_api.rs`](crates/sdk/tests/public_api.rs) &mdash; 121 lines

</details>

<details>
<summary><code>crates/signing/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/signing/Cargo.toml) &mdash; 53 lines
- [`README.md`](crates/signing/README.md) &mdash; 143 lines

</details>

<details>
<summary><code>crates/signing/benches/</code> &mdash; 1 file(s)</summary>

- [`typed_data.rs`](crates/signing/benches/typed_data.rs) &mdash; 30 lines

</details>

<details>
<summary><code>crates/signing/src/</code> &mdash; 6 file(s)</summary>

- [`cache.rs`](crates/signing/src/cache.rs) &mdash; 11 lines
- [`cancellation.rs`](crates/signing/src/cancellation.rs) &mdash; 195 lines
- [`domain.rs`](crates/signing/src/domain.rs) &mdash; 195 lines
- [`errors.rs`](crates/signing/src/errors.rs) &mdash; 75 lines
- [`lib.rs`](crates/signing/src/lib.rs) &mdash; 50 lines
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
<summary><code>crates/signing/tests/</code> &mdash; 6 file(s)</summary>

- [`cancellation_contract.rs`](crates/signing/tests/cancellation_contract.rs) &mdash; 192 lines
- [`domain_contract.rs`](crates/signing/tests/domain_contract.rs) &mdash; 103 lines
- [`eip1271_contract.rs`](crates/signing/tests/eip1271_contract.rs) &mdash; 145 lines
- [`order_signing_contract.rs`](crates/signing/tests/order_signing_contract.rs) &mdash; 301 lines
- [`property_contract.rs`](crates/signing/tests/property_contract.rs) &mdash; 467 lines
- [`ui.rs`](crates/signing/tests/ui.rs) &mdash; 5 lines

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
- [`README.md`](crates/subgraph/README.md) &mdash; 96 lines

</details>

<details>
<summary><code>crates/subgraph/src/</code> &mdash; 6 file(s)</summary>

- [`api.rs`](crates/subgraph/src/api.rs) &mdash; 734 lines
- [`builder.rs`](crates/subgraph/src/builder.rs) &mdash; 402 lines
- [`error.rs`](crates/subgraph/src/error.rs) &mdash; 360 lines
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

- [`Cargo.toml`](crates/trading/Cargo.toml) &mdash; 100 lines
- [`README.md`](crates/trading/README.md) &mdash; 282 lines

</details>

<details>
<summary><code>crates/trading/benches/</code> &mdash; 1 file(s)</summary>

- [`order_build.rs`](crates/trading/benches/order_build.rs) &mdash; 51 lines

</details>

<details>
<summary><code>crates/trading/src/</code> &mdash; 13 file(s)</summary>

- [`allowance.rs`](crates/trading/src/allowance.rs) &mdash; 124 lines
- [`app_data.rs`](crates/trading/src/app_data.rs) &mdash; 323 lines
- [`cancel.rs`](crates/trading/src/cancel.rs) &mdash; 65 lines
- [`error.rs`](crates/trading/src/error.rs) &mdash; 255 lines
- [`lib.rs`](crates/trading/src/lib.rs) &mdash; 115 lines
- [`onchain.rs`](crates/trading/src/onchain.rs) &mdash; 391 lines
- [`order.rs`](crates/trading/src/order.rs) &mdash; 336 lines
- [`params.rs`](crates/trading/src/params.rs) &mdash; 109 lines
- [`post.rs`](crates/trading/src/post.rs) &mdash; 780 lines
- [`quote.rs`](crates/trading/src/quote.rs) &mdash; 429 lines
- [`slippage.rs`](crates/trading/src/slippage.rs) &mdash; 828 lines
- [`validation.rs`](crates/trading/src/validation.rs) &mdash; 274 lines
- [`wait.rs`](crates/trading/src/wait.rs) &mdash; 393 lines

</details>

<details>
<summary><code>crates/trading/src/client/</code> &mdash; 6 file(s)</summary>

- [`builder.rs`](crates/trading/src/client/builder.rs) &mdash; 265 lines
- [`helpers.rs`](crates/trading/src/client/helpers.rs) &mdash; 169 lines
- [`limit.rs`](crates/trading/src/client/limit.rs) &mdash; 288 lines
- [`methods.rs`](crates/trading/src/client/methods.rs) &mdash; 597 lines
- [`mod.rs`](crates/trading/src/client/mod.rs) &mdash; 151 lines
- [`swap.rs`](crates/trading/src/client/swap.rs) &mdash; 330 lines

</details>

<details>
<summary><code>crates/trading/src/types/</code> &mdash; 4 file(s)</summary>

- [`mod.rs`](crates/trading/src/types/mod.rs) &mdash; 16 lines
- [`params.rs`](crates/trading/src/types/params.rs) &mdash; 1,232 lines
- [`result.rs`](crates/trading/src/types/result.rs) &mdash; 313 lines
- [`seams.rs`](crates/trading/src/types/seams.rs) &mdash; 102 lines

</details>

<details>
<summary><code>crates/trading/tests/</code> &mdash; 24 file(s)</summary>

- [`allowance_contract.rs`](crates/trading/tests/allowance_contract.rs) &mdash; 143 lines
- [`app_code_contract.rs`](crates/trading/tests/app_code_contract.rs) &mdash; 43 lines
- [`app_data_merge_contract.rs`](crates/trading/tests/app_data_merge_contract.rs) &mdash; 661 lines
- [`cancel_contract.rs`](crates/trading/tests/cancel_contract.rs) &mdash; 87 lines
- [`cancellation_composition_contract.rs`](crates/trading/tests/cancellation_composition_contract.rs) &mdash; 519 lines
- [`error_variant_shape.rs`](crates/trading/tests/error_variant_shape.rs) &mdash; 113 lines
- [`invariant_contract.rs`](crates/trading/tests/invariant_contract.rs) &mdash; 516 lines
- [`limit_from_quote_contract.rs`](crates/trading/tests/limit_from_quote_contract.rs) &mdash; 103 lines
- [`limit_lifecycle_contract.rs`](crates/trading/tests/limit_lifecycle_contract.rs) &mdash; 118 lines
- [`onchain_contract.rs`](crates/trading/tests/onchain_contract.rs) &mdash; 458 lines
- [`order_contract.rs`](crates/trading/tests/order_contract.rs) &mdash; 185 lines
- [`parameters_contract.rs`](crates/trading/tests/parameters_contract.rs) &mdash; 133 lines
- [`post_contract.rs`](crates/trading/tests/post_contract.rs) &mdash; 930 lines
- [`property_contract.rs`](crates/trading/tests/property_contract.rs) &mdash; 212 lines
- [`quote_contract.rs`](crates/trading/tests/quote_contract.rs) &mdash; 866 lines
- [`quote_projection_parity.rs`](crates/trading/tests/quote_projection_parity.rs) &mdash; 154 lines
- [`sdk_contract.rs`](crates/trading/tests/sdk_contract.rs) &mdash; 672 lines
- [`slippage_contract.rs`](crates/trading/tests/slippage_contract.rs) &mdash; 244 lines
- [`swap_lifecycle_contract.rs`](crates/trading/tests/swap_lifecycle_contract.rs) &mdash; 147 lines
- [`types_contract.rs`](crates/trading/tests/types_contract.rs) &mdash; 300 lines
- [`ui.rs`](crates/trading/tests/ui.rs) &mdash; 11 lines
- [`validation_contract.rs`](crates/trading/tests/validation_contract.rs) &mdash; 342 lines
- [`wait_helper_contract.rs`](crates/trading/tests/wait_helper_contract.rs) &mdash; 190 lines
- [`wait_telemetry_contract.rs`](crates/trading/tests/wait_telemetry_contract.rs) &mdash; 85 lines

</details>

<details>
<summary><code>crates/trading/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/trading/tests/common/mod.rs) &mdash; 1,009 lines

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
<summary><code>docs/</code> &mdash; 1 file(s)</summary>

- [`code-of-conduct.md`](docs/code-of-conduct.md) &mdash; 71 lines

</details>

<details>
<summary><code>docs/adr/</code> &mdash; 56 file(s)</summary>

- [`0000-template.md`](docs/adr/0000-template.md) &mdash; 41 lines
- [`0001-multi-crate-sdk-family-with-thin-facade.md`](docs/adr/0001-multi-crate-sdk-family-with-thin-facade.md) &mdash; 76 lines
- [`0002-dedicated-trading-orchestration-crate.md`](docs/adr/0002-dedicated-trading-orchestration-crate.md) &mdash; 51 lines
- [`0003-separate-read-only-subgraph-crate.md`](docs/adr/0003-separate-read-only-subgraph-crate.md) &mdash; 72 lines
- [`0005-boundary-specific-runtime-contracts-and-strong-domain-types.md`](docs/adr/0005-boundary-specific-runtime-contracts-and-strong-domain-types.md) &mdash; 75 lines
- [`0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md`](docs/adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md) &mdash; 58 lines
- [`0010-runtime-neutral-async-and-transport-posture.md`](docs/adr/0010-runtime-neutral-async-and-transport-posture.md) &mdash; 97 lines
- [`0011-typed-amount-boundary-and-typestate-ready-state-construction.md`](docs/adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md) &mdash; 141 lines
- [`0012-alloy-sol-bindings-and-registry-authority.md`](docs/adr/0012-alloy-sol-bindings-and-registry-authority.md) &mdash; 114 lines
- [`0013-http-transport-injection-and-typestate-builders.md`](docs/adr/0013-http-transport-injection-and-typestate-builders.md) &mdash; 103 lines
- [`0014-eip1271-verification-cache.md`](docs/adr/0014-eip1271-verification-cache.md) &mdash; 136 lines
- [`0015-client-side-order-bounds-validator.md`](docs/adr/0015-client-side-order-bounds-validator.md) &mdash; 128 lines
- [`0016-split-sell-and-buy-token-balance-enums.md`](docs/adr/0016-split-sell-and-buy-token-balance-enums.md) &mdash; 98 lines
- [`0017-typed-orderbook-rejection-parser.md`](docs/adr/0017-typed-orderbook-rejection-parser.md) &mdash; 143 lines
- [`0018-typed-app-data-merge.md`](docs/adr/0018-typed-app-data-merge.md) &mdash; 139 lines
- [`0020-ethflow-owner-threading.md`](docs/adr/0020-ethflow-owner-threading.md) &mdash; 166 lines
- [`0021-orderbook-total-fee-policy.md`](docs/adr/0021-orderbook-total-fee-policy.md) &mdash; 116 lines
- [`0022-ecdsa-signature-v-normalization.md`](docs/adr/0022-ecdsa-signature-v-normalization.md) &mdash; 142 lines
- [`0024-asyncprovider-asyncsigningprovider-capability-split.md`](docs/adr/0024-asyncprovider-asyncsigningprovider-capability-split.md) &mdash; 77 lines
- [`0025-workspace-url-redaction-convention.md`](docs/adr/0025-workspace-url-redaction-convention.md) &mdash; 66 lines
- [`0026-alloy-major-release-absorption-plan.md`](docs/adr/0026-alloy-major-release-absorption-plan.md) &mdash; 103 lines
- [`0027-post-quantum-signing-absorption-plan.md`](docs/adr/0027-post-quantum-signing-absorption-plan.md) &mdash; 85 lines
- [`0028-account-abstraction-integration-plan.md`](docs/adr/0028-account-abstraction-integration-plan.md) &mdash; 89 lines
- [`0030-workspace-locked-versioning-tag-baseline.md`](docs/adr/0030-workspace-locked-versioning-tag-baseline.md) &mdash; 84 lines
- [`0031-wire-dto-openapi-driven-with-order-auction-order-split.md`](docs/adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md) &mdash; 83 lines
- [`0032-deployment-authority-machine-readable-provenance.md`](docs/adr/0032-deployment-authority-machine-readable-provenance.md) &mdash; 109 lines
- [`0033-minimum-viable-panic-surface.md`](docs/adr/0033-minimum-viable-panic-surface.md) &mdash; 76 lines
- [`0035-alloy-provider-adapter.md`](docs/adr/0035-alloy-provider-adapter.md) &mdash; 158 lines
- [`0038-transaction-lifecycle-types.md`](docs/adr/0038-transaction-lifecycle-types.md) &mdash; 79 lines
- [`0039-typescript-callable-wasm-sdk-surface.md`](docs/adr/0039-typescript-callable-wasm-sdk-surface.md) &mdash; 166 lines
- [`0040-wallet-provider-callback-boundary-for-js-consumers.md`](docs/adr/0040-wallet-provider-callback-boundary-for-js-consumers.md) &mdash; 95 lines
- [`0041-transport-policy-l3-layering.md`](docs/adr/0041-transport-policy-l3-layering.md) &mdash; 101 lines
- [`0044-bundle-size-profile-and-flavor-builds.md`](docs/adr/0044-bundle-size-profile-and-flavor-builds.md) &mdash; 115 lines
- [`0045-async-signer-trait-narrowing.md`](docs/adr/0045-async-signer-trait-narrowing.md) &mdash; 61 lines
- [`0048-composable-conditional-order-framework.md`](docs/adr/0048-composable-conditional-order-framework.md) &mdash; 194 lines
- [`0049-cow-shed-account-abstraction-proxy.md`](docs/adr/0049-cow-shed-account-abstraction-proxy.md) &mdash; 151 lines
- [`0050-eip1271-signature-blob-encoding.md`](docs/adr/0050-eip1271-signature-blob-encoding.md) &mdash; 171 lines
- [`0051-signing-owned-eip1271-signature-provider-trait.md`](docs/adr/0051-signing-owned-eip1271-signature-provider-trait.md) &mdash; 141 lines
- [`0052-alloy-primitives-canonical-primitive-layer.md`](docs/adr/0052-alloy-primitives-canonical-primitive-layer.md) &mdash; 133 lines
- [`0053-typed-signer-rejection-classification.md`](docs/adr/0053-typed-signer-rejection-classification.md) &mdash; 161 lines
- [`0054-onchain-order-event-decoding-is-fail-closed.md`](docs/adr/0054-onchain-order-event-decoding-is-fail-closed.md) &mdash; 92 lines
- [`0055-bounded-response-reads.md`](docs/adr/0055-bounded-response-reads.md) &mdash; 101 lines
- [`0057-log-provider-capability-trait.md`](docs/adr/0057-log-provider-capability-trait.md) &mdash; 130 lines
- [`0058-typed-quote-request-response-surface.md`](docs/adr/0058-typed-quote-request-response-surface.md) &mdash; 186 lines
- [`0059-hash-concrete-orderdata-directly.md`](docs/adr/0059-hash-concrete-orderdata-directly.md) &mdash; 82 lines
- [`0060-uniform-error-classification.md`](docs/adr/0060-uniform-error-classification.md) &mdash; 127 lines
- [`0061-wasm-abi-receiver-pay-to-owner.md`](docs/adr/0061-wasm-abi-receiver-pay-to-owner.md) &mdash; 88 lines
- [`0062-internal-shared-test-support-crate.md`](docs/adr/0062-internal-shared-test-support-crate.md) &mdash; 67 lines
- [`0063-published-consumer-test-doubles-crate.md`](docs/adr/0063-published-consumer-test-doubles-crate.md) &mdash; 103 lines
- [`0064-app-data-typed-validation.md`](docs/adr/0064-app-data-typed-validation.md) &mdash; 95 lines
- [`0066-trading-slippage-and-suggestion-policy.md`](docs/adr/0066-trading-slippage-and-suggestion-policy.md) &mdash; 66 lines
- [`0067-idiomatic-accessor-naming.md`](docs/adr/0067-idiomatic-accessor-naming.md) &mdash; 75 lines
- [`0068-payload-only-typed-data-signing.md`](docs/adr/0068-payload-only-typed-data-signing.md) &mdash; 87 lines
- [`0069-layered-trading-operation-surface-and-signing-free-transport.md`](docs/adr/0069-layered-trading-operation-surface-and-signing-free-transport.md) &mdash; 92 lines
- [`0070-onchain-transaction-helper-boundary.md`](docs/adr/0070-onchain-transaction-helper-boundary.md) &mdash; 126 lines
- [`0071-wasm-component-distribution-channel.md`](docs/adr/0071-wasm-component-distribution-channel.md) &mdash; 104 lines

</details>

<details>
<summary><code>docs/audit/</code> &mdash; 18 file(s)</summary>

- [`alloy-adapters-audit.md`](docs/audit/alloy-adapters-audit.md) &mdash; 48 lines
- [`bounded-response-reads-audit.md`](docs/audit/bounded-response-reads-audit.md) &mdash; 45 lines
- [`contract-bindings-parity-audit.md`](docs/audit/contract-bindings-parity-audit.md) &mdash; 46 lines
- [`cow-shed-contract-bindings-audit.md`](docs/audit/cow-shed-contract-bindings-audit.md) &mdash; 49 lines
- [`credential-redaction-audit.md`](docs/audit/credential-redaction-audit.md) &mdash; 54 lines
- [`dependency-gate-audit.md`](docs/audit/dependency-gate-audit.md) &mdash; 65 lines
- [`deployment-registry-audit.md`](docs/audit/deployment-registry-audit.md) &mdash; 64 lines
- [`ecdsa-signature-normalization-audit.md`](docs/audit/ecdsa-signature-normalization-audit.md) &mdash; 42 lines
- [`eip1271-verification-cache-audit.md`](docs/audit/eip1271-verification-cache-audit.md) &mdash; 43 lines
- [`error-classification-audit.md`](docs/audit/error-classification-audit.md) &mdash; 45 lines
- [`event-log-decoding-audit.md`](docs/audit/event-log-decoding-audit.md) &mdash; 42 lines
- [`fuzz-coverage-audit.md`](docs/audit/fuzz-coverage-audit.md) &mdash; 51 lines
- [`http-transport-contract-audit.md`](docs/audit/http-transport-contract-audit.md) &mdash; 49 lines
- [`panic-free-public-surface-audit.md`](docs/audit/panic-free-public-surface-audit.md) &mdash; 43 lines
- [`source-lock-provenance-audit.md`](docs/audit/source-lock-provenance-audit.md) &mdash; 44 lines
- [`trading-order-integrity-audit.md`](docs/audit/trading-order-integrity-audit.md) &mdash; 47 lines
- [`wasm-surface-audit.md`](docs/audit/wasm-surface-audit.md) &mdash; 49 lines
- [`workflow-security-audit.md`](docs/audit/workflow-security-audit.md) &mdash; 44 lines

</details>

<details>
<summary><code>docs/providers/</code> &mdash; 1 file(s)</summary>

- [`adapting-alloy.md`](docs/providers/adapting-alloy.md) &mdash; 209 lines

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
- [`signing.spec.ts`](e2e/wasm-typescript/tests/signing.spec.ts) &mdash; 59 lines
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

- [`Cargo.toml`](examples/native/Cargo.toml) &mdash; 154 lines
- [`README.md`](examples/native/README.md) &mdash; 160 lines

</details>

<details>
<summary><code>examples/native/scenarios/</code> &mdash; 30 file(s)</summary>

- [`alloy_custom_traits.rs`](examples/native/scenarios/alloy_custom_traits.rs) &mdash; 164 lines
- [`alloy_provider.rs`](examples/native/scenarios/alloy_provider.rs) &mdash; 42 lines
- [`alloy_quickstart.rs`](examples/native/scenarios/alloy_quickstart.rs) &mdash; 47 lines
- [`alloy_signer.rs`](examples/native/scenarios/alloy_signer.rs) &mdash; 71 lines
- [`alloy_trading_full_flow.rs`](examples/native/scenarios/alloy_trading_full_flow.rs) &mdash; 104 lines
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
- [`receiver_redirect.rs`](examples/native/scenarios/receiver_redirect.rs) &mdash; 69 lines
- [`sign_order.rs`](examples/native/scenarios/sign_order.rs) &mdash; 64 lines
- [`slippage_suggester.rs`](examples/native/scenarios/slippage_suggester.rs) &mdash; 71 lines
- [`subgraph_live.rs`](examples/native/scenarios/subgraph_live.rs) &mdash; 48 lines
- [`subgraph_query.rs`](examples/native/scenarios/subgraph_query.rs) &mdash; 177 lines
- [`swap_quickstart.rs`](examples/native/scenarios/swap_quickstart.rs) &mdash; 68 lines
- [`token_balance.rs`](examples/native/scenarios/token_balance.rs) &mdash; 75 lines
- [`trading_full_cycle.rs`](examples/native/scenarios/trading_full_cycle.rs) &mdash; 107 lines
- [`transaction_lifecycle.rs`](examples/native/scenarios/transaction_lifecycle.rs) &mdash; 76 lines
- [`twap_order.rs`](examples/native/scenarios/twap_order.rs) &mdash; 78 lines

</details>

<details>
<summary><code>examples/native/src/</code> &mdash; 2 file(s)</summary>

- [`lib.rs`](examples/native/src/lib.rs) &mdash; 18 lines
- [`support.rs`](examples/native/src/support.rs) &mdash; 363 lines

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
- [`fuzz_app_data_size_limit.rs`](fuzz/fuzz_targets/fuzz_app_data_size_limit.rs) &mdash; 156 lines
- [`fuzz_cid_to_app_data_hex.rs`](fuzz/fuzz_targets/fuzz_cid_to_app_data_hex.rs) &mdash; 90 lines
- [`fuzz_composable_merkle_roundtrip.rs`](fuzz/fuzz_targets/fuzz_composable_merkle_roundtrip.rs) &mdash; 74 lines
- [`fuzz_core_identity_validators.rs`](fuzz/fuzz_targets/fuzz_core_identity_validators.rs) &mdash; 195 lines
- [`fuzz_decode_magic_value_response.rs`](fuzz/fuzz_targets/fuzz_decode_magic_value_response.rs) &mdash; 214 lines
- [`fuzz_decoded_body_canonical_status_text.rs`](fuzz/fuzz_targets/fuzz_decoded_body_canonical_status_text.rs) &mdash; 243 lines
- [`fuzz_ecdsa_v_normalization.rs`](fuzz/fuzz_targets/fuzz_ecdsa_v_normalization.rs) &mdash; 54 lines
- [`fuzz_eip1271_signature_data_codec.rs`](fuzz/fuzz_targets/fuzz_eip1271_signature_data_codec.rs) &mdash; 56 lines
- [`fuzz_eth_flow_event_log_decode.rs`](fuzz/fuzz_targets/fuzz_eth_flow_event_log_decode.rs) &mdash; 52 lines
- [`fuzz_ethflow_create_order_encode.rs`](fuzz/fuzz_targets/fuzz_ethflow_create_order_encode.rs) &mdash; 115 lines
- [`fuzz_flashloan_hints.rs`](fuzz/fuzz_targets/fuzz_flashloan_hints.rs) &mdash; 111 lines
- [`fuzz_hash_order_cancellations.rs`](fuzz/fuzz_targets/fuzz_hash_order_cancellations.rs) &mdash; 153 lines
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
- [`fuzz_recover_ecdsa_address.rs`](fuzz/fuzz_targets/fuzz_recover_ecdsa_address.rs) &mdash; 92 lines
- [`fuzz_recoverable_signature_differential.rs`](fuzz/fuzz_targets/fuzz_recoverable_signature_differential.rs) &mdash; 89 lines
- [`fuzz_recoverable_signature_parse_hex.rs`](fuzz/fuzz_targets/fuzz_recoverable_signature_parse_hex.rs) &mdash; 61 lines
- [`fuzz_redact_response_body.rs`](fuzz/fuzz_targets/fuzz_redact_response_body.rs) &mdash; 84 lines
- [`fuzz_retry_policy_delay.rs`](fuzz/fuzz_targets/fuzz_retry_policy_delay.rs) &mdash; 153 lines
- [`fuzz_schema_version_is_semver.rs`](fuzz/fuzz_targets/fuzz_schema_version_is_semver.rs) &mdash; 92 lines
- [`fuzz_settlement_event_log_decode.rs`](fuzz/fuzz_targets/fuzz_settlement_event_log_decode.rs) &mdash; 54 lines
- [`fuzz_signing_domain_separator.rs`](fuzz/fuzz_targets/fuzz_signing_domain_separator.rs) &mdash; 123 lines
- [`fuzz_slippage_amounts.rs`](fuzz/fuzz_targets/fuzz_slippage_amounts.rs) &mdash; 160 lines
- [`fuzz_slippage_policy_helpers.rs`](fuzz/fuzz_targets/fuzz_slippage_policy_helpers.rs) &mdash; 193 lines
- [`fuzz_stringify_deterministic.rs`](fuzz/fuzz_targets/fuzz_stringify_deterministic.rs) &mdash; 73 lines
- [`fuzz_subgraph_graphql_error_decode.rs`](fuzz/fuzz_targets/fuzz_subgraph_graphql_error_decode.rs) &mdash; 93 lines
- [`fuzz_transport_error_classify.rs`](fuzz/fuzz_targets/fuzz_transport_error_classify.rs) &mdash; 282 lines
- [`fuzz_typed_data_digest.rs`](fuzz/fuzz_targets/fuzz_typed_data_digest.rs) &mdash; 138 lines
- [`fuzz_valid_to_relative.rs`](fuzz/fuzz_targets/fuzz_valid_to_relative.rs) &mdash; 84 lines

</details>

<details>
<summary><code>parity/</code> &mdash; 2 file(s)</summary>

- [`README.md`](parity/README.md) &mdash; 265 lines
- [`source-lock.yaml`](parity/source-lock.yaml) &mdash; 90 lines

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
<summary><code>parity/fixtures/signing/</code> &mdash; 1 file(s)</summary>

- [`eip1271_typescript_vector.json`](parity/fixtures/signing/eip1271_typescript_vector.json) &mdash; 30 lines

</details>

<details>
<summary><code>parity/fixtures/trading/</code> &mdash; 1 file(s)</summary>

- [`protocol_fee_partner_fee_composition.json`](parity/fixtures/trading/protocol_fee_partner_fee_composition.json) &mdash; 39 lines

</details>

<details>
<summary><code>parity/openapi/</code> &mdash; 2 file(s)</summary>

- [`coverage.yaml`](parity/openapi/coverage.yaml) &mdash; 181 lines
- [`services-orderbook.yml`](parity/openapi/services-orderbook.yml) &mdash; 2,805 lines

</details>

<details>
<summary><code>tests/</code> &mdash; 11 file(s)</summary>

- [`alloy_read_contract_parity_invariant.rs`](tests/alloy_read_contract_parity_invariant.rs) &mdash; 104 lines
- [`alloy_two_family_lockfile_invariant.rs`](tests/alloy_two_family_lockfile_invariant.rs) &mdash; 112 lines
- [`alloy_umbrella_composition.rs`](tests/alloy_umbrella_composition.rs) &mdash; 100 lines
- [`Cargo.toml`](tests/Cargo.toml) &mdash; 69 lines
- [`component_wit_record_drift.rs`](tests/component_wit_record_drift.rs) &mdash; 107 lines
- [`cow_shed_typed_data_digest.rs`](tests/cow_shed_typed_data_digest.rs) &mdash; 77 lines
- [`dependency_default_features_audit.rs`](tests/dependency_default_features_audit.rs) &mdash; 82 lines
- [`msrv_consistency.rs`](tests/msrv_consistency.rs) &mdash; 37 lines
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
- [`main.rs`](xtask/src/main.rs) &mdash; 284 lines
- [`version_surface.rs`](xtask/src/version_surface.rs) &mdash; 280 lines

</details>

<details>
<summary><code>xtask/src/docs/</code> &mdash; 3 file(s)</summary>

- [`agree.rs`](xtask/src/docs/agree.rs) &mdash; 266 lines
- [`audit_index.rs`](xtask/src/docs/audit_index.rs) &mdash; 163 lines
- [`mod.rs`](xtask/src/docs/mod.rs) &mdash; 9 lines

</details>

<details>
<summary><code>xtask/src/parity/</code> &mdash; 5 file(s)</summary>

- [`mod.rs`](xtask/src/parity/mod.rs) &mdash; 1,466 lines
- [`openapi_coverage.rs`](xtask/src/parity/openapi_coverage.rs) &mdash; 854 lines
- [`registry_confirm.rs`](xtask/src/parity/registry_confirm.rs) &mdash; 364 lines
- [`sync.rs`](xtask/src/parity/sync.rs) &mdash; 565 lines
- [`vendor_openapi.rs`](xtask/src/parity/vendor_openapi.rs) &mdash; 67 lines

</details>

<details>
<summary><code>xtask/src/policy/</code> &mdash; 20 file(s)</summary>

- [`check_adr_coverage.rs`](xtask/src/policy/check_adr_coverage.rs) &mdash; 225 lines
- [`check_alloy_family_pins.rs`](xtask/src/policy/check_alloy_family_pins.rs) &mdash; 237 lines
- [`check_chain_patch_eligibility.rs`](xtask/src/policy/check_chain_patch_eligibility.rs) &mdash; 202 lines
- [`check_deny_unknown_fields.rs`](xtask/src/policy/check_deny_unknown_fields.rs) &mdash; 126 lines
- [`check_enum_policy.rs`](xtask/src/policy/check_enum_policy.rs) &mdash; 142 lines
- [`check_msrv_notice.rs`](xtask/src/policy/check_msrv_notice.rs) &mdash; 163 lines
- [`check_panic_allowlist.rs`](xtask/src/policy/check_panic_allowlist.rs) &mdash; 356 lines
- [`check_property_citations.rs`](xtask/src/policy/check_property_citations.rs) &mdash; 222 lines
- [`check_readme_include.rs`](xtask/src/policy/check_readme_include.rs) &mdash; 100 lines
- [`check_shell_wrappers.rs`](xtask/src/policy/check_shell_wrappers.rs) &mdash; 90 lines
- [`check_wasm_invariant.rs`](xtask/src/policy/check_wasm_invariant.rs) &mdash; 271 lines
- [`check_workflow_security.rs`](xtask/src/policy/check_workflow_security.rs) &mdash; 131 lines
- [`check_workspace_versions.rs`](xtask/src/policy/check_workspace_versions.rs) &mdash; 185 lines
- [`classify_release.rs`](xtask/src/policy/classify_release.rs) &mdash; 188 lines
- [`dependency_invariant.rs`](xtask/src/policy/dependency_invariant.rs) &mdash; 177 lines
- [`fences.rs`](xtask/src/policy/fences.rs) &mdash; 472 lines
- [`fixtures.rs`](xtask/src/policy/fixtures.rs) &mdash; 13 lines
- [`mod.rs`](xtask/src/policy/mod.rs) &mdash; 107 lines
- [`run_deterministic_examples.rs`](xtask/src/policy/run_deterministic_examples.rs) &mdash; 177 lines
- [`workspace.rs`](xtask/src/policy/workspace.rs) &mdash; 548 lines

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
- [`openapi_coverage.rs`](xtask/tests/openapi_coverage.rs) &mdash; 166 lines
- [`registry_confirm.rs`](xtask/tests/registry_confirm.rs) &mdash; 157 lines
- [`vendor_openapi.rs`](xtask/tests/vendor_openapi.rs) &mdash; 104 lines

</details>

<details>
<summary><code>xtask/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](xtask/tests/common/mod.rs) &mdash; 158 lines

</details>


