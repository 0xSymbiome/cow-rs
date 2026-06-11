# Repository File Map

> **Branch:** `feat/ferrous-foundation` &nbsp;&middot;&nbsp; **HEAD:** `9e245eb` &nbsp;&middot;&nbsp; **Generated:** 2026-06-11  
> **Total tracked files:** **1,016** &nbsp;&middot;&nbsp; **Lines of code:** tokei 14.0.0

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

- **36,452 lines of Rust** across the 16 SDK crates, covered by **38,784 lines of tests** — a **1.1× test-to-code ratio** — plus **180 lines of benchmarks**.
- **12,224 doc-comment lines** documenting the public API (~33.5% of crate code), plus **846 inline comment lines**.
- **6,381 lines of TypeScript** across examples, e2e harnesses, and wasm bindings.
- **23,260 lines of Markdown prose** — ADRs, audit notes, and READMEs.
- **17,323 lines of data & config** — JSON schemas, parity fixtures, YAML, TOML, and lockfiles. Tracked and counted in the totals below; listed separately here because it's data, not hand-written code.

**Footprint** (tracked files)

- **587 files** live under `crates/` — 16 workspace member crates make up roughly 58% of the repo.
- **154 files** under `docs/` are mostly architecture decision records and audit notes.
- **37 files** under `parity/` are golden fixtures captured from upstream services to keep the Rust SDK byte-compatible.
- **48 files** under `fuzz/` cover cargo-fuzz targets and their seed corpora.
- **88 files** under `examples/` + `e2e/` are runnable demos and integration harnesses.
- **44 files** under `xtask/` are the maintenance automation crate (parity refresh, policy checks, doc generation).

---

## Top-level layout

| Path | Files | Lines | Code | Purpose |
|------|------:|------:|-----:|---------|
| `crates/` | 587 | 121,987 | 83,314 | Workspace member crates (the SDK itself) |
| `docs/` | 154 | 23,858 | 0 | Architecture decision records, audit notes, provider notes |
| `examples/` | 62 | 12,652 | 5,612 | Runnable usage examples (Rust + TypeScript) |
| `fuzz/` | 48 | 9,716 | 3,989 | cargo-fuzz targets, corpora, and failure artifacts |
| `xtask/` | 44 | 8,939 | 7,654 | Cargo xtask automation crate (parity, policy, docs subcommands) |
| `parity/` | 37 | 6,437 | 6,209 | Golden fixtures + pinned specs from upstream services |
| `e2e/` | 26 | 3,928 | 3,170 | End-to-end integration harnesses |
| `.github/` | 24 | 3,594 | 2,947 | GitHub Actions workflows and repo config |
| `tests/` | 17 | 1,569 | 1,340 | Workspace-level integration tests |
| `.cargo/` | 2 | 31 | 26 | Cargo configuration |
| `SECURITY.md` | 1 | 182 | 0 | Security policy |
| `rust-toolchain.toml` | 1 | 6 | 4 | Pinned Rust toolchain |
| `ROADMAP.md` | 1 | 64 | 0 | Roadmap document |
| `README.md` | 1 | 209 | 0 | Top-level README |
| `PROPERTIES.md` | 1 | 258 | 0 | Property-based testing index |
| `.gitattributes` | 1 | 31 | 0 | Git attributes |
| `LICENSE` | 1 | 674 | 0 | License text |
| `Cargo.lock` | 1 | 5,738 | 0 | Workspace lockfile |
| `.githooks/` | 1 | 35 | 28 | Tracked git hook scripts |
| `.gitignore` | 1 | 19 | 0 | Top-level git ignore rules |
| `.yamllint` | 1 | 7 | 0 | YAML lint configuration |
| `CONTRIBUTING.md` | 1 | 288 | 0 | Contribution guide |
| `CHANGELOG.md` | 1 | 744 | 0 | Release changelog |
| `llvm-cov-summary.txt` | 1 | 200 | 0 | Coverage summary snapshot |
| `Cargo.toml` | 1 | 124 | 109 | Workspace manifest |
| **Total** | **1016** | **201,290** | **114,402** | |

---

## File composition by extension

| Extension | Files | Lines | Code | Comments | Blank | Typical role |
|-----------|------:|------:|-----:|---------:|------:|--------------|
| `.rs` | 596 | 119,514 | 90,698 | 17,786 | 11,030 | Rust source and tests |
| `.md` | 184 | 28,378 | 0 | 23,260 | 5,118 | Markdown docs (ADRs, audit notes, READMEs) |
| `.ts` | 60 | 17,394 | 6,381 | 9,874 | 1,139 | TypeScript (examples, e2e, wasm bindings) |
| `.json` | 46 | 3,768 | 3,665 | 103 | 0 | JSON schemas, parity fixtures, test vectors |
| `.toml` | 31 | 1,977 | 1,627 | 90 | 260 | Cargo manifests and tool configs |
| `.stderr` | 26 | 588 | 0 | 567 | 21 | trybuild compile-fail snapshots |
| `.yml` | 17 | 5,351 | 4,770 | 420 | 161 | CI workflows and config |
| `.yaml` | 14 | 7,812 | 6,349 | 30 | 1,433 | CI workflows, OpenAPI specs, config |
| `.txt` | 6 | 233 | 0 | 232 | 1 | Plain text fixtures / summaries |
| `.mjs` | 6 | 594 | 485 | 35 | 74 | JavaScript modules |
| `.gitignore` | 5 | 31 | 0 | 31 | 0 |  |
| `.sh` | 5 | 430 | 363 | 4 | 63 | Shell scripts |
| `(none)` | 3 | 710 | 28 | 555 | 127 |  |
| `.graphql` | 3 | 24 | 24 | 0 | 0 | GraphQL queries (subgraph) |
| `.lock` | 3 | 14,392 | 0 | 13,015 | 1,377 | Cargo / package lockfiles |
| `.bin` | 2 | 0 | 0 | 0 | 0 | Binary fixtures |
| `.keep` | 2 | 2 | 0 | 0 | 2 |  |
| `.snap` | 2 | 29 | 0 | 29 | 0 | Snapshot test outputs |
| `.npmignore` | 1 | 6 | 0 | 6 | 0 |  |
| `.html` | 1 | 12 | 12 | 0 | 0 | Static HTML for browser examples |
| `.gitattributes` | 1 | 31 | 0 | 27 | 4 |  |
| `.yamllint` | 1 | 7 | 0 | 6 | 1 |  |
| `.proptest-regressions` | 1 | 7 | 0 | 7 | 0 | proptest regression seeds |
| **Total** | **1016** | **201,290** | **114,402** | **66,077** | **20,811** | |

> **Code + Comments + Blank = Lines** for every row. ``Comments`` is all non-code, non-blank content: inline + doc-comments in source, prose in Markdown/text, and raw content in formats tokei does not parse as code (lockfiles, ``.stderr``, snapshots). Rust doc-comments are isolated in the per-crate ``Doc`` column above.

---

## Workspace crates (`crates/`)

16 member crates compose the SDK. `Code` is Rust `src/` code; `Tests` and `Benches` are Rust lines under `tests/` and `benches/`; `Doc` is `src/` doc-comment lines (`///` / `//!`) — the public-API documentation surface; `T:C` is the test-to-code ratio. Descriptions are pulled live from each crate's `Cargo.toml`.

| Crate | Files | Code | Tests | Benches | Doc | T:C | Purpose |
|-------|------:|-----:|------:|--------:|----:|----:|---------|
| [`wasm`](crates/wasm) | 113 | 6,091 | 3,606 | 0 | 1,557 | 0.6× | TypeScript-callable wasm-bindgen leaf for the CoW Protocol Rust SDK |
| [`core`](crates/core) | 77 | 5,382 | 4,443 | 0 | 2,285 | 0.8× | Shared CoW Protocol core types and validation primitives |
| [`trading`](crates/trading) | 54 | 5,318 | 6,568 | 46 | 1,756 | 1.2× | High-level CoW Protocol trading orchestration surface |
| [`orderbook`](crates/orderbook) | 41 | 4,288 | 5,151 | 14 | 1,650 | 1.2× | Typed CoW Protocol orderbook client models and decoding helpers |
| [`browser-wallet`](crates/browser-wallet) | 30 | 3,698 | 2,649 | 0 | 589 | 0.7× | Browser wallet integration for the CoW Protocol Rust SDK |
| [`contracts`](crates/contracts) | 62 | 3,129 | 4,390 | 61 | 1,563 | 1.4× | CoW Protocol low-level contracts helpers for order hashing, signature codecs and verification, ABI bindings, and fail-closed on-chain event decoding |
| [`app-data`](crates/app-data) | 41 | 1,428 | 2,182 | 33 | 745 | 1.5× | CoW Protocol app-data encoding, validation, and CID compatibility |
| [`alloy-provider`](crates/alloy-provider) | 27 | 1,354 | 1,538 | 0 | 208 | 1.1× | Alloy-backed read-only Provider adapter for the CoW Protocol Rust SDK |
| [`subgraph`](crates/subgraph) | 26 | 1,286 | 2,201 | 0 | 463 | 1.7× | Typed CoW Protocol subgraph query primitives |
| [`signing`](crates/signing) | 25 | 929 | 1,634 | 26 | 333 | 1.8× | Deterministic CoW Protocol order hashing, EIP-712 signing, and UID helpers |
| [`alloy`](crates/alloy) | 28 | 829 | 1,359 | 0 | 213 | 1.6× | Composed Alloy provider and signer adapter for the CoW Protocol Rust SDK |
| [`test-utils`](crates/test-utils) | 10 | 792 | 145 | 0 | 252 | 0.2× | Internal, unpublished shared test helpers for the cow-rs workspace. |
| [`alloy-signer`](crates/alloy-signer) | 23 | 697 | 540 | 0 | 140 | 0.8× | Alloy-backed local private-key Signer adapter for the CoW Protocol Rust SDK |
| [`test`](crates/test) | 9 | 683 | 182 | 0 | 173 | 0.3× | In-memory test doubles for the cow-rs SDK public traits (OrderbookClient, Signer, Provider) so downstream applications can test their CoW Protocol integration without a live orderbook, RPC endpoint, or wallet. |
| [`transport-wasm`](crates/transport-wasm) | 8 | 423 | 816 | 0 | 126 | 1.9× | Browser fetch-based HTTP transport for the CoW Protocol Rust SDK |
| [`sdk`](crates/sdk) | 13 | 125 | 1,380 | 0 | 171 | 11.0× | Facade crate for CoW Protocol Rust SDK surfaces |
| **Total** | **587** | **36,452** | **38,784** | **180** | **12,224** | **1.1×** | |

---

## Source hotspots

The 25 largest hand-written source files by code lines (Rust + TypeScript). This is where complexity — and review attention — concentrates.

| File | Lang | Kind | Code | Comments |
|------|------|------|-----:|---------:|
| [`crates/orderbook/tests/api_contract.rs`](crates/orderbook/tests/api_contract.rs) | Rust | test | 1,099 | 15 |
| [`crates/subgraph/tests/api_contract.rs`](crates/subgraph/tests/api_contract.rs) | Rust | test | 1,061 | 0 |
| [`xtask/src/parity/mod.rs`](xtask/src/parity/mod.rs) | Rust | src | 977 | 85 |
| [`crates/sdk/tests/error_redaction_contract.rs`](crates/sdk/tests/error_redaction_contract.rs) | Rust | test | 874 | 62 |
| [`crates/orderbook/tests/request_contract.rs`](crates/orderbook/tests/request_contract.rs) | Rust | test | 874 | 16 |
| [`crates/trading/src/types/params.rs`](crates/trading/src/types/params.rs) | Rust | src | 827 | 285 |
| [`crates/trading/tests/common/mod.rs`](crates/trading/tests/common/mod.rs) | Rust | test | 819 | 2 |
| [`crates/trading/tests/quote_contract.rs`](crates/trading/tests/quote_contract.rs) | Rust | test | 729 | 0 |
| [`crates/browser-wallet/src/provider/provider_impl.rs`](crates/browser-wallet/src/provider/provider_impl.rs) | Rust | src | 712 | 24 |
| [`crates/browser-wallet/tests/wasm_bridge_contract.rs`](crates/browser-wallet/tests/wasm_bridge_contract.rs) | Rust | test | 708 | 0 |
| [`xtask/src/parity/openapi_coverage.rs`](xtask/src/parity/openapi_coverage.rs) | Rust | src | 687 | 5 |
| [`crates/browser-wallet/tests/wallet_contract.rs`](crates/browser-wallet/tests/wallet_contract.rs) | Rust | test | 679 | 0 |
| [`crates/trading/tests/post_contract.rs`](crates/trading/tests/post_contract.rs) | Rust | test | 663 | 30 |
| [`crates/wasm/snapshots/raw/cloudflare-web.d.ts`](crates/wasm/snapshots/raw/cloudflare-web.d.ts) | TypeScript | src | 648 | 1,945 |
| [`crates/orderbook/src/types/order.rs`](crates/orderbook/src/types/order.rs) | Rust | src | 629 | 230 |
| [`crates/orderbook/src/types/quote.rs`](crates/orderbook/src/types/quote.rs) | Rust | src | 624 | 245 |
| [`crates/trading/src/post.rs`](crates/trading/src/post.rs) | Rust | src | 622 | 129 |
| [`crates/wasm/snapshots/raw/default-nodejs.d.ts`](crates/wasm/snapshots/raw/default-nodejs.d.ts) | TypeScript | src | 613 | 2,028 |
| [`crates/wasm/snapshots/raw/default-bundler.d.ts`](crates/wasm/snapshots/raw/default-bundler.d.ts) | TypeScript | src | 613 | 2,028 |
| [`crates/orderbook/src/request.rs`](crates/orderbook/src/request.rs) | Rust | src | 610 | 123 |
| [`crates/trading/tests/sdk_contract.rs`](crates/trading/tests/sdk_contract.rs) | Rust | test | 608 | 8 |
| [`crates/trading/src/slippage.rs`](crates/trading/src/slippage.rs) | Rust | src | 598 | 121 |
| [`crates/wasm/src/exports/trading.rs`](crates/wasm/src/exports/trading.rs) | Rust | src | 594 | 105 |
| [`crates/orderbook/src/api.rs`](crates/orderbook/src/api.rs) | Rust | src | 592 | 250 |
| [`crates/wasm/npm/src/default.ts`](crates/wasm/npm/src/default.ts) | TypeScript | src | 590 | 0 |

---

## Examples (`examples/`)

| Example | Files | Lines | Code | Purpose |
|---------|------:|------:|-----:|---------|
| [`native`](examples/native) | 32 | 3,396 | 2,423 | Native Rust scenario walkthroughs |
| [`wasm`](examples/wasm) | 29 | 9,207 | 3,189 | Browser console scenarios (raw wasm) |
| **Total (listed)** | **61** | **12,603** | **5,612** | |

---

## End-to-end harnesses (`e2e/`)

| Harness | Files | Lines | Code | Purpose |
|---------|------:|------:|-----:|---------|
| [`wasm-typescript`](e2e/wasm-typescript) | 14 | 2,036 | 1,645 | Wasm + TypeScript integration harness |
| [`wasm-typescript-cf`](e2e/wasm-typescript-cf) | 12 | 1,892 | 1,525 | Wasm + TypeScript Cloudflare harness |
| **Total (listed)** | **26** | **3,928** | **3,170** | |

---

## Upstream parity (`parity/`)

| Subtree | Files | Lines | Code | Purpose |
|---------|------:|------:|-----:|---------|
| [`fixtures`](parity/fixtures) | 33 | 3,364 | 3,364 | Golden fixtures captured from upstream services |
| [`openapi`](parity/openapi) | 2 | 2,820 | 2,784 | OpenAPI specs pinned for parity |
| **Total (listed)** | **35** | **6,184** | **6,148** | |

---

## Documentation (`docs/`)

| Subtree | Files | Lines | Code | Purpose |
|---------|------:|------:|-----:|---------|
| [`adr`](docs/adr) | 70 | 8,064 | 0 | Architecture Decision Records |
| [`audit`](docs/audit) | 63 | 10,125 | 0 | Audit notes and review artifacts |
| [`providers`](docs/providers) | 2 | 277 | 0 | Provider integration notes |
| **Total (listed)** | **135** | **18,466** | **0** | |

---

## Fuzzing (`fuzz/`)

| Subtree | Files | Lines | Code | Purpose |
|---------|------:|------:|-----:|---------|
| [`fuzz_targets`](fuzz/fuzz_targets) | 45 | 5,383 | 3,694 | cargo-fuzz target sources |
| **Total (listed)** | **45** | **5,383** | **3,694** | |

---

## CI & repo-level configuration

| Path | Files | Purpose |
|------|------:|---------|
| `.github/workflows/` | 13 | GitHub Actions pipelines |
| `.github/config/`    | 8 | Shared CI config |
| `.githooks/`         | 1 | Tracked git hooks |
| `.cargo/`            | 2 | Cargo config (e.g. rustflags) |
| `tests/`             | 17 | Workspace-level integration tests |

---

## Full file index

Every tracked file, grouped by the directory it lives in. Each section is collapsed by default — click to expand. The number after each file is its total line count.

<details>
<summary><code>(repo root)</code> &mdash; 14 file(s)</summary>

- [`.gitattributes`](.gitattributes) &mdash; 31 lines
- [`.gitignore`](.gitignore) &mdash; 19 lines
- [`.yamllint`](.yamllint) &mdash; 7 lines
- [`Cargo.lock`](Cargo.lock) &mdash; 5,738 lines
- [`Cargo.toml`](Cargo.toml) &mdash; 124 lines
- [`CHANGELOG.md`](CHANGELOG.md) &mdash; 744 lines
- [`CONTRIBUTING.md`](CONTRIBUTING.md) &mdash; 288 lines
- [`LICENSE`](LICENSE) &mdash; 674 lines
- [`llvm-cov-summary.txt`](llvm-cov-summary.txt) &mdash; 200 lines
- [`PROPERTIES.md`](PROPERTIES.md) &mdash; 258 lines
- [`README.md`](README.md) &mdash; 209 lines
- [`ROADMAP.md`](ROADMAP.md) &mdash; 64 lines
- [`rust-toolchain.toml`](rust-toolchain.toml) &mdash; 6 lines
- [`SECURITY.md`](SECURITY.md) &mdash; 182 lines

</details>

<details>
<summary><code>.cargo/</code> &mdash; 2 file(s)</summary>

- [`config.toml`](.cargo/config.toml) &mdash; 28 lines
- [`mutants.toml`](.cargo/mutants.toml) &mdash; 3 lines

</details>

<details>
<summary><code>.githooks/</code> &mdash; 1 file(s)</summary>

- [`commit-msg`](.githooks/commit-msg) &mdash; 35 lines

</details>

<details>
<summary><code>.github/</code> &mdash; 2 file(s)</summary>

- [`commit-template.md`](.github/commit-template.md) &mdash; 12 lines
- [`dependabot.yml`](.github/dependabot.yml) &mdash; 102 lines

</details>

<details>
<summary><code>.github/codeql/</code> &mdash; 1 file(s)</summary>

- [`codeql-config.yml`](.github/codeql/codeql-config.yml) &mdash; 25 lines

</details>

<details>
<summary><code>.github/config/</code> &mdash; 8 file(s)</summary>

- [`audit-refresh-map.yml`](.github/config/audit-refresh-map.yml) &mdash; 32 lines
- [`deny-unknown-fields-allowlist.yaml`](.github/config/deny-unknown-fields-allowlist.yaml) &mdash; 20 lines
- [`deny.toml`](.github/config/deny.toml) &mdash; 154 lines
- [`enum-policy.yaml`](.github/config/enum-policy.yaml) &mdash; 507 lines
- [`nextest.toml`](.github/config/nextest.toml) &mdash; 38 lines
- [`panic-allowlist.yaml`](.github/config/panic-allowlist.yaml) &mdash; 101 lines
- [`principle-adr-map.yaml`](.github/config/principle-adr-map.yaml) &mdash; 111 lines
- [`typos.toml`](.github/config/typos.toml) &mdash; 30 lines

</details>

<details>
<summary><code>.github/workflows/</code> &mdash; 13 file(s)</summary>

- [`_quality-gate.yml`](.github/workflows/_quality-gate.yml) &mdash; 351 lines
- [`alloy-release-candidate.yml`](.github/workflows/alloy-release-candidate.yml) &mdash; 134 lines
- [`benchmarks.yml`](.github/workflows/benchmarks.yml) &mdash; 68 lines
- [`browser-wallet-wasm.yml`](.github/workflows/browser-wallet-wasm.yml) &mdash; 212 lines
- [`ci.yml`](.github/workflows/ci.yml) &mdash; 313 lines
- [`codeql.yml`](.github/workflows/codeql.yml) &mdash; 55 lines
- [`commit-format.yml`](.github/workflows/commit-format.yml) &mdash; 98 lines
- [`crate-checks.yml`](.github/workflows/crate-checks.yml) &mdash; 99 lines
- [`docs-quality.yml`](.github/workflows/docs-quality.yml) &mdash; 90 lines
- [`release-readiness.yml`](.github/workflows/release-readiness.yml) &mdash; 326 lines
- [`retry-soak.yml`](.github/workflows/retry-soak.yml) &mdash; 35 lines
- [`upstream-drift.yml`](.github/workflows/upstream-drift.yml) &mdash; 40 lines
- [`wasm.yml`](.github/workflows/wasm.yml) &mdash; 641 lines

</details>

<details>
<summary><code>crates/alloy/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/alloy/Cargo.toml) &mdash; 65 lines
- [`README.md`](crates/alloy/README.md) &mdash; 139 lines

</details>

<details>
<summary><code>crates/alloy-provider/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/alloy-provider/Cargo.toml) &mdash; 53 lines
- [`README.md`](crates/alloy-provider/README.md) &mdash; 128 lines

</details>

<details>
<summary><code>crates/alloy-provider/src/</code> &mdash; 8 file(s)</summary>

- [`builder.rs`](crates/alloy-provider/src/builder.rs) &mdash; 198 lines
- [`client.rs`](crates/alloy-provider/src/client.rs) &mdash; 29 lines
- [`conversion.rs`](crates/alloy-provider/src/conversion.rs) &mdash; 326 lines
- [`error.rs`](crates/alloy-provider/src/error.rs) &mdash; 258 lines
- [`lib.rs`](crates/alloy-provider/src/lib.rs) &mdash; 196 lines
- [`provider.rs`](crates/alloy-provider/src/provider.rs) &mdash; 178 lines
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
- [`provider_contract.rs`](crates/alloy-provider/tests/provider_contract.rs) &mdash; 326 lines
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

- [`Cargo.toml`](crates/alloy-signer/Cargo.toml) &mdash; 51 lines
- [`README.md`](crates/alloy-signer/README.md) &mdash; 129 lines

</details>

<details>
<summary><code>crates/alloy-signer/src/</code> &mdash; 5 file(s)</summary>

- [`builder.rs`](crates/alloy-signer/src/builder.rs) &mdash; 262 lines
- [`conversion.rs`](crates/alloy-signer/src/conversion.rs) &mdash; 295 lines
- [`error.rs`](crates/alloy-signer/src/error.rs) &mdash; 212 lines
- [`lib.rs`](crates/alloy-signer/src/lib.rs) &mdash; 64 lines
- [`signer.rs`](crates/alloy-signer/src/signer.rs) &mdash; 116 lines

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
- [`signer_contract.rs`](crates/alloy-signer/tests/signer_contract.rs) &mdash; 133 lines
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

- [`builder.rs`](crates/alloy/src/builder.rs) &mdash; 389 lines
- [`client.rs`](crates/alloy/src/client.rs) &mdash; 262 lines
- [`conversion.rs`](crates/alloy/src/conversion.rs) &mdash; 17 lines
- [`error.rs`](crates/alloy/src/error.rs) &mdash; 279 lines
- [`handle.rs`](crates/alloy/src/handle.rs) &mdash; 131 lines
- [`lib.rs`](crates/alloy/src/lib.rs) &mdash; 72 lines

</details>

<details>
<summary><code>crates/alloy/tests/</code> &mdash; 16 file(s)</summary>

- [`builder_contract.rs`](crates/alloy/tests/builder_contract.rs) &mdash; 282 lines
- [`cancellation_contract.rs`](crates/alloy/tests/cancellation_contract.rs) &mdash; 28 lines
- [`chain_coherence_mismatch.rs`](crates/alloy/tests/chain_coherence_mismatch.rs) &mdash; 93 lines
- [`chain_coherence.rs`](crates/alloy/tests/chain_coherence.rs) &mdash; 35 lines
- [`compile_fail.rs`](crates/alloy/tests/compile_fail.rs) &mdash; 8 lines
- [`eip712_reference_vectors.rs`](crates/alloy/tests/eip712_reference_vectors.rs) &mdash; 92 lines
- [`error_contract.rs`](crates/alloy/tests/error_contract.rs) &mdash; 215 lines
- [`handle_survives_drop.rs`](crates/alloy/tests/handle_survives_drop.rs) &mdash; 32 lines
- [`log_provider_contract.rs`](crates/alloy/tests/log_provider_contract.rs) &mdash; 40 lines
- [`no_broadcast_for_sign_transaction.rs`](crates/alloy/tests/no_broadcast_for_sign_transaction.rs) &mdash; 41 lines
- [`provider_contract.rs`](crates/alloy/tests/provider_contract.rs) &mdash; 224 lines
- [`read_contract_contract.rs`](crates/alloy/tests/read_contract_contract.rs) &mdash; 129 lines
- [`redaction_contract.rs`](crates/alloy/tests/redaction_contract.rs) &mdash; 208 lines
- [`send_transaction_does_not_wait_for_confirmation.rs`](crates/alloy/tests/send_transaction_does_not_wait_for_confirmation.rs) &mdash; 151 lines
- [`signer_error_trait_contract.rs`](crates/alloy/tests/signer_error_trait_contract.rs) &mdash; 43 lines
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
- [`README.md`](crates/app-data/README.md) &mdash; 139 lines

</details>

<details>
<summary><code>crates/app-data/benches/</code> &mdash; 1 file(s)</summary>

- [`stringify.rs`](crates/app-data/benches/stringify.rs) &mdash; 38 lines

</details>

<details>
<summary><code>crates/app-data/src/</code> &mdash; 6 file(s)</summary>

- [`cid.rs`](crates/app-data/src/cid.rs) &mdash; 143 lines
- [`errors.rs`](crates/app-data/src/errors.rs) &mdash; 227 lines
- [`fetch.rs`](crates/app-data/src/fetch.rs) &mdash; 189 lines
- [`info.rs`](crates/app-data/src/info.rs) &mdash; 359 lines
- [`lib.rs`](crates/app-data/src/lib.rs) &mdash; 64 lines
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
- [`ipfs.rs`](crates/app-data/src/types/ipfs.rs) &mdash; 24 lines
- [`mod.rs`](crates/app-data/src/types/mod.rs) &mdash; 21 lines
- [`params.rs`](crates/app-data/src/types/params.rs) &mdash; 360 lines
- [`partner_fee.rs`](crates/app-data/src/types/partner_fee.rs) &mdash; 386 lines
- [`validation.rs`](crates/app-data/src/types/validation.rs) &mdash; 31 lines

</details>

<details>
<summary><code>crates/app-data/tests/</code> &mdash; 18 file(s)</summary>

- [`app_data_info_contract.rs`](crates/app-data/tests/app_data_info_contract.rs) &mdash; 95 lines
- [`canonical_json_contract.rs`](crates/app-data/tests/canonical_json_contract.rs) &mdash; 44 lines
- [`cid_contract.rs`](crates/app-data/tests/cid_contract.rs) &mdash; 105 lines
- [`error_contract.rs`](crates/app-data/tests/error_contract.rs) &mdash; 13 lines
- [`error_variant_shape.rs`](crates/app-data/tests/error_variant_shape.rs) &mdash; 91 lines
- [`fetch_contract.rs`](crates/app-data/tests/fetch_contract.rs) &mdash; 245 lines
- [`fetch_telemetry_contract.rs`](crates/app-data/tests/fetch_telemetry_contract.rs) &mdash; 83 lines
- [`flashloan_contract.rs`](crates/app-data/tests/flashloan_contract.rs) &mdash; 295 lines
- [`hooks_contract.rs`](crates/app-data/tests/hooks_contract.rs) &mdash; 159 lines
- [`ipfs_config_redaction_contract.rs`](crates/app-data/tests/ipfs_config_redaction_contract.rs) &mdash; 52 lines
- [`json_recursion_contract.rs`](crates/app-data/tests/json_recursion_contract.rs) &mdash; 24 lines
- [`metadata_signer_contract.rs`](crates/app-data/tests/metadata_signer_contract.rs) &mdash; 171 lines
- [`partner_fee_contract.rs`](crates/app-data/tests/partner_fee_contract.rs) &mdash; 426 lines
- [`property_contract.rs`](crates/app-data/tests/property_contract.rs) &mdash; 326 lines
- [`schema_contract.rs`](crates/app-data/tests/schema_contract.rs) &mdash; 158 lines
- [`schema_drift_contract.rs`](crates/app-data/tests/schema_drift_contract.rs) &mdash; 78 lines
- [`typed_metadata_contract.rs`](crates/app-data/tests/typed_metadata_contract.rs) &mdash; 67 lines
- [`validated_shape_contract.rs`](crates/app-data/tests/validated_shape_contract.rs) &mdash; 138 lines

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
<summary><code>crates/browser-wallet/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/browser-wallet/Cargo.toml) &mdash; 52 lines
- [`README.md`](crates/browser-wallet/README.md) &mdash; 73 lines

</details>

<details>
<summary><code>crates/browser-wallet/src/</code> &mdash; 6 file(s)</summary>

- [`error.rs`](crates/browser-wallet/src/error.rs) &mdash; 422 lines
- [`events.rs`](crates/browser-wallet/src/events.rs) &mdash; 220 lines
- [`js.rs`](crates/browser-wallet/src/js.rs) &mdash; 602 lines
- [`lib.rs`](crates/browser-wallet/src/lib.rs) &mdash; 66 lines
- [`mock.rs`](crates/browser-wallet/src/mock.rs) &mdash; 521 lines
- [`signer.rs`](crates/browser-wallet/src/signer.rs) &mdash; 320 lines

</details>

<details>
<summary><code>crates/browser-wallet/src/provider/</code> &mdash; 6 file(s)</summary>

- [`builder.rs`](crates/browser-wallet/src/provider/builder.rs) &mdash; 140 lines
- [`mod.rs`](crates/browser-wallet/src/provider/mod.rs) &mdash; 175 lines
- [`origin.rs`](crates/browser-wallet/src/provider/origin.rs) &mdash; 75 lines
- [`provider_impl.rs`](crates/browser-wallet/src/provider/provider_impl.rs) &mdash; 779 lines
- [`signing_provider_impl.rs`](crates/browser-wallet/src/provider/signing_provider_impl.rs) &mdash; 34 lines
- [`transport.rs`](crates/browser-wallet/src/provider/transport.rs) &mdash; 37 lines

</details>

<details>
<summary><code>crates/browser-wallet/src/wallet/</code> &mdash; 5 file(s)</summary>

- [`chain_mgmt.rs`](crates/browser-wallet/src/wallet/chain_mgmt.rs) &mdash; 164 lines
- [`chain.rs`](crates/browser-wallet/src/wallet/chain.rs) &mdash; 362 lines
- [`detect.rs`](crates/browser-wallet/src/wallet/detect.rs) &mdash; 133 lines
- [`discovery.rs`](crates/browser-wallet/src/wallet/discovery.rs) &mdash; 399 lines
- [`mod.rs`](crates/browser-wallet/src/wallet/mod.rs) &mdash; 258 lines

</details>

<details>
<summary><code>crates/browser-wallet/tests/</code> &mdash; 11 file(s)</summary>

- [`non_exhaustive_type_contract.rs`](crates/browser-wallet/tests/non_exhaustive_type_contract.rs) &mdash; 81 lines
- [`origin_contract.rs`](crates/browser-wallet/tests/origin_contract.rs) &mdash; 135 lines
- [`provider_contract.rs`](crates/browser-wallet/tests/provider_contract.rs) &mdash; 135 lines
- [`signer_contract.rs`](crates/browser-wallet/tests/signer_contract.rs) &mdash; 484 lines
- [`signer_error_trait_contract.rs`](crates/browser-wallet/tests/signer_error_trait_contract.rs) &mdash; 133 lines
- [`signing_provider_contract.rs`](crates/browser-wallet/tests/signing_provider_contract.rs) &mdash; 161 lines
- [`state_machine_contract.rs`](crates/browser-wallet/tests/state_machine_contract.rs) &mdash; 162 lines
- [`transaction_receipt_parsing.rs`](crates/browser-wallet/tests/transaction_receipt_parsing.rs) &mdash; 251 lines
- [`wallet_contract.rs`](crates/browser-wallet/tests/wallet_contract.rs) &mdash; 769 lines
- [`wallet_telemetry_contract.rs`](crates/browser-wallet/tests/wallet_telemetry_contract.rs) &mdash; 41 lines
- [`wasm_bridge_contract.rs`](crates/browser-wallet/tests/wasm_bridge_contract.rs) &mdash; 765 lines

</details>

<details>
<summary><code>crates/contracts/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/contracts/Cargo.toml) &mdash; 66 lines
- [`README.md`](crates/contracts/README.md) &mdash; 69 lines

</details>

<details>
<summary><code>crates/contracts/benches/</code> &mdash; 2 file(s)</summary>

- [`order_hashing.rs`](crates/contracts/benches/order_hashing.rs) &mdash; 27 lines
- [`uid_packing.rs`](crates/contracts/benches/uid_packing.rs) &mdash; 43 lines

</details>

<details>
<summary><code>crates/contracts/src/</code> &mdash; 13 file(s)</summary>

- [`deployments.rs`](crates/contracts/src/deployments.rs) &mdash; 444 lines
- [`errors.rs`](crates/contracts/src/errors.rs) &mdash; 226 lines
- [`eth_flow.rs`](crates/contracts/src/eth_flow.rs) &mdash; 607 lines
- [`hex_field.rs`](crates/contracts/src/hex_field.rs) &mdash; 234 lines
- [`interaction.rs`](crates/contracts/src/interaction.rs) &mdash; 116 lines
- [`lib.rs`](crates/contracts/src/lib.rs) &mdash; 85 lines
- [`onchain_orders.rs`](crates/contracts/src/onchain_orders.rs) &mdash; 312 lines
- [`order.rs`](crates/contracts/src/order.rs) &mdash; 529 lines
- [`primitives.rs`](crates/contracts/src/primitives.rs) &mdash; 233 lines
- [`settlement.rs`](crates/contracts/src/settlement.rs) &mdash; 288 lines
- [`signature.rs`](crates/contracts/src/signature.rs) &mdash; 617 lines
- [`tokens.rs`](crates/contracts/src/tokens.rs) &mdash; 98 lines
- [`verify.rs`](crates/contracts/src/verify.rs) &mdash; 219 lines

</details>

<details>
<summary><code>crates/contracts/src/cow_shed/</code> &mdash; 8 file(s)</summary>

- [`bindings.rs`](crates/contracts/src/cow_shed/bindings.rs) &mdash; 153 lines
- [`calls.rs`](crates/contracts/src/cow_shed/calls.rs) &mdash; 145 lines
- [`eip712.rs`](crates/contracts/src/cow_shed/eip712.rs) &mdash; 168 lines
- [`errors.rs`](crates/contracts/src/cow_shed/errors.rs) &mdash; 88 lines
- [`hooks.rs`](crates/contracts/src/cow_shed/hooks.rs) &mdash; 286 lines
- [`mod.rs`](crates/contracts/src/cow_shed/mod.rs) &mdash; 80 lines
- [`types.rs`](crates/contracts/src/cow_shed/types.rs) &mdash; 87 lines
- [`version.rs`](crates/contracts/src/cow_shed/version.rs) &mdash; 49 lines

</details>

<details>
<summary><code>crates/contracts/src/cow_shed/address/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/contracts/src/cow_shed/address/mod.rs) &mdash; 165 lines

</details>

<details>
<summary><code>crates/contracts/src/cow_shed/address/proxy-creation-code/</code> &mdash; 2 file(s)</summary>

- [`v1.0.0.bin`](crates/contracts/src/cow_shed/address/proxy-creation-code/v1.0.0.bin)
- [`v1.0.1.bin`](crates/contracts/src/cow_shed/address/proxy-creation-code/v1.0.1.bin)

</details>

<details>
<summary><code>crates/contracts/tests/</code> &mdash; 28 file(s)</summary>

- [`calldata_parity_contract.rs`](crates/contracts/tests/calldata_parity_contract.rs) &mdash; 150 lines
- [`deployment_address_parity_contract.rs`](crates/contracts/tests/deployment_address_parity_contract.rs) &mdash; 154 lines
- [`domain_separator_parity_contract.rs`](crates/contracts/tests/domain_separator_parity_contract.rs) &mdash; 35 lines
- [`eip712_message_hash_parity_contract.rs`](crates/contracts/tests/eip712_message_hash_parity_contract.rs) &mdash; 101 lines
- [`eip712_type_hash_parity_contract.rs`](crates/contracts/tests/eip712_type_hash_parity_contract.rs) &mdash; 66 lines
- [`eoa_signature_byte_order_contract.rs`](crates/contracts/tests/eoa_signature_byte_order_contract.rs) &mdash; 101 lines
- [`error_contract.rs`](crates/contracts/tests/error_contract.rs) &mdash; 197 lines
- [`eth_flow_events_contract.rs`](crates/contracts/tests/eth_flow_events_contract.rs) &mdash; 143 lines
- [`init_code_derivation_contract.rs`](crates/contracts/tests/init_code_derivation_contract.rs) &mdash; 44 lines
- [`interaction_contract.rs`](crates/contracts/tests/interaction_contract.rs) &mdash; 79 lines
- [`non_exhaustive_dto_contract.rs`](crates/contracts/tests/non_exhaustive_dto_contract.rs) &mdash; 215 lines
- [`onchain_orders.rs`](crates/contracts/tests/onchain_orders.rs) &mdash; 296 lines
- [`order_contract.rs`](crates/contracts/tests/order_contract.rs) &mdash; 133 lines
- [`order_digest_parity_contract.rs`](crates/contracts/tests/order_digest_parity_contract.rs) &mdash; 156 lines
- [`parity_contract.rs`](crates/contracts/tests/parity_contract.rs) &mdash; 567 lines
- [`property_contract.rs`](crates/contracts/tests/property_contract.rs) &mdash; 441 lines
- [`proxy_address_parity_contract.rs`](crates/contracts/tests/proxy_address_parity_contract.rs) &mdash; 54 lines
- [`recoverable_signature_contract.rs`](crates/contracts/tests/recoverable_signature_contract.rs) &mdash; 307 lines
- [`selector_parity_contract.rs`](crates/contracts/tests/selector_parity_contract.rs) &mdash; 39 lines
- [`selector_parity_cow_shed_contract.rs`](crates/contracts/tests/selector_parity_cow_shed_contract.rs) &mdash; 103 lines
- [`settlement_events_contract.rs`](crates/contracts/tests/settlement_events_contract.rs) &mdash; 199 lines
- [`sign_telemetry_contract.rs`](crates/contracts/tests/sign_telemetry_contract.rs) &mdash; 58 lines
- [`signature_contract.rs`](crates/contracts/tests/signature_contract.rs) &mdash; 670 lines
- [`signed_calldata_parity_contract.rs`](crates/contracts/tests/signed_calldata_parity_contract.rs) &mdash; 143 lines
- [`tokens_contract.rs`](crates/contracts/tests/tokens_contract.rs) &mdash; 235 lines
- [`ui.rs`](crates/contracts/tests/ui.rs) &mdash; 11 lines
- [`v_normalization_contract.rs`](crates/contracts/tests/v_normalization_contract.rs) &mdash; 105 lines
- [`verify_telemetry_contract.rs`](crates/contracts/tests/verify_telemetry_contract.rs) &mdash; 204 lines

</details>

<details>
<summary><code>crates/contracts/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/contracts/tests/common/mod.rs) &mdash; 170 lines

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

- [`Cargo.toml`](crates/core/Cargo.toml) &mdash; 76 lines
- [`README.md`](crates/core/README.md) &mdash; 70 lines

</details>

<details>
<summary><code>crates/core/src/</code> &mdash; 5 file(s)</summary>

- [`cancellation.rs`](crates/core/src/cancellation.rs) &mdash; 122 lines
- [`errors.rs`](crates/core/src/errors.rs) &mdash; 178 lines
- [`lib.rs`](crates/core/src/lib.rs) &mdash; 103 lines
- [`prelude.rs`](crates/core/src/prelude.rs) &mdash; 25 lines
- [`validation.rs`](crates/core/src/validation.rs) &mdash; 116 lines

</details>

<details>
<summary><code>crates/core/src/config/</code> &mdash; 6 file(s)</summary>

- [`chains.rs`](crates/core/src/config/chains.rs) &mdash; 244 lines
- [`env.rs`](crates/core/src/config/env.rs) &mdash; 71 lines
- [`hosts.rs`](crates/core/src/config/hosts.rs) &mdash; 236 lines
- [`http.rs`](crates/core/src/config/http.rs) &mdash; 122 lines
- [`mod.rs`](crates/core/src/config/mod.rs) &mdash; 36 lines
- [`protocol.rs`](crates/core/src/config/protocol.rs) &mdash; 161 lines

</details>

<details>
<summary><code>crates/core/src/redaction/</code> &mdash; 3 file(s)</summary>

- [`body.rs`](crates/core/src/redaction/body.rs) &mdash; 397 lines
- [`mod.rs`](crates/core/src/redaction/mod.rs) &mdash; 21 lines
- [`wrappers.rs`](crates/core/src/redaction/wrappers.rs) &mdash; 363 lines

</details>

<details>
<summary><code>crates/core/src/traits/</code> &mdash; 5 file(s)</summary>

- [`mod.rs`](crates/core/src/traits/mod.rs) &mdash; 8 lines
- [`provider.rs`](crates/core/src/traits/provider.rs) &mdash; 288 lines
- [`signer.rs`](crates/core/src/traits/signer.rs) &mdash; 243 lines
- [`transaction.rs`](crates/core/src/traits/transaction.rs) &mdash; 209 lines
- [`typed_data.rs`](crates/core/src/traits/typed_data.rs) &mdash; 199 lines

</details>

<details>
<summary><code>crates/core/src/transport/</code> &mdash; 4 file(s)</summary>

- [`error.rs`](crates/core/src/transport/error.rs) &mdash; 77 lines
- [`http.rs`](crates/core/src/transport/http.rs) &mdash; 118 lines
- [`mod.rs`](crates/core/src/transport/mod.rs) &mdash; 185 lines
- [`reqwest.rs`](crates/core/src/transport/reqwest.rs) &mdash; 529 lines

</details>

<details>
<summary><code>crates/core/src/transport/policy/</code> &mdash; 10 file(s)</summary>

- [`classify.rs`](crates/core/src/transport/policy/classify.rs) &mdash; 95 lines
- [`config.rs`](crates/core/src/transport/policy/config.rs) &mdash; 356 lines
- [`jitter.rs`](crates/core/src/transport/policy/jitter.rs) &mdash; 136 lines
- [`mod.rs`](crates/core/src/transport/policy/mod.rs) &mdash; 64 lines
- [`rate_limit.rs`](crates/core/src/transport/policy/rate_limit.rs) &mdash; 295 lines
- [`retry_after.rs`](crates/core/src/transport/policy/retry_after.rs) &mdash; 153 lines
- [`retry.rs`](crates/core/src/transport/policy/retry.rs) &mdash; 220 lines
- [`runner.rs`](crates/core/src/transport/policy/runner.rs) &mdash; 528 lines
- [`status.rs`](crates/core/src/transport/policy/status.rs) &mdash; 42 lines
- [`time.rs`](crates/core/src/transport/policy/time.rs) &mdash; 66 lines

</details>

<details>
<summary><code>crates/core/src/types/</code> &mdash; 8 file(s)</summary>

- [`amount.rs`](crates/core/src/types/amount.rs) &mdash; 575 lines
- [`app_code.rs`](crates/core/src/types/app_code.rs) &mdash; 215 lines
- [`identity.rs`](crates/core/src/types/identity.rs) &mdash; 1,029 lines
- [`logs.rs`](crates/core/src/types/logs.rs) &mdash; 281 lines
- [`mod.rs`](crates/core/src/types/mod.rs) &mdash; 72 lines
- [`order.rs`](crates/core/src/types/order.rs) &mdash; 245 lines
- [`quote.rs`](crates/core/src/types/quote.rs) &mdash; 166 lines
- [`validity.rs`](crates/core/src/types/validity.rs) &mdash; 101 lines

</details>

<details>
<summary><code>crates/core/tests/</code> &mdash; 21 file(s)</summary>

- [`address_literal_ui.rs`](crates/core/tests/address_literal_ui.rs) &mdash; 27 lines
- [`amount_arithmetic_ui.rs`](crates/core/tests/amount_arithmetic_ui.rs) &mdash; 30 lines
- [`cancellation_contract.rs`](crates/core/tests/cancellation_contract.rs) &mdash; 126 lines
- [`cancellation_coverage_validator.rs`](crates/core/tests/cancellation_coverage_validator.rs) &mdash; 232 lines
- [`classify_contract.rs`](crates/core/tests/classify_contract.rs) &mdash; 151 lines
- [`config_contract.rs`](crates/core/tests/config_contract.rs) &mdash; 239 lines
- [`policy_contract.rs`](crates/core/tests/policy_contract.rs) &mdash; 701 lines
- [`property_contract.rs`](crates/core/tests/property_contract.rs) &mdash; 642 lines
- [`provider_capability_split_contract.rs`](crates/core/tests/provider_capability_split_contract.rs) &mdash; 294 lines
- [`redaction_contract.rs`](crates/core/tests/redaction_contract.rs) &mdash; 209 lines
- [`retry_after_contract.proptest-regressions`](crates/core/tests/retry_after_contract.proptest-regressions) &mdash; 7 lines
- [`retry_after_contract.rs`](crates/core/tests/retry_after_contract.rs) &mdash; 295 lines
- [`retry_after_fixture_contract.rs`](crates/core/tests/retry_after_fixture_contract.rs) &mdash; 118 lines
- [`token_balance_parity.rs`](crates/core/tests/token_balance_parity.rs) &mdash; 89 lines
- [`token_balance_ui.rs`](crates/core/tests/token_balance_ui.rs) &mdash; 20 lines
- [`trait_evolution_contract.rs`](crates/core/tests/trait_evolution_contract.rs) &mdash; 203 lines
- [`traits_contract.rs`](crates/core/tests/traits_contract.rs) &mdash; 512 lines
- [`transport_contract.rs`](crates/core/tests/transport_contract.rs) &mdash; 651 lines
- [`types_contract.rs`](crates/core/tests/types_contract.rs) &mdash; 707 lines
- [`wasm_sleep_zero_delay_contract.rs`](crates/core/tests/wasm_sleep_zero_delay_contract.rs) &mdash; 35 lines
- [`wire_format_preservation_contract.rs`](crates/core/tests/wire_format_preservation_contract.rs) &mdash; 339 lines

</details>

<details>
<summary><code>crates/core/tests/proptest-regressions/</code> &mdash; 1 file(s)</summary>

- [`property_contract.txt`](crates/core/tests/proptest-regressions/property_contract.txt) &mdash; 9 lines

</details>

<details>
<summary><code>crates/core/tests/ui/</code> &mdash; 12 file(s)</summary>

- [`address_literal_bad_checksum.rs`](crates/core/tests/ui/address_literal_bad_checksum.rs) &mdash; 14 lines
- [`address_literal_bad_checksum.stderr`](crates/core/tests/ui/address_literal_bad_checksum.stderr) &mdash; 18 lines
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

- [`Cargo.toml`](crates/orderbook/Cargo.toml) &mdash; 52 lines
- [`README.md`](crates/orderbook/README.md) &mdash; 67 lines

</details>

<details>
<summary><code>crates/orderbook/benches/</code> &mdash; 1 file(s)</summary>

- [`quote_cost.rs`](crates/orderbook/benches/quote_cost.rs) &mdash; 17 lines

</details>

<details>
<summary><code>crates/orderbook/src/</code> &mdash; 7 file(s)</summary>

- [`api.rs`](crates/orderbook/src/api.rs) &mdash; 906 lines
- [`builder.rs`](crates/orderbook/src/builder.rs) &mdash; 443 lines
- [`error.rs`](crates/orderbook/src/error.rs) &mdash; 529 lines
- [`lib.rs`](crates/orderbook/src/lib.rs) &mdash; 239 lines
- [`rejection.rs`](crates/orderbook/src/rejection.rs) &mdash; 569 lines
- [`request.rs`](crates/orderbook/src/request.rs) &mdash; 785 lines
- [`transform.rs`](crates/orderbook/src/transform.rs) &mdash; 89 lines

</details>

<details>
<summary><code>crates/orderbook/src/types/</code> &mdash; 8 file(s)</summary>

- [`app_data.rs`](crates/orderbook/src/types/app_data.rs) &mdash; 159 lines
- [`auction.rs`](crates/orderbook/src/types/auction.rs) &mdash; 345 lines
- [`enums.rs`](crates/orderbook/src/types/enums.rs) &mdash; 202 lines
- [`lists.rs`](crates/orderbook/src/types/lists.rs) &mdash; 192 lines
- [`mod.rs`](crates/orderbook/src/types/mod.rs) &mdash; 109 lines
- [`order.rs`](crates/orderbook/src/types/order.rs) &mdash; 907 lines
- [`prices.rs`](crates/orderbook/src/types/prices.rs) &mdash; 59 lines
- [`quote.rs`](crates/orderbook/src/types/quote.rs) &mdash; 937 lines

</details>

<details>
<summary><code>crates/orderbook/tests/</code> &mdash; 15 file(s)</summary>

- [`api_contract.rs`](crates/orderbook/tests/api_contract.rs) &mdash; 1,256 lines
- [`builder_contract.rs`](crates/orderbook/tests/builder_contract.rs) &mdash; 251 lines
- [`cancellation_composition_contract.rs`](crates/orderbook/tests/cancellation_composition_contract.rs) &mdash; 475 lines
- [`error_variant_shape.rs`](crates/orderbook/tests/error_variant_shape.rs) &mdash; 112 lines
- [`fee_amount_is_not_a_public_builder_setter.rs`](crates/orderbook/tests/fee_amount_is_not_a_public_builder_setter.rs) &mdash; 198 lines
- [`host_policy_contract.rs`](crates/orderbook/tests/host_policy_contract.rs) &mdash; 112 lines
- [`invariant_contract.rs`](crates/orderbook/tests/invariant_contract.rs) &mdash; 330 lines
- [`order_creation_fee_deserialize.rs`](crates/orderbook/tests/order_creation_fee_deserialize.rs) &mdash; 153 lines
- [`rejection_category_contract.rs`](crates/orderbook/tests/rejection_category_contract.rs) &mdash; 81 lines
- [`rejection_contract.rs`](crates/orderbook/tests/rejection_contract.rs) &mdash; 567 lines
- [`request_contract.rs`](crates/orderbook/tests/request_contract.rs) &mdash; 977 lines
- [`signing_scheme_bridge_contract.rs`](crates/orderbook/tests/signing_scheme_bridge_contract.rs) &mdash; 166 lines
- [`transform_contract.rs`](crates/orderbook/tests/transform_contract.rs) &mdash; 358 lines
- [`types_contract.rs`](crates/orderbook/tests/types_contract.rs) &mdash; 516 lines
- [`wire_contract.rs`](crates/orderbook/tests/wire_contract.rs) &mdash; 218 lines

</details>

<details>
<summary><code>crates/orderbook/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/orderbook/tests/common/mod.rs) &mdash; 239 lines

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

- [`Cargo.toml`](crates/sdk/Cargo.toml) &mdash; 103 lines
- [`README.md`](crates/sdk/README.md) &mdash; 147 lines

</details>

<details>
<summary><code>crates/sdk/src/</code> &mdash; 1 file(s)</summary>

- [`lib.rs`](crates/sdk/src/lib.rs) &mdash; 326 lines

</details>

<details>
<summary><code>crates/sdk/tests/</code> &mdash; 7 file(s)</summary>

- [`amount_roundtrip.rs`](crates/sdk/tests/amount_roundtrip.rs) &mdash; 38 lines
- [`error_class_contract.rs`](crates/sdk/tests/error_class_contract.rs) &mdash; 285 lines
- [`error_redaction_contract.rs`](crates/sdk/tests/error_redaction_contract.rs) &mdash; 1,008 lines
- [`public_api_default_features_only.rs`](crates/sdk/tests/public_api_default_features_only.rs) &mdash; 64 lines
- [`public_api_with_all_features.rs`](crates/sdk/tests/public_api_with_all_features.rs) &mdash; 58 lines
- [`public_api.rs`](crates/sdk/tests/public_api.rs) &mdash; 153 lines
- [`ui.rs`](crates/sdk/tests/ui.rs) &mdash; 5 lines

</details>

<details>
<summary><code>crates/sdk/tests/fixtures/</code> &mdash; 2 file(s)</summary>

- [`public_api_default_features_only.snap`](crates/sdk/tests/fixtures/public_api_default_features_only.snap) &mdash; 15 lines
- [`public_api_with_all_features.snap`](crates/sdk/tests/fixtures/public_api_with_all_features.snap) &mdash; 14 lines

</details>

<details>
<summary><code>crates/sdk/tests/ui/</code> &mdash; 1 file(s)</summary>

- [`orderbook_client_reachable_through_trading_re_export.rs`](crates/sdk/tests/ui/orderbook_client_reachable_through_trading_re_export.rs) &mdash; 5 lines

</details>

<details>
<summary><code>crates/signing/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/signing/Cargo.toml) &mdash; 63 lines
- [`README.md`](crates/signing/README.md) &mdash; 97 lines

</details>

<details>
<summary><code>crates/signing/benches/</code> &mdash; 1 file(s)</summary>

- [`typed_data.rs`](crates/signing/benches/typed_data.rs) &mdash; 30 lines

</details>

<details>
<summary><code>crates/signing/src/</code> &mdash; 6 file(s)</summary>

- [`cache.rs`](crates/signing/src/cache.rs) &mdash; 262 lines
- [`cancellation.rs`](crates/signing/src/cancellation.rs) &mdash; 195 lines
- [`domain.rs`](crates/signing/src/domain.rs) &mdash; 211 lines
- [`errors.rs`](crates/signing/src/errors.rs) &mdash; 74 lines
- [`lib.rs`](crates/signing/src/lib.rs) &mdash; 52 lines
- [`order_signing.rs`](crates/signing/src/order_signing.rs) &mdash; 469 lines

</details>

<details>
<summary><code>crates/signing/src/eip1271/</code> &mdash; 4 file(s)</summary>

- [`error.rs`](crates/signing/src/eip1271/error.rs) &mdash; 27 lines
- [`mod.rs`](crates/signing/src/eip1271/mod.rs) &mdash; 9 lines
- [`provider.rs`](crates/signing/src/eip1271/provider.rs) &mdash; 44 lines
- [`sol_types.rs`](crates/signing/src/eip1271/sol_types.rs) &mdash; 75 lines

</details>

<details>
<summary><code>crates/signing/tests/</code> &mdash; 8 file(s)</summary>

- [`cancellation_contract.rs`](crates/signing/tests/cancellation_contract.rs) &mdash; 193 lines
- [`domain_contract.rs`](crates/signing/tests/domain_contract.rs) &mdash; 118 lines
- [`eip1271_cache_contract.rs`](crates/signing/tests/eip1271_cache_contract.rs) &mdash; 601 lines
- [`eip1271_contract.rs`](crates/signing/tests/eip1271_contract.rs) &mdash; 157 lines
- [`order_signing_contract.rs`](crates/signing/tests/order_signing_contract.rs) &mdash; 306 lines
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
- [`README.md`](crates/subgraph/README.md) &mdash; 72 lines

</details>

<details>
<summary><code>crates/subgraph/src/</code> &mdash; 6 file(s)</summary>

- [`api.rs`](crates/subgraph/src/api.rs) &mdash; 755 lines
- [`builder.rs`](crates/subgraph/src/builder.rs) &mdash; 365 lines
- [`error.rs`](crates/subgraph/src/error.rs) &mdash; 403 lines
- [`lib.rs`](crates/subgraph/src/lib.rs) &mdash; 36 lines
- [`queries.rs`](crates/subgraph/src/queries.rs) &mdash; 12 lines
- [`types.rs`](crates/subgraph/src/types.rs) &mdash; 327 lines

</details>

<details>
<summary><code>crates/subgraph/src/query_documents/</code> &mdash; 3 file(s)</summary>

- [`last_days_volume.graphql`](crates/subgraph/src/query_documents/last_days_volume.graphql) &mdash; 6 lines
- [`last_hours_volume.graphql`](crates/subgraph/src/query_documents/last_hours_volume.graphql) &mdash; 6 lines
- [`totals.graphql`](crates/subgraph/src/query_documents/totals.graphql) &mdash; 12 lines

</details>

<details>
<summary><code>crates/subgraph/tests/</code> &mdash; 8 file(s)</summary>

- [`api_contract.rs`](crates/subgraph/tests/api_contract.rs) &mdash; 1,181 lines
- [`builder_contract.rs`](crates/subgraph/tests/builder_contract.rs) &mdash; 233 lines
- [`cancellation_composition_contract.rs`](crates/subgraph/tests/cancellation_composition_contract.rs) &mdash; 206 lines
- [`error_contract.rs`](crates/subgraph/tests/error_contract.rs) &mdash; 252 lines
- [`error_redaction_contract.rs`](crates/subgraph/tests/error_redaction_contract.rs) &mdash; 104 lines
- [`host_policy_contract.rs`](crates/subgraph/tests/host_policy_contract.rs) &mdash; 94 lines
- [`query_contract.rs`](crates/subgraph/tests/query_contract.rs) &mdash; 188 lines
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

- [`Cargo.toml`](crates/test/Cargo.toml) &mdash; 23 lines
- [`README.md`](crates/test/README.md) &mdash; 59 lines

</details>

<details>
<summary><code>crates/test-utils/</code> &mdash; 1 file(s)</summary>

- [`Cargo.toml`](crates/test-utils/Cargo.toml) &mdash; 30 lines

</details>

<details>
<summary><code>crates/test-utils/src/</code> &mdash; 8 file(s)</summary>

- [`arb.rs`](crates/test-utils/src/arb.rs) &mdash; 55 lines
- [`builders.rs`](crates/test-utils/src/builders.rs) &mdash; 173 lines
- [`consts.rs`](crates/test-utils/src/consts.rs) &mdash; 68 lines
- [`eip712.rs`](crates/test-utils/src/eip712.rs) &mdash; 110 lines
- [`fixtures.rs`](crates/test-utils/src/fixtures.rs) &mdash; 67 lines
- [`lib.rs`](crates/test-utils/src/lib.rs) &mdash; 20 lines
- [`mocks.rs`](crates/test-utils/src/mocks.rs) &mdash; 326 lines
- [`trace.rs`](crates/test-utils/src/trace.rs) &mdash; 361 lines

</details>

<details>
<summary><code>crates/test-utils/tests/</code> &mdash; 1 file(s)</summary>

- [`smoke.rs`](crates/test-utils/tests/smoke.rs) &mdash; 174 lines

</details>

<details>
<summary><code>crates/test/src/</code> &mdash; 6 file(s)</summary>

- [`defaults.rs`](crates/test/src/defaults.rs) &mdash; 94 lines
- [`error.rs`](crates/test/src/error.rs) &mdash; 95 lines
- [`lib.rs`](crates/test/src/lib.rs) &mdash; 93 lines
- [`orderbook.rs`](crates/test/src/orderbook.rs) &mdash; 215 lines
- [`provider.rs`](crates/test/src/provider.rs) &mdash; 246 lines
- [`signer.rs`](crates/test/src/signer.rs) &mdash; 232 lines

</details>

<details>
<summary><code>crates/test/tests/</code> &mdash; 1 file(s)</summary>

- [`contract.rs`](crates/test/tests/contract.rs) &mdash; 212 lines

</details>

<details>
<summary><code>crates/trading/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/trading/Cargo.toml) &mdash; 69 lines
- [`README.md`](crates/trading/README.md) &mdash; 231 lines

</details>

<details>
<summary><code>crates/trading/benches/</code> &mdash; 1 file(s)</summary>

- [`order_build.rs`](crates/trading/benches/order_build.rs) &mdash; 51 lines

</details>

<details>
<summary><code>crates/trading/src/</code> &mdash; 13 file(s)</summary>

- [`allowance.rs`](crates/trading/src/allowance.rs) &mdash; 145 lines
- [`app_data.rs`](crates/trading/src/app_data.rs) &mdash; 242 lines
- [`cancel.rs`](crates/trading/src/cancel.rs) &mdash; 73 lines
- [`error.rs`](crates/trading/src/error.rs) &mdash; 235 lines
- [`lib.rs`](crates/trading/src/lib.rs) &mdash; 112 lines
- [`onchain.rs`](crates/trading/src/onchain.rs) &mdash; 504 lines
- [`order.rs`](crates/trading/src/order.rs) &mdash; 365 lines
- [`params.rs`](crates/trading/src/params.rs) &mdash; 109 lines
- [`post.rs`](crates/trading/src/post.rs) &mdash; 803 lines
- [`quote.rs`](crates/trading/src/quote.rs) &mdash; 460 lines
- [`slippage.rs`](crates/trading/src/slippage.rs) &mdash; 785 lines
- [`validation.rs`](crates/trading/src/validation.rs) &mdash; 331 lines
- [`wait.rs`](crates/trading/src/wait.rs) &mdash; 393 lines

</details>

<details>
<summary><code>crates/trading/src/client/</code> &mdash; 5 file(s)</summary>

- [`builder.rs`](crates/trading/src/client/builder.rs) &mdash; 280 lines
- [`helpers.rs`](crates/trading/src/client/helpers.rs) &mdash; 164 lines
- [`methods.rs`](crates/trading/src/client/methods.rs) &mdash; 597 lines
- [`mod.rs`](crates/trading/src/client/mod.rs) &mdash; 94 lines
- [`swap.rs`](crates/trading/src/client/swap.rs) &mdash; 361 lines

</details>

<details>
<summary><code>crates/trading/src/types/</code> &mdash; 4 file(s)</summary>

- [`mod.rs`](crates/trading/src/types/mod.rs) &mdash; 16 lines
- [`params.rs`](crates/trading/src/types/params.rs) &mdash; 1,218 lines
- [`result.rs`](crates/trading/src/types/result.rs) &mdash; 323 lines
- [`seams.rs`](crates/trading/src/types/seams.rs) &mdash; 102 lines

</details>

<details>
<summary><code>crates/trading/tests/</code> &mdash; 23 file(s)</summary>

- [`allowance_contract.rs`](crates/trading/tests/allowance_contract.rs) &mdash; 145 lines
- [`app_code_contract.rs`](crates/trading/tests/app_code_contract.rs) &mdash; 43 lines
- [`app_data_merge_contract.rs`](crates/trading/tests/app_data_merge_contract.rs) &mdash; 616 lines
- [`cancel_contract.rs`](crates/trading/tests/cancel_contract.rs) &mdash; 87 lines
- [`cancellation_composition_contract.rs`](crates/trading/tests/cancellation_composition_contract.rs) &mdash; 543 lines
- [`error_variant_shape.rs`](crates/trading/tests/error_variant_shape.rs) &mdash; 113 lines
- [`invariant_contract.rs`](crates/trading/tests/invariant_contract.rs) &mdash; 442 lines
- [`limit_from_quote_contract.rs`](crates/trading/tests/limit_from_quote_contract.rs) &mdash; 103 lines
- [`onchain_contract.rs`](crates/trading/tests/onchain_contract.rs) &mdash; 324 lines
- [`order_contract.rs`](crates/trading/tests/order_contract.rs) &mdash; 185 lines
- [`parameters_contract.rs`](crates/trading/tests/parameters_contract.rs) &mdash; 133 lines
- [`post_contract.rs`](crates/trading/tests/post_contract.rs) &mdash; 769 lines
- [`property_contract.rs`](crates/trading/tests/property_contract.rs) &mdash; 212 lines
- [`quote_contract.rs`](crates/trading/tests/quote_contract.rs) &mdash; 790 lines
- [`quote_projection_parity.rs`](crates/trading/tests/quote_projection_parity.rs) &mdash; 77 lines
- [`sdk_contract.rs`](crates/trading/tests/sdk_contract.rs) &mdash; 679 lines
- [`slippage_contract.rs`](crates/trading/tests/slippage_contract.rs) &mdash; 250 lines
- [`swap_lifecycle_contract.rs`](crates/trading/tests/swap_lifecycle_contract.rs) &mdash; 146 lines
- [`types_contract.rs`](crates/trading/tests/types_contract.rs) &mdash; 319 lines
- [`ui.rs`](crates/trading/tests/ui.rs) &mdash; 11 lines
- [`validation_contract.rs`](crates/trading/tests/validation_contract.rs) &mdash; 342 lines
- [`wait_helper_contract.rs`](crates/trading/tests/wait_helper_contract.rs) &mdash; 190 lines
- [`wait_telemetry_contract.rs`](crates/trading/tests/wait_telemetry_contract.rs) &mdash; 85 lines

</details>

<details>
<summary><code>crates/trading/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/trading/tests/common/mod.rs) &mdash; 938 lines

</details>

<details>
<summary><code>crates/trading/tests/proptest-regressions/</code> &mdash; 1 file(s)</summary>

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
<summary><code>crates/transport-wasm/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/transport-wasm/Cargo.toml) &mdash; 57 lines
- [`README.md`](crates/transport-wasm/README.md) &mdash; 34 lines

</details>

<details>
<summary><code>crates/transport-wasm/src/</code> &mdash; 2 file(s)</summary>

- [`fetch.rs`](crates/transport-wasm/src/fetch.rs) &mdash; 567 lines
- [`lib.rs`](crates/transport-wasm/src/lib.rs) &mdash; 60 lines

</details>

<details>
<summary><code>crates/transport-wasm/tests/</code> &mdash; 3 file(s)</summary>

- [`fetch_contract.rs`](crates/transport-wasm/tests/fetch_contract.rs) &mdash; 376 lines
- [`parity_contract.rs`](crates/transport-wasm/tests/parity_contract.rs) &mdash; 530 lines
- [`wasm.rs`](crates/transport-wasm/tests/wasm.rs) &mdash; 9 lines

</details>

<details>
<summary><code>crates/transport-wasm/tests/wasm/</code> &mdash; 1 file(s)</summary>

- [`fetch_smoke.rs`](crates/transport-wasm/tests/wasm/fetch_smoke.rs) &mdash; 20 lines

</details>

<details>
<summary><code>crates/wasm/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](crates/wasm/Cargo.toml) &mdash; 104 lines
- [`README.md`](crates/wasm/README.md) &mdash; 186 lines

</details>

<details>
<summary><code>crates/wasm/npm/</code> &mdash; 11 file(s)</summary>

- [`.gitignore`](crates/wasm/npm/.gitignore) &mdash; 3 lines
- [`.npmignore`](crates/wasm/npm/.npmignore) &mdash; 6 lines
- [`flavours.json`](crates/wasm/npm/flavours.json) &mdash; 66 lines
- [`LICENSE`](crates/wasm/npm/LICENSE) &mdash; 1 lines
- [`package.json`](crates/wasm/npm/package.json) &mdash; 103 lines
- [`package.template.json`](crates/wasm/npm/package.template.json) &mdash; 46 lines
- [`pnpm-lock.yaml`](crates/wasm/npm/pnpm-lock.yaml) &mdash; 771 lines
- [`README.md`](crates/wasm/npm/README.md) &mdash; 266 lines
- [`tsconfig.facade.json`](crates/wasm/npm/tsconfig.facade.json) &mdash; 5 lines
- [`tsconfig.json`](crates/wasm/npm/tsconfig.json) &mdash; 24 lines
- [`vitest.config.ts`](crates/wasm/npm/vitest.config.ts) &mdash; 9 lines

</details>

<details>
<summary><code>crates/wasm/npm/scripts/</code> &mdash; 10 file(s)</summary>

- [`build.sh`](crates/wasm/npm/scripts/build.sh) &mdash; 157 lines
- [`compile-facade.sh`](crates/wasm/npm/scripts/compile-facade.sh) &mdash; 157 lines
- [`measure-wasm-size.mjs`](crates/wasm/npm/scripts/measure-wasm-size.mjs) &mdash; 159 lines
- [`pack-and-resolve-tarball.sh`](crates/wasm/npm/scripts/pack-and-resolve-tarball.sh) &mdash; 22 lines
- [`prepublish-guard.sh`](crates/wasm/npm/scripts/prepublish-guard.sh) &mdash; 25 lines
- [`render-package-json.mjs`](crates/wasm/npm/scripts/render-package-json.mjs) &mdash; 111 lines
- [`verify-exports.mjs`](crates/wasm/npm/scripts/verify-exports.mjs) &mdash; 114 lines
- [`verify-facade-denylist.mjs`](crates/wasm/npm/scripts/verify-facade-denylist.mjs) &mdash; 79 lines
- [`verify-no-raw-exports.mjs`](crates/wasm/npm/scripts/verify-no-raw-exports.mjs) &mdash; 47 lines
- [`verify-package-resolution.sh`](crates/wasm/npm/scripts/verify-package-resolution.sh) &mdash; 69 lines

</details>

<details>
<summary><code>crates/wasm/npm/src/</code> &mdash; 10 file(s)</summary>

- [`callbacks.ts`](crates/wasm/npm/src/callbacks.ts) &mdash; 95 lines
- [`cloudflare.ts`](crates/wasm/npm/src/cloudflare.ts) &mdash; 548 lines
- [`default.ts`](crates/wasm/npm/src/default.ts) &mdash; 657 lines
- [`envelope.ts`](crates/wasm/npm/src/envelope.ts) &mdash; 6 lines
- [`errors.ts`](crates/wasm/npm/src/errors.ts) &mdash; 252 lines
- [`index.ts`](crates/wasm/npm/src/index.ts) &mdash; 1 lines
- [`internal.ts`](crates/wasm/npm/src/internal.ts) &mdash; 156 lines
- [`options.ts`](crates/wasm/npm/src/options.ts) &mdash; 80 lines
- [`orderbook.ts`](crates/wasm/npm/src/orderbook.ts) &mdash; 350 lines
- [`signing.ts`](crates/wasm/npm/src/signing.ts) &mdash; 150 lines

</details>

<details>
<summary><code>crates/wasm/npm/src/raw/</code> &mdash; 4 file(s)</summary>

- [`cloudflare.ts`](crates/wasm/npm/src/raw/cloudflare.ts) &mdash; 33 lines
- [`default.ts`](crates/wasm/npm/src/raw/default.ts) &mdash; 34 lines
- [`orderbook.ts`](crates/wasm/npm/src/raw/orderbook.ts) &mdash; 26 lines
- [`signing.ts`](crates/wasm/npm/src/raw/signing.ts) &mdash; 19 lines

</details>

<details>
<summary><code>crates/wasm/npm/tests/</code> &mdash; 8 file(s)</summary>

- [`facade-cancellation.test.ts`](crates/wasm/npm/tests/facade-cancellation.test.ts) &mdash; 29 lines
- [`facade-default.test.ts`](crates/wasm/npm/tests/facade-default.test.ts) &mdash; 34 lines
- [`facade-error-normalization.test.ts`](crates/wasm/npm/tests/facade-error-normalization.test.ts) &mdash; 85 lines
- [`facade-error-shape.test.ts`](crates/wasm/npm/tests/facade-error-shape.test.ts) &mdash; 47 lines
- [`facade-orderbook.test.ts`](crates/wasm/npm/tests/facade-orderbook.test.ts) &mdash; 20 lines
- [`facade-resource-cleanup.test.ts`](crates/wasm/npm/tests/facade-resource-cleanup.test.ts) &mdash; 24 lines
- [`facade-signing.test.ts`](crates/wasm/npm/tests/facade-signing.test.ts) &mdash; 19 lines
- [`fixtures.ts`](crates/wasm/npm/tests/fixtures.ts) &mdash; 34 lines

</details>

<details>
<summary><code>crates/wasm/snapshots/facade/</code> &mdash; 5 file(s)</summary>

- [`.keep`](crates/wasm/snapshots/facade/.keep) &mdash; 1 lines
- [`cloudflare.d.ts`](crates/wasm/snapshots/facade/cloudflare.d.ts) &mdash; 69 lines
- [`default.d.ts`](crates/wasm/snapshots/facade/default.d.ts) &mdash; 83 lines
- [`orderbook.d.ts`](crates/wasm/snapshots/facade/orderbook.d.ts) &mdash; 50 lines
- [`signing.d.ts`](crates/wasm/snapshots/facade/signing.d.ts) &mdash; 23 lines

</details>

<details>
<summary><code>crates/wasm/snapshots/raw/</code> &mdash; 8 file(s)</summary>

- [`.keep`](crates/wasm/snapshots/raw/.keep) &mdash; 1 lines
- [`cloudflare-web.d.ts`](crates/wasm/snapshots/raw/cloudflare-web.d.ts) &mdash; 2,730 lines
- [`default-bundler.d.ts`](crates/wasm/snapshots/raw/default-bundler.d.ts) &mdash; 2,781 lines
- [`default-nodejs.d.ts`](crates/wasm/snapshots/raw/default-nodejs.d.ts) &mdash; 2,781 lines
- [`orderbook-bundler.d.ts`](crates/wasm/snapshots/raw/orderbook-bundler.d.ts) &mdash; 1,776 lines
- [`orderbook-nodejs.d.ts`](crates/wasm/snapshots/raw/orderbook-nodejs.d.ts) &mdash; 1,776 lines
- [`signing-bundler.d.ts`](crates/wasm/snapshots/raw/signing-bundler.d.ts) &mdash; 774 lines
- [`signing-nodejs.d.ts`](crates/wasm/snapshots/raw/signing-nodejs.d.ts) &mdash; 774 lines

</details>

<details>
<summary><code>crates/wasm/src/</code> &mdash; 1 file(s)</summary>

- [`lib.rs`](crates/wasm/src/lib.rs) &mdash; 39 lines

</details>

<details>
<summary><code>crates/wasm/src/exports/</code> &mdash; 15 file(s)</summary>

- [`callbacks.rs`](crates/wasm/src/exports/callbacks.rs) &mdash; 135 lines
- [`cancel.rs`](crates/wasm/src/exports/cancel.rs) &mdash; 243 lines
- [`chains.rs`](crates/wasm/src/exports/chains.rs) &mdash; 243 lines
- [`eip1271.rs`](crates/wasm/src/exports/eip1271.rs) &mdash; 204 lines
- [`envelope.rs`](crates/wasm/src/exports/envelope.rs) &mdash; 37 lines
- [`errors.rs`](crates/wasm/src/exports/errors.rs) &mdash; 761 lines
- [`events.rs`](crates/wasm/src/exports/events.rs) &mdash; 64 lines
- [`ipfs.rs`](crates/wasm/src/exports/ipfs.rs) &mdash; 255 lines
- [`mod.rs`](crates/wasm/src/exports/mod.rs) &mdash; 95 lines
- [`orderbook.rs`](crates/wasm/src/exports/orderbook.rs) &mdash; 642 lines
- [`registry.rs`](crates/wasm/src/exports/registry.rs) &mdash; 112 lines
- [`signing.rs`](crates/wasm/src/exports/signing.rs) &mdash; 746 lines
- [`subgraph.rs`](crates/wasm/src/exports/subgraph.rs) &mdash; 241 lines
- [`trading.rs`](crates/wasm/src/exports/trading.rs) &mdash; 743 lines
- [`transport.rs`](crates/wasm/src/exports/transport.rs) &mdash; 594 lines

</details>

<details>
<summary><code>crates/wasm/src/exports/dto/</code> &mdash; 12 file(s)</summary>

- [`app_data.rs`](crates/wasm/src/exports/dto/app_data.rs) &mdash; 104 lines
- [`contracts.rs`](crates/wasm/src/exports/dto/contracts.rs) &mdash; 105 lines
- [`core.rs`](crates/wasm/src/exports/dto/core.rs) &mdash; 144 lines
- [`events.rs`](crates/wasm/src/exports/dto/events.rs) &mdash; 298 lines
- [`mod.rs`](crates/wasm/src/exports/dto/mod.rs) &mdash; 91 lines
- [`order.rs`](crates/wasm/src/exports/dto/order.rs) &mdash; 239 lines
- [`orderbook.rs`](crates/wasm/src/exports/dto/orderbook.rs) &mdash; 376 lines
- [`quote.rs`](crates/wasm/src/exports/dto/quote.rs) &mdash; 267 lines
- [`signing.rs`](crates/wasm/src/exports/dto/signing.rs) &mdash; 205 lines
- [`subgraph.rs`](crates/wasm/src/exports/dto/subgraph.rs) &mdash; 19 lines
- [`trading.rs`](crates/wasm/src/exports/dto/trading.rs) &mdash; 289 lines
- [`transport.rs`](crates/wasm/src/exports/dto/transport.rs) &mdash; 317 lines

</details>

<details>
<summary><code>crates/wasm/src/helpers/</code> &mdash; 6 file(s)</summary>

- [`app_data.rs`](crates/wasm/src/helpers/app_data.rs) &mdash; 67 lines
- [`chains.rs`](crates/wasm/src/helpers/chains.rs) &mdash; 82 lines
- [`dto.rs`](crates/wasm/src/helpers/dto.rs) &mdash; 412 lines
- [`errors.rs`](crates/wasm/src/helpers/errors.rs) &mdash; 51 lines
- [`mod.rs`](crates/wasm/src/helpers/mod.rs) &mdash; 17 lines
- [`signing.rs`](crates/wasm/src/helpers/signing.rs) &mdash; 41 lines

</details>

<details>
<summary><code>crates/wasm/tests/</code> &mdash; 20 file(s)</summary>

- [`host_pure_helpers.rs`](crates/wasm/tests/host_pure_helpers.rs) &mdash; 295 lines
- [`no_ffi_helpers.rs`](crates/wasm/tests/no_ffi_helpers.rs) &mdash; 63 lines
- [`wasm_callback_contract.rs`](crates/wasm/tests/wasm_callback_contract.rs) &mdash; 391 lines
- [`wasm_callback_lifetime_contract.rs`](crates/wasm/tests/wasm_callback_lifetime_contract.rs) &mdash; 55 lines
- [`wasm_callback_transport_contract.rs`](crates/wasm/tests/wasm_callback_transport_contract.rs) &mdash; 135 lines
- [`wasm_cancellation_contract.rs`](crates/wasm/tests/wasm_cancellation_contract.rs) &mdash; 239 lines
- [`wasm_dto_parity_contract.rs`](crates/wasm/tests/wasm_dto_parity_contract.rs) &mdash; 108 lines
- [`wasm_eip1271_contract.rs`](crates/wasm/tests/wasm_eip1271_contract.rs) &mdash; 248 lines
- [`wasm_envelope_contract.rs`](crates/wasm/tests/wasm_envelope_contract.rs) &mdash; 33 lines
- [`wasm_error_abi_contract.rs`](crates/wasm/tests/wasm_error_abi_contract.rs) &mdash; 262 lines
- [`wasm_facade_snapshot_contract.rs`](crates/wasm/tests/wasm_facade_snapshot_contract.rs) &mdash; 154 lines
- [`wasm_fail_closed_contract.rs`](crates/wasm/tests/wasm_fail_closed_contract.rs) &mdash; 194 lines
- [`wasm_ipfs_contract.rs`](crates/wasm/tests/wasm_ipfs_contract.rs) &mdash; 181 lines
- [`wasm_redaction_contract.rs`](crates/wasm/tests/wasm_redaction_contract.rs) &mdash; 127 lines
- [`wasm_retry_runner_contract.rs`](crates/wasm/tests/wasm_retry_runner_contract.rs) &mdash; 69 lines
- [`wasm_snapshot_surface_contract.rs`](crates/wasm/tests/wasm_snapshot_surface_contract.rs) &mdash; 381 lines
- [`wasm_surface_contract.rs`](crates/wasm/tests/wasm_surface_contract.rs) &mdash; 229 lines
- [`wasm_telemetry_contract.rs`](crates/wasm/tests/wasm_telemetry_contract.rs) &mdash; 54 lines
- [`wasm_transport_policy_contract.rs`](crates/wasm/tests/wasm_transport_policy_contract.rs) &mdash; 320 lines
- [`wasm_workflow_coverage_contract.rs`](crates/wasm/tests/wasm_workflow_coverage_contract.rs) &mdash; 412 lines

</details>

<details>
<summary><code>crates/wasm/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](crates/wasm/tests/common/mod.rs) &mdash; 195 lines

</details>

<details>
<summary><code>docs/</code> &mdash; 19 file(s)</summary>

- [`alloy-doctrine.md`](docs/alloy-doctrine.md) &mdash; 315 lines
- [`alloy-major-release-runbook.md`](docs/alloy-major-release-runbook.md) &mdash; 63 lines
- [`architecture.md`](docs/architecture.md) &mdash; 438 lines
- [`browser-runtime-proof-posture.md`](docs/browser-runtime-proof-posture.md) &mdash; 117 lines
- [`code-of-conduct.md`](docs/code-of-conduct.md) &mdash; 71 lines
- [`deployments.md`](docs/deployments.md) &mdash; 102 lines
- [`examples.md`](docs/examples.md) &mdash; 112 lines
- [`getting-started.md`](docs/getting-started.md) &mdash; 824 lines
- [`integrations.md`](docs/integrations.md) &mdash; 406 lines
- [`msrv-policy.md`](docs/msrv-policy.md) &mdash; 39 lines
- [`observability.md`](docs/observability.md) &mdash; 408 lines
- [`parity.md`](docs/parity.md) &mdash; 474 lines
- [`performance.md`](docs/performance.md) &mdash; 271 lines
- [`principles.md`](docs/principles.md) &mdash; 245 lines
- [`publication-handoff.md`](docs/publication-handoff.md) &mdash; 118 lines
- [`README.md`](docs/README.md) &mdash; 123 lines
- [`release-checklist.md`](docs/release-checklist.md) &mdash; 485 lines
- [`transport.md`](docs/transport.md) &mdash; 447 lines
- [`verification.md`](docs/verification.md) &mdash; 334 lines

</details>

<details>
<summary><code>docs/adr/</code> &mdash; 70 file(s)</summary>

- [`0000-template.md`](docs/adr/0000-template.md) &mdash; 44 lines
- [`0001-multi-crate-sdk-family-with-thin-facade.md`](docs/adr/0001-multi-crate-sdk-family-with-thin-facade.md) &mdash; 62 lines
- [`0002-dedicated-trading-orchestration-crate.md`](docs/adr/0002-dedicated-trading-orchestration-crate.md) &mdash; 47 lines
- [`0003-separate-read-only-subgraph-crate.md`](docs/adr/0003-separate-read-only-subgraph-crate.md) &mdash; 65 lines
- [`0004-feature-gated-browser-wallet-sidecar.md`](docs/adr/0004-feature-gated-browser-wallet-sidecar.md) &mdash; 47 lines
- [`0005-boundary-specific-runtime-contracts-and-strong-domain-types.md`](docs/adr/0005-boundary-specific-runtime-contracts-and-strong-domain-types.md) &mdash; 86 lines
- [`0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md`](docs/adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md) &mdash; 55 lines
- [`0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md`](docs/adr/0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md) &mdash; 67 lines
- [`0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md`](docs/adr/0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md) &mdash; 48 lines
- [`0009-wasm-verification-consoles-hybrid-extensibility-and-two-tier-proof.md`](docs/adr/0009-wasm-verification-consoles-hybrid-extensibility-and-two-tier-proof.md) &mdash; 80 lines
- [`0010-runtime-neutral-async-and-transport-posture.md`](docs/adr/0010-runtime-neutral-async-and-transport-posture.md) &mdash; 92 lines
- [`0011-typed-amount-boundary-and-typestate-ready-state-construction.md`](docs/adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md) &mdash; 480 lines
- [`0012-alloy-sol-bindings-and-registry-authority.md`](docs/adr/0012-alloy-sol-bindings-and-registry-authority.md) &mdash; 153 lines
- [`0013-http-transport-injection-and-typestate-builders.md`](docs/adr/0013-http-transport-injection-and-typestate-builders.md) &mdash; 83 lines
- [`0014-eip1271-verification-cache.md`](docs/adr/0014-eip1271-verification-cache.md) &mdash; 219 lines
- [`0015-client-side-order-bounds-validator.md`](docs/adr/0015-client-side-order-bounds-validator.md) &mdash; 186 lines
- [`0016-split-sell-and-buy-token-balance-enums.md`](docs/adr/0016-split-sell-and-buy-token-balance-enums.md) &mdash; 91 lines
- [`0017-typed-orderbook-rejection-parser.md`](docs/adr/0017-typed-orderbook-rejection-parser.md) &mdash; 183 lines
- [`0018-typed-app-data-merge.md`](docs/adr/0018-typed-app-data-merge.md) &mdash; 142 lines
- [`0019-http-transport-sole-dispatch.md`](docs/adr/0019-http-transport-sole-dispatch.md) &mdash; 71 lines
- [`0020-ethflow-owner-threading.md`](docs/adr/0020-ethflow-owner-threading.md) &mdash; 181 lines
- [`0021-orderbook-total-fee-policy.md`](docs/adr/0021-orderbook-total-fee-policy.md) &mdash; 121 lines
- [`0022-ecdsa-signature-v-normalization.md`](docs/adr/0022-ecdsa-signature-v-normalization.md) &mdash; 180 lines
- [`0023-legacy-compatibility-shim-removal.md`](docs/adr/0023-legacy-compatibility-shim-removal.md) &mdash; 101 lines
- [`0024-asyncprovider-asyncsigningprovider-capability-split.md`](docs/adr/0024-asyncprovider-asyncsigningprovider-capability-split.md) &mdash; 59 lines
- [`0025-workspace-url-redaction-convention.md`](docs/adr/0025-workspace-url-redaction-convention.md) &mdash; 56 lines
- [`0026-alloy-major-release-absorption-plan.md`](docs/adr/0026-alloy-major-release-absorption-plan.md) &mdash; 114 lines
- [`0027-post-quantum-signing-absorption-plan.md`](docs/adr/0027-post-quantum-signing-absorption-plan.md) &mdash; 89 lines
- [`0028-account-abstraction-integration-plan.md`](docs/adr/0028-account-abstraction-integration-plan.md) &mdash; 93 lines
- [`0029-trait-evolution-extension-traits.md`](docs/adr/0029-trait-evolution-extension-traits.md) &mdash; 96 lines
- [`0030-workspace-locked-versioning-tag-baseline.md`](docs/adr/0030-workspace-locked-versioning-tag-baseline.md) &mdash; 74 lines
- [`0031-wire-dto-openapi-driven-with-order-auction-order-split.md`](docs/adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md) &mdash; 116 lines
- [`0032-deployment-authority-machine-readable-provenance.md`](docs/adr/0032-deployment-authority-machine-readable-provenance.md) &mdash; 183 lines
- [`0033-minimum-viable-panic-surface.md`](docs/adr/0033-minimum-viable-panic-surface.md) &mdash; 71 lines
- [`0034-interaction-encoder-target-policy.md`](docs/adr/0034-interaction-encoder-target-policy.md) &mdash; 95 lines
- [`0035-alloy-provider-adapter.md`](docs/adr/0035-alloy-provider-adapter.md) &mdash; 139 lines
- [`0036-alloy-signer-adapter.md`](docs/adr/0036-alloy-signer-adapter.md) &mdash; 109 lines
- [`0037-alloy-umbrella-adapter.md`](docs/adr/0037-alloy-umbrella-adapter.md) &mdash; 154 lines
- [`0038-transaction-lifecycle-types.md`](docs/adr/0038-transaction-lifecycle-types.md) &mdash; 97 lines
- [`0039-typescript-callable-wasm-sdk-surface.md`](docs/adr/0039-typescript-callable-wasm-sdk-surface.md) &mdash; 155 lines
- [`0040-wallet-provider-callback-boundary-for-js-consumers.md`](docs/adr/0040-wallet-provider-callback-boundary-for-js-consumers.md) &mdash; 72 lines
- [`0041-transport-policy-l3-layering.md`](docs/adr/0041-transport-policy-l3-layering.md) &mdash; 113 lines
- [`0042-pure-helpers-extraction.md`](docs/adr/0042-pure-helpers-extraction.md) &mdash; 79 lines
- [`0043-callback-registry-internalization.md`](docs/adr/0043-callback-registry-internalization.md) &mdash; 55 lines
- [`0044-bundle-size-profile-and-flavor-builds.md`](docs/adr/0044-bundle-size-profile-and-flavor-builds.md) &mdash; 98 lines
- [`0045-async-signer-trait-narrowing.md`](docs/adr/0045-async-signer-trait-narrowing.md) &mdash; 54 lines
- [`0046-transport-policy-js-exposure.md`](docs/adr/0046-transport-policy-js-exposure.md) &mdash; 53 lines
- [`0047-typescript-facade-architecture.md`](docs/adr/0047-typescript-facade-architecture.md) &mdash; 60 lines
- [`0048-composable-conditional-order-framework.md`](docs/adr/0048-composable-conditional-order-framework.md) &mdash; 198 lines
- [`0049-cow-shed-account-abstraction-proxy.md`](docs/adr/0049-cow-shed-account-abstraction-proxy.md) &mdash; 264 lines
- [`0050-eip1271-signature-blob-encoding.md`](docs/adr/0050-eip1271-signature-blob-encoding.md) &mdash; 174 lines
- [`0051-signing-owned-eip1271-signature-provider-trait.md`](docs/adr/0051-signing-owned-eip1271-signature-provider-trait.md) &mdash; 135 lines
- [`0052-alloy-primitives-canonical-primitive-layer.md`](docs/adr/0052-alloy-primitives-canonical-primitive-layer.md) &mdash; 422 lines
- [`0053-typed-signer-rejection-classification.md`](docs/adr/0053-typed-signer-rejection-classification.md) &mdash; 156 lines
- [`0054-onchain-order-event-decoding-is-fail-closed.md`](docs/adr/0054-onchain-order-event-decoding-is-fail-closed.md) &mdash; 83 lines
- [`0055-bounded-response-reads.md`](docs/adr/0055-bounded-response-reads.md) &mdash; 95 lines
- [`0056-settlement-event-decoding-is-fail-closed.md`](docs/adr/0056-settlement-event-decoding-is-fail-closed.md) &mdash; 75 lines
- [`0057-log-provider-capability-trait.md`](docs/adr/0057-log-provider-capability-trait.md) &mdash; 133 lines
- [`0058-typed-quote-request-response-surface.md`](docs/adr/0058-typed-quote-request-response-surface.md) &mdash; 145 lines
- [`0059-hash-concrete-orderdata-directly.md`](docs/adr/0059-hash-concrete-orderdata-directly.md) &mdash; 66 lines
- [`0060-uniform-error-classification.md`](docs/adr/0060-uniform-error-classification.md) &mdash; 176 lines
- [`0061-wasm-abi-receiver-pay-to-owner.md`](docs/adr/0061-wasm-abi-receiver-pay-to-owner.md) &mdash; 79 lines
- [`0062-internal-shared-test-support-crate.md`](docs/adr/0062-internal-shared-test-support-crate.md) &mdash; 61 lines
- [`0063-published-consumer-test-doubles-crate.md`](docs/adr/0063-published-consumer-test-doubles-crate.md) &mdash; 74 lines
- [`0064-app-data-typed-validation.md`](docs/adr/0064-app-data-typed-validation.md) &mdash; 90 lines
- [`0065-canonical-browser-wallet-example.md`](docs/adr/0065-canonical-browser-wallet-example.md) &mdash; 66 lines
- [`0066-trading-slippage-and-suggestion-policy.md`](docs/adr/0066-trading-slippage-and-suggestion-policy.md) &mdash; 59 lines
- [`0067-idiomatic-accessor-naming.md`](docs/adr/0067-idiomatic-accessor-naming.md) &mdash; 70 lines
- [`0068-payload-only-typed-data-signing.md`](docs/adr/0068-payload-only-typed-data-signing.md) &mdash; 78 lines
- [`README.md`](docs/adr/README.md) &mdash; 197 lines

</details>

<details>
<summary><code>docs/audit/</code> &mdash; 63 file(s)</summary>

- [`alloy-provider-adapter-audit.md`](docs/audit/alloy-provider-adapter-audit.md) &mdash; 174 lines
- [`alloy-signer-adapter-audit.md`](docs/audit/alloy-signer-adapter-audit.md) &mdash; 148 lines
- [`alloy-umbrella-adapter-audit.md`](docs/audit/alloy-umbrella-adapter-audit.md) &mdash; 101 lines
- [`bounded-response-reads-audit.md`](docs/audit/bounded-response-reads-audit.md) &mdash; 126 lines
- [`browser-wallet-alloy-dependency-audit.md`](docs/audit/browser-wallet-alloy-dependency-audit.md) &mdash; 136 lines
- [`browser-wallet-chain-coherence-audit.md`](docs/audit/browser-wallet-chain-coherence-audit.md) &mdash; 102 lines
- [`browser-wallet-trust-posture-audit.md`](docs/audit/browser-wallet-trust-posture-audit.md) &mdash; 116 lines
- [`cid-dependency-audit.md`](docs/audit/cid-dependency-audit.md) &mdash; 134 lines
- [`contract-bindings-parity-audit.md`](docs/audit/contract-bindings-parity-audit.md) &mdash; 520 lines
- [`cooperative-cancellation-contract-audit.md`](docs/audit/cooperative-cancellation-contract-audit.md) &mdash; 187 lines
- [`cow-sdk-wasm-comparative-benchmark-validation-note.md`](docs/audit/cow-sdk-wasm-comparative-benchmark-validation-note.md) &mdash; 574 lines
- [`cow-shed-app-data-integration-audit.md`](docs/audit/cow-shed-app-data-integration-audit.md) &mdash; 112 lines
- [`cow-shed-contract-bindings-audit.md`](docs/audit/cow-shed-contract-bindings-audit.md) &mdash; 239 lines
- [`credential-surface-audit.md`](docs/audit/credential-surface-audit.md) &mdash; 212 lines
- [`credential-surface-contract-hygiene-audit.md`](docs/audit/credential-surface-contract-hygiene-audit.md) &mdash; 169 lines
- [`dependency-gate-audit.md`](docs/audit/dependency-gate-audit.md) &mdash; 359 lines
- [`deployment-registry-audit.md`](docs/audit/deployment-registry-audit.md) &mdash; 122 lines
- [`ecdsa-signature-normalization-audit.md`](docs/audit/ecdsa-signature-normalization-audit.md) &mdash; 224 lines
- [`eip1271-verification-cache-audit.md`](docs/audit/eip1271-verification-cache-audit.md) &mdash; 199 lines
- [`error-classification-audit.md`](docs/audit/error-classification-audit.md) &mdash; 143 lines
- [`fuzz-coverage-audit.md`](docs/audit/fuzz-coverage-audit.md) &mdash; 292 lines
- [`http-transport-contract-audit.md`](docs/audit/http-transport-contract-audit.md) &mdash; 248 lines
- [`lens-chain-evidence-audit.md`](docs/audit/lens-chain-evidence-audit.md) &mdash; 27 lines
- [`log-provider-capability-audit.md`](docs/audit/log-provider-capability-audit.md) &mdash; 127 lines
- [`onchain-order-log-decoding-audit.md`](docs/audit/onchain-order-log-decoding-audit.md) &mdash; 78 lines
- [`panic-free-public-surface-audit.md`](docs/audit/panic-free-public-surface-audit.md) &mdash; 148 lines
- [`partner-api-routing-audit.md`](docs/audit/partner-api-routing-audit.md) &mdash; 81 lines
- [`public-api-naming-convention-audit.md`](docs/audit/public-api-naming-convention-audit.md) &mdash; 68 lines
- [`quote-request-app-data-fix-review.md`](docs/audit/quote-request-app-data-fix-review.md) &mdash; 70 lines
- [`quote-response-surface-audit.md`](docs/audit/quote-response-surface-audit.md) &mdash; 143 lines
- [`README.md`](docs/audit/README.md) &mdash; 190 lines
- [`settlement-event-log-decoding-audit.md`](docs/audit/settlement-event-log-decoding-audit.md) &mdash; 76 lines
- [`shared-logic-reviewability-audit.md`](docs/audit/shared-logic-reviewability-audit.md) &mdash; 166 lines
- [`signer-error-classification-audit.md`](docs/audit/signer-error-classification-audit.md) &mdash; 118 lines
- [`source-lock-provenance-audit.md`](docs/audit/source-lock-provenance-audit.md) &mdash; 259 lines
- [`subgraph-error-display-audit.md`](docs/audit/subgraph-error-display-audit.md) &mdash; 162 lines
- [`trade-parameter-lifecycle-audit.md`](docs/audit/trade-parameter-lifecycle-audit.md) &mdash; 145 lines
- [`trading-app-data-merge-audit.md`](docs/audit/trading-app-data-merge-audit.md) &mdash; 186 lines
- [`trading-ethflow-owner-identity-audit.md`](docs/audit/trading-ethflow-owner-identity-audit.md) &mdash; 153 lines
- [`trading-order-bounds-validator-audit.md`](docs/audit/trading-order-bounds-validator-audit.md) &mdash; 238 lines
- [`trading-order-construction-integrity-audit.md`](docs/audit/trading-order-construction-integrity-audit.md) &mdash; 130 lines
- [`trading-orderbook-context-audit.md`](docs/audit/trading-orderbook-context-audit.md) &mdash; 85 lines
- [`trading-quote-orderbook-binding-audit.md`](docs/audit/trading-quote-orderbook-binding-audit.md) &mdash; 78 lines
- [`trading-sdk-runtime-prerequisites-audit.md`](docs/audit/trading-sdk-runtime-prerequisites-audit.md) &mdash; 138 lines
- [`transaction-receipt-shape-audit.md`](docs/audit/transaction-receipt-shape-audit.md) &mdash; 99 lines
- [`transport-policy-coverage-audit.md`](docs/audit/transport-policy-coverage-audit.md) &mdash; 243 lines
- [`typestate-builder-contract-audit.md`](docs/audit/typestate-builder-contract-audit.md) &mdash; 260 lines
- [`unsafe-code-policy-audit.md`](docs/audit/unsafe-code-policy-audit.md) &mdash; 83 lines
- [`url-credential-redaction-audit.md`](docs/audit/url-credential-redaction-audit.md) &mdash; 159 lines
- [`wasm-browser-runner-determinism-audit.md`](docs/audit/wasm-browser-runner-determinism-audit.md) &mdash; 90 lines
- [`wasm-callback-shape-design-audit.md`](docs/audit/wasm-callback-shape-design-audit.md) &mdash; 106 lines
- [`wasm-capability-coverage-audit.md`](docs/audit/wasm-capability-coverage-audit.md) &mdash; 359 lines
- [`wasm-component-model-future-prep-audit.md`](docs/audit/wasm-component-model-future-prep-audit.md) &mdash; 84 lines
- [`wasm-eip1271-parity-audit.md`](docs/audit/wasm-eip1271-parity-audit.md) &mdash; 89 lines
- [`wasm-facade-architecture-audit.md`](docs/audit/wasm-facade-architecture-audit.md) &mdash; 98 lines
- [`wasm-performance-budget-audit.md`](docs/audit/wasm-performance-budget-audit.md) &mdash; 118 lines
- [`wasm-public-api-stability-audit.md`](docs/audit/wasm-public-api-stability-audit.md) &mdash; 102 lines
- [`wasm-schema-versioning-policy-audit.md`](docs/audit/wasm-schema-versioning-policy-audit.md) &mdash; 78 lines
- [`wasm-surface-audit.md`](docs/audit/wasm-surface-audit.md) &mdash; 132 lines
- [`wasm-type-generation-audit.md`](docs/audit/wasm-type-generation-audit.md) &mdash; 130 lines
- [`wasm-unsupported-target-audit.md`](docs/audit/wasm-unsupported-target-audit.md) &mdash; 57 lines
- [`wire-dto-coverage-audit.md`](docs/audit/wire-dto-coverage-audit.md) &mdash; 189 lines
- [`workflow-security-audit.md`](docs/audit/workflow-security-audit.md) &mdash; 146 lines

</details>

<details>
<summary><code>docs/providers/</code> &mdash; 2 file(s)</summary>

- [`adapting-alloy.md`](docs/providers/adapting-alloy.md) &mdash; 202 lines
- [`README.md`](docs/providers/README.md) &mdash; 75 lines

</details>

<details>
<summary><code>e2e/wasm-typescript/</code> &mdash; 8 file(s)</summary>

- [`index.html`](e2e/wasm-typescript/index.html) &mdash; 12 lines
- [`package.json`](e2e/wasm-typescript/package.json) &mdash; 30 lines
- [`playwright.config.ts`](e2e/wasm-typescript/playwright.config.ts) &mdash; 16 lines
- [`pnpm-lock.yaml`](e2e/wasm-typescript/pnpm-lock.yaml) &mdash; 1,574 lines
- [`pnpm-workspace.yaml`](e2e/wasm-typescript/pnpm-workspace.yaml) &mdash; 3 lines
- [`tsconfig.json`](e2e/wasm-typescript/tsconfig.json) &mdash; 14 lines
- [`vite.config.ts`](e2e/wasm-typescript/vite.config.ts) &mdash; 32 lines
- [`vitest.config.ts`](e2e/wasm-typescript/vitest.config.ts) &mdash; 11 lines

</details>

<details>
<summary><code>e2e/wasm-typescript-cf/</code> &mdash; 7 file(s)</summary>

- [`package.json`](e2e/wasm-typescript-cf/package.json) &mdash; 27 lines
- [`pnpm-lock.yaml`](e2e/wasm-typescript-cf/pnpm-lock.yaml) &mdash; 1,675 lines
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

- [`index.ts`](e2e/wasm-typescript/src/index.ts) &mdash; 53 lines

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

- [`README.md`](examples/README.md) &mdash; 49 lines

</details>

<details>
<summary><code>examples/native/</code> &mdash; 2 file(s)</summary>

- [`Cargo.toml`](examples/native/Cargo.toml) &mdash; 137 lines
- [`README.md`](examples/native/README.md) &mdash; 154 lines

</details>

<details>
<summary><code>examples/native/scenarios/</code> &mdash; 27 file(s)</summary>

- [`alloy_custom_traits.rs`](examples/native/scenarios/alloy_custom_traits.rs) &mdash; 184 lines
- [`alloy_provider.rs`](examples/native/scenarios/alloy_provider.rs) &mdash; 42 lines
- [`alloy_quickstart.rs`](examples/native/scenarios/alloy_quickstart.rs) &mdash; 48 lines
- [`alloy_signer.rs`](examples/native/scenarios/alloy_signer.rs) &mdash; 75 lines
- [`alloy_trading_full_flow.rs`](examples/native/scenarios/alloy_trading_full_flow.rs) &mdash; 104 lines
- [`app_data.rs`](examples/native/scenarios/app_data.rs) &mdash; 48 lines
- [`cancel_in_flight.rs`](examples/native/scenarios/cancel_in_flight.rs) &mdash; 88 lines
- [`eip1271_signer.rs`](examples/native/scenarios/eip1271_signer.rs) &mdash; 71 lines
- [`error_classification.rs`](examples/native/scenarios/error_classification.rs) &mdash; 283 lines
- [`ethflow_checker.rs`](examples/native/scenarios/ethflow_checker.rs) &mdash; 105 lines
- [`ethflow.rs`](examples/native/scenarios/ethflow.rs) &mdash; 107 lines
- [`facade_surface.rs`](examples/native/scenarios/facade_surface.rs) &mdash; 41 lines
- [`limit_order.rs`](examples/native/scenarios/limit_order.rs) &mdash; 76 lines
- [`onchain_actions.rs`](examples/native/scenarios/onchain_actions.rs) &mdash; 162 lines
- [`order_history.rs`](examples/native/scenarios/order_history.rs) &mdash; 106 lines
- [`order_lifecycle.rs`](examples/native/scenarios/order_lifecycle.rs) &mdash; 59 lines
- [`orderbook_live.rs`](examples/native/scenarios/orderbook_live.rs) &mdash; 63 lines
- [`orderbook_transport.rs`](examples/native/scenarios/orderbook_transport.rs) &mdash; 124 lines
- [`quote.rs`](examples/native/scenarios/quote.rs) &mdash; 66 lines
- [`receipt_lifecycle.rs`](examples/native/scenarios/receipt_lifecycle.rs) &mdash; 82 lines
- [`sign_order.rs`](examples/native/scenarios/sign_order.rs) &mdash; 64 lines
- [`slippage_suggester.rs`](examples/native/scenarios/slippage_suggester.rs) &mdash; 71 lines
- [`subgraph_live.rs`](examples/native/scenarios/subgraph_live.rs) &mdash; 48 lines
- [`subgraph_query.rs`](examples/native/scenarios/subgraph_query.rs) &mdash; 177 lines
- [`swap_quickstart.rs`](examples/native/scenarios/swap_quickstart.rs) &mdash; 46 lines
- [`trading_full_cycle.rs`](examples/native/scenarios/trading_full_cycle.rs) &mdash; 107 lines
- [`transaction_lifecycle.rs`](examples/native/scenarios/transaction_lifecycle.rs) &mdash; 76 lines

</details>

<details>
<summary><code>examples/native/src/</code> &mdash; 2 file(s)</summary>

- [`lib.rs`](examples/native/src/lib.rs) &mdash; 18 lines
- [`support.rs`](examples/native/src/support.rs) &mdash; 358 lines

</details>

<details>
<summary><code>examples/native/tests/</code> &mdash; 1 file(s)</summary>

- [`scenario_contract.rs`](examples/native/tests/scenario_contract.rs) &mdash; 206 lines

</details>

<details>
<summary><code>examples/wasm/cow-gateway-cloudflare/</code> &mdash; 8 file(s)</summary>

- [`.gitignore`](examples/wasm/cow-gateway-cloudflare/.gitignore) &mdash; 5 lines
- [`package.json`](examples/wasm/cow-gateway-cloudflare/package.json) &mdash; 26 lines
- [`pnpm-lock.yaml`](examples/wasm/cow-gateway-cloudflare/pnpm-lock.yaml) &mdash; 1,946 lines
- [`pnpm-workspace.yaml`](examples/wasm/cow-gateway-cloudflare/pnpm-workspace.yaml) &mdash; 4 lines
- [`README.md`](examples/wasm/cow-gateway-cloudflare/README.md) &mdash; 102 lines
- [`tsconfig.json`](examples/wasm/cow-gateway-cloudflare/tsconfig.json) &mdash; 14 lines
- [`vitest.config.ts`](examples/wasm/cow-gateway-cloudflare/vitest.config.ts) &mdash; 15 lines
- [`wrangler.toml`](examples/wasm/cow-gateway-cloudflare/wrangler.toml) &mdash; 9 lines

</details>

<details>
<summary><code>examples/wasm/cow-gateway-cloudflare/scripts/</code> &mdash; 1 file(s)</summary>

- [`build.mjs`](examples/wasm/cow-gateway-cloudflare/scripts/build.mjs) &mdash; 84 lines

</details>

<details>
<summary><code>examples/wasm/cow-gateway-cloudflare/src/</code> &mdash; 4 file(s)</summary>

- [`vite-env.d.ts`](examples/wasm/cow-gateway-cloudflare/src/vite-env.d.ts) &mdash; 4 lines
- [`wasm.d.ts`](examples/wasm/cow-gateway-cloudflare/src/wasm.d.ts) &mdash; 4 lines
- [`worker-exports.d.ts`](examples/wasm/cow-gateway-cloudflare/src/worker-exports.d.ts) &mdash; 16 lines
- [`worker.ts`](examples/wasm/cow-gateway-cloudflare/src/worker.ts) &mdash; 185 lines

</details>

<details>
<summary><code>examples/wasm/cow-gateway-cloudflare/tests/</code> &mdash; 3 file(s)</summary>

- [`forbidden-instantiation.spec.ts`](examples/wasm/cow-gateway-cloudflare/tests/forbidden-instantiation.spec.ts) &mdash; 17 lines
- [`transport.spec.ts`](examples/wasm/cow-gateway-cloudflare/tests/transport.spec.ts) &mdash; 34 lines
- [`worker.spec.ts`](examples/wasm/cow-gateway-cloudflare/tests/worker.spec.ts) &mdash; 61 lines

</details>

<details>
<summary><code>examples/wasm/cow-signer-node/</code> &mdash; 5 file(s)</summary>

- [`.gitignore`](examples/wasm/cow-signer-node/.gitignore) &mdash; 2 lines
- [`package.json`](examples/wasm/cow-signer-node/package.json) &mdash; 22 lines
- [`pnpm-lock.yaml`](examples/wasm/cow-signer-node/pnpm-lock.yaml) &mdash; 932 lines
- [`README.md`](examples/wasm/cow-signer-node/README.md) &mdash; 81 lines
- [`tsconfig.json`](examples/wasm/cow-signer-node/tsconfig.json) &mdash; 13 lines

</details>

<details>
<summary><code>examples/wasm/cow-signer-node/src/</code> &mdash; 2 file(s)</summary>

- [`index.test.ts`](examples/wasm/cow-signer-node/src/index.test.ts) &mdash; 48 lines
- [`index.ts`](examples/wasm/cow-signer-node/src/index.ts) &mdash; 118 lines

</details>

<details>
<summary><code>examples/wasm/cow-trader-dioxus/</code> &mdash; 5 file(s)</summary>

- [`.gitignore`](examples/wasm/cow-trader-dioxus/.gitignore) &mdash; 2 lines
- [`Cargo.lock`](examples/wasm/cow-trader-dioxus/Cargo.lock) &mdash; 4,849 lines
- [`Cargo.toml`](examples/wasm/cow-trader-dioxus/Cargo.toml) &mdash; 25 lines
- [`Dioxus.toml`](examples/wasm/cow-trader-dioxus/Dioxus.toml) &mdash; 5 lines
- [`README.md`](examples/wasm/cow-trader-dioxus/README.md) &mdash; 110 lines

</details>

<details>
<summary><code>examples/wasm/cow-trader-dioxus/src/</code> &mdash; 1 file(s)</summary>

- [`main.rs`](examples/wasm/cow-trader-dioxus/src/main.rs) &mdash; 474 lines

</details>

<details>
<summary><code>fuzz/</code> &mdash; 3 file(s)</summary>

- [`Cargo.lock`](fuzz/Cargo.lock) &mdash; 3,805 lines
- [`Cargo.toml`](fuzz/Cargo.toml) &mdash; 346 lines
- [`README.md`](fuzz/README.md) &mdash; 182 lines

</details>

<details>
<summary><code>fuzz/fuzz_targets/</code> &mdash; 45 file(s)</summary>

- [`fuzz_amount_parse_units.rs`](fuzz/fuzz_targets/fuzz_amount_parse_units.rs) &mdash; 62 lines
- [`fuzz_amount_parse.rs`](fuzz/fuzz_targets/fuzz_amount_parse.rs) &mdash; 75 lines
- [`fuzz_app_data_cid_roundtrip.rs`](fuzz/fuzz_targets/fuzz_app_data_cid_roundtrip.rs) &mdash; 91 lines
- [`fuzz_app_data_merge.rs`](fuzz/fuzz_targets/fuzz_app_data_merge.rs) &mdash; 309 lines
- [`fuzz_app_data_params_from_doc.rs`](fuzz/fuzz_targets/fuzz_app_data_params_from_doc.rs) &mdash; 362 lines
- [`fuzz_app_data_size_limit.rs`](fuzz/fuzz_targets/fuzz_app_data_size_limit.rs) &mdash; 158 lines
- [`fuzz_calculate_total_fee.rs`](fuzz/fuzz_targets/fuzz_calculate_total_fee.rs) &mdash; 96 lines
- [`fuzz_cid_to_app_data_hex.rs`](fuzz/fuzz_targets/fuzz_cid_to_app_data_hex.rs) &mdash; 90 lines
- [`fuzz_contract_call_serde.rs`](fuzz/fuzz_targets/fuzz_contract_call_serde.rs) &mdash; 63 lines
- [`fuzz_core_identity_validators.rs`](fuzz/fuzz_targets/fuzz_core_identity_validators.rs) &mdash; 195 lines
- [`fuzz_decode_magic_value_response.rs`](fuzz/fuzz_targets/fuzz_decode_magic_value_response.rs) &mdash; 232 lines
- [`fuzz_decoded_body_canonical_status_text.rs`](fuzz/fuzz_targets/fuzz_decoded_body_canonical_status_text.rs) &mdash; 243 lines
- [`fuzz_ecdsa_v_normalization.rs`](fuzz/fuzz_targets/fuzz_ecdsa_v_normalization.rs) &mdash; 54 lines
- [`fuzz_eip1271_signature_data_codec.rs`](fuzz/fuzz_targets/fuzz_eip1271_signature_data_codec.rs) &mdash; 56 lines
- [`fuzz_eth_flow_event_log_decode.rs`](fuzz/fuzz_targets/fuzz_eth_flow_event_log_decode.rs) &mdash; 52 lines
- [`fuzz_ethflow_create_order_encode.rs`](fuzz/fuzz_targets/fuzz_ethflow_create_order_encode.rs) &mdash; 115 lines
- [`fuzz_flashloan_hints.rs`](fuzz/fuzz_targets/fuzz_flashloan_hints.rs) &mdash; 111 lines
- [`fuzz_hash_order_cancellations.rs`](fuzz/fuzz_targets/fuzz_hash_order_cancellations.rs) &mdash; 162 lines
- [`fuzz_hook_list_deserialize.rs`](fuzz/fuzz_targets/fuzz_hook_list_deserialize.rs) &mdash; 96 lines
- [`fuzz_jitter_delay_for_attempt.rs`](fuzz/fuzz_targets/fuzz_jitter_delay_for_attempt.rs) &mdash; 115 lines
- [`fuzz_onchain_order_log_decode.rs`](fuzz/fuzz_targets/fuzz_onchain_order_log_decode.rs) &mdash; 61 lines
- [`fuzz_order_bounds_validator.rs`](fuzz/fuzz_targets/fuzz_order_bounds_validator.rs) &mdash; 272 lines
- [`fuzz_order_signature_classify.rs`](fuzz/fuzz_targets/fuzz_order_signature_classify.rs) &mdash; 83 lines
- [`fuzz_order_uid_pack_unpack.rs`](fuzz/fuzz_targets/fuzz_order_uid_pack_unpack.rs) &mdash; 57 lines
- [`fuzz_orderbook_rejection_code.rs`](fuzz/fuzz_targets/fuzz_orderbook_rejection_code.rs) &mdash; 87 lines
- [`fuzz_orderbook_rejection_decode.rs`](fuzz/fuzz_targets/fuzz_orderbook_rejection_decode.rs) &mdash; 52 lines
- [`fuzz_parse_retry_after.rs`](fuzz/fuzz_targets/fuzz_parse_retry_after.rs) &mdash; 51 lines
- [`fuzz_partner_fee_from_value.rs`](fuzz/fuzz_targets/fuzz_partner_fee_from_value.rs) &mdash; 78 lines
- [`fuzz_recover_ecdsa_address.rs`](fuzz/fuzz_targets/fuzz_recover_ecdsa_address.rs) &mdash; 88 lines
- [`fuzz_recoverable_signature_differential.rs`](fuzz/fuzz_targets/fuzz_recoverable_signature_differential.rs) &mdash; 92 lines
- [`fuzz_recoverable_signature_parse_hex.rs`](fuzz/fuzz_targets/fuzz_recoverable_signature_parse_hex.rs) &mdash; 61 lines
- [`fuzz_redact_response_body.rs`](fuzz/fuzz_targets/fuzz_redact_response_body.rs) &mdash; 84 lines
- [`fuzz_retry_policy_delay.rs`](fuzz/fuzz_targets/fuzz_retry_policy_delay.rs) &mdash; 153 lines
- [`fuzz_rpc_error_payload_serde.rs`](fuzz/fuzz_targets/fuzz_rpc_error_payload_serde.rs) &mdash; 71 lines
- [`fuzz_schema_version_is_semver.rs`](fuzz/fuzz_targets/fuzz_schema_version_is_semver.rs) &mdash; 92 lines
- [`fuzz_settlement_event_log_decode.rs`](fuzz/fuzz_targets/fuzz_settlement_event_log_decode.rs) &mdash; 54 lines
- [`fuzz_signing_domain_separator.rs`](fuzz/fuzz_targets/fuzz_signing_domain_separator.rs) &mdash; 126 lines
- [`fuzz_slippage_amounts.rs`](fuzz/fuzz_targets/fuzz_slippage_amounts.rs) &mdash; 160 lines
- [`fuzz_slippage_policy_helpers.rs`](fuzz/fuzz_targets/fuzz_slippage_policy_helpers.rs) &mdash; 182 lines
- [`fuzz_stringify_deterministic.rs`](fuzz/fuzz_targets/fuzz_stringify_deterministic.rs) &mdash; 73 lines
- [`fuzz_subgraph_graphql_error_decode.rs`](fuzz/fuzz_targets/fuzz_subgraph_graphql_error_decode.rs) &mdash; 93 lines
- [`fuzz_transaction_request_serde.rs`](fuzz/fuzz_targets/fuzz_transaction_request_serde.rs) &mdash; 62 lines
- [`fuzz_transport_error_classify.rs`](fuzz/fuzz_targets/fuzz_transport_error_classify.rs) &mdash; 282 lines
- [`fuzz_typed_data_digest.rs`](fuzz/fuzz_targets/fuzz_typed_data_digest.rs) &mdash; 143 lines
- [`fuzz_valid_to_relative.rs`](fuzz/fuzz_targets/fuzz_valid_to_relative.rs) &mdash; 89 lines

</details>

<details>
<summary><code>parity/</code> &mdash; 2 file(s)</summary>

- [`README.md`](parity/README.md) &mdash; 179 lines
- [`source-lock.yaml`](parity/source-lock.yaml) &mdash; 74 lines

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
<summary><code>parity/fixtures/app_data/schemas/</code> &mdash; 4 file(s)</summary>

- [`flashloan.json`](parity/fixtures/app_data/schemas/flashloan.json) &mdash; 60 lines
- [`hook-v0.2.0.json`](parity/fixtures/app_data/schemas/hook-v0.2.0.json) &mdash; 55 lines
- [`partner-fee-v1.0.0.json`](parity/fixtures/app_data/schemas/partner-fee-v1.0.0.json) &mdash; 118 lines
- [`quote-v1.1.0.json`](parity/fixtures/app_data/schemas/quote-v1.1.0.json) &mdash; 38 lines

</details>

<details>
<summary><code>parity/fixtures/cow_shed/</code> &mdash; 7 file(s)</summary>

- [`canonical_selectors.json`](parity/fixtures/cow_shed/canonical_selectors.json) &mdash; 61 lines
- [`domain_separator.json`](parity/fixtures/cow_shed/domain_separator.json) &mdash; 45 lines
- [`eoa_signature_byte_order.json`](parity/fixtures/cow_shed/eoa_signature_byte_order.json) &mdash; 65 lines
- [`execute_hooks_calldata.json`](parity/fixtures/cow_shed/execute_hooks_calldata.json) &mdash; 128 lines
- [`execute_hooks_digest.json`](parity/fixtures/cow_shed/execute_hooks_digest.json) &mdash; 103 lines
- [`proxy_addresses.json`](parity/fixtures/cow_shed/proxy_addresses.json) &mdash; 324 lines
- [`version_calls.json`](parity/fixtures/cow_shed/version_calls.json) &mdash; 80 lines

</details>

<details>
<summary><code>parity/fixtures/ecdsa/</code> &mdash; 1 file(s)</summary>

- [`v_normalization.json`](parity/fixtures/ecdsa/v_normalization.json) &mdash; 155 lines

</details>

<details>
<summary><code>parity/fixtures/eip712/</code> &mdash; 2 file(s)</summary>

- [`order_digests.json`](parity/fixtures/eip712/order_digests.json) &mdash; 234 lines
- [`settlement_domain_separator.json`](parity/fixtures/eip712/settlement_domain_separator.json) &mdash; 22 lines

</details>

<details>
<summary><code>parity/fixtures/orderbook/</code> &mdash; 8 file(s)</summary>

- [`onchain_order_data.json`](parity/fixtures/orderbook/onchain_order_data.json) &mdash; 16 lines
- [`order_quote_response.json`](parity/fixtures/orderbook/order_quote_response.json) &mdash; 38 lines
- [`order_with_full_metadata.json`](parity/fixtures/orderbook/order_with_full_metadata.json) &mdash; 87 lines
- [`solver_competition_response.json`](parity/fixtures/orderbook/solver_competition_response.json) &mdash; 56 lines
- [`solver_execution.json`](parity/fixtures/orderbook/solver_execution.json) &mdash; 20 lines
- [`stored_order_quote.json`](parity/fixtures/orderbook/stored_order_quote.json) &mdash; 25 lines
- [`total_surplus.json`](parity/fixtures/orderbook/total_surplus.json) &mdash; 16 lines
- [`trade.json`](parity/fixtures/orderbook/trade.json) &mdash; 34 lines

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
- [`eth_sign_typed_data_request.json`](parity/fixtures/signing/eth_sign_typed_data_request.json) &mdash; 873 lines

</details>

<details>
<summary><code>parity/openapi/</code> &mdash; 2 file(s)</summary>

- [`coverage.yaml`](parity/openapi/coverage.yaml) &mdash; 90 lines
- [`services-orderbook.yml`](parity/openapi/services-orderbook.yml) &mdash; 2,730 lines

</details>

<details>
<summary><code>tests/</code> &mdash; 15 file(s)</summary>

- [`alloy_provider_invariant_covers_every_published_crate.rs`](tests/alloy_provider_invariant_covers_every_published_crate.rs) &mdash; 31 lines
- [`alloy_read_contract_parity_invariant.rs`](tests/alloy_read_contract_parity_invariant.rs) &mdash; 104 lines
- [`alloy_signer_invariant_covers_every_published_crate.rs`](tests/alloy_signer_invariant_covers_every_published_crate.rs) &mdash; 31 lines
- [`alloy_two_family_lockfile_invariant.rs`](tests/alloy_two_family_lockfile_invariant.rs) &mdash; 112 lines
- [`alloy_two_family_pin_lockstep.rs`](tests/alloy_two_family_pin_lockstep.rs) &mdash; 94 lines
- [`alloy_umbrella_composition.rs`](tests/alloy_umbrella_composition.rs) &mdash; 116 lines
- [`Cargo.toml`](tests/Cargo.toml) &mdash; 81 lines
- [`cow_shed_typed_data_digest.rs`](tests/cow_shed_typed_data_digest.rs) &mdash; 76 lines
- [`dependency_default_features_audit.rs`](tests/dependency_default_features_audit.rs) &mdash; 82 lines
- [`msrv_consistency.rs`](tests/msrv_consistency.rs) &mdash; 37 lines
- [`signer_rejection_propagation_invariant.rs`](tests/signer_rejection_propagation_invariant.rs) &mdash; 133 lines
- [`supported_chains_doc_table.rs`](tests/supported_chains_doc_table.rs) &mdash; 103 lines
- [`transaction_lifecycle_cross_adapter_invariant.rs`](tests/transaction_lifecycle_cross_adapter_invariant.rs) &mdash; 181 lines
- [`wasm_dependency_invariant.rs`](tests/wasm_dependency_invariant.rs) &mdash; 71 lines
- [`workspace_alloy_pin_lockstep.rs`](tests/workspace_alloy_pin_lockstep.rs) &mdash; 126 lines

</details>

<details>
<summary><code>tests/support/</code> &mdash; 2 file(s)</summary>

- [`published_crates.rs`](tests/support/published_crates.rs) &mdash; 80 lines
- [`rpc.rs`](tests/support/rpc.rs) &mdash; 111 lines

</details>

<details>
<summary><code>xtask/</code> &mdash; 1 file(s)</summary>

- [`Cargo.toml`](xtask/Cargo.toml) &mdash; 27 lines

</details>

<details>
<summary><code>xtask/src/</code> &mdash; 2 file(s)</summary>

- [`lib.rs`](xtask/src/lib.rs) &mdash; 20 lines
- [`main.rs`](xtask/src/main.rs) &mdash; 266 lines

</details>

<details>
<summary><code>xtask/src/docs/</code> &mdash; 3 file(s)</summary>

- [`agree.rs`](xtask/src/docs/agree.rs) &mdash; 264 lines
- [`audit_index.rs`](xtask/src/docs/audit_index.rs) &mdash; 138 lines
- [`mod.rs`](xtask/src/docs/mod.rs) &mdash; 9 lines

</details>

<details>
<summary><code>xtask/src/parity/</code> &mdash; 5 file(s)</summary>

- [`mod.rs`](xtask/src/parity/mod.rs) &mdash; 1,191 lines
- [`openapi_coverage.rs`](xtask/src/parity/openapi_coverage.rs) &mdash; 751 lines
- [`registry_confirm.rs`](xtask/src/parity/registry_confirm.rs) &mdash; 364 lines
- [`sync.rs`](xtask/src/parity/sync.rs) &mdash; 494 lines
- [`vendor_openapi.rs`](xtask/src/parity/vendor_openapi.rs) &mdash; 67 lines

</details>

<details>
<summary><code>xtask/src/policy/</code> &mdash; 20 file(s)</summary>

- [`check_adr_coverage.rs`](xtask/src/policy/check_adr_coverage.rs) &mdash; 219 lines
- [`check_alloy_family_pins.rs`](xtask/src/policy/check_alloy_family_pins.rs) &mdash; 238 lines
- [`check_chain_patch_eligibility.rs`](xtask/src/policy/check_chain_patch_eligibility.rs) &mdash; 202 lines
- [`check_deny_unknown_fields.rs`](xtask/src/policy/check_deny_unknown_fields.rs) &mdash; 127 lines
- [`check_enum_policy.rs`](xtask/src/policy/check_enum_policy.rs) &mdash; 147 lines
- [`check_msrv_notice.rs`](xtask/src/policy/check_msrv_notice.rs) &mdash; 163 lines
- [`check_panic_allowlist.rs`](xtask/src/policy/check_panic_allowlist.rs) &mdash; 378 lines
- [`check_property_citations.rs`](xtask/src/policy/check_property_citations.rs) &mdash; 158 lines
- [`check_readme_include.rs`](xtask/src/policy/check_readme_include.rs) &mdash; 105 lines
- [`check_shell_wrappers.rs`](xtask/src/policy/check_shell_wrappers.rs) &mdash; 95 lines
- [`check_wasm_invariant.rs`](xtask/src/policy/check_wasm_invariant.rs) &mdash; 271 lines
- [`check_workflow_security.rs`](xtask/src/policy/check_workflow_security.rs) &mdash; 131 lines
- [`check_workspace_versions.rs`](xtask/src/policy/check_workspace_versions.rs) &mdash; 174 lines
- [`classify_release.rs`](xtask/src/policy/classify_release.rs) &mdash; 205 lines
- [`dependency_invariant.rs`](xtask/src/policy/dependency_invariant.rs) &mdash; 177 lines
- [`fences.rs`](xtask/src/policy/fences.rs) &mdash; 515 lines
- [`fixtures.rs`](xtask/src/policy/fixtures.rs) &mdash; 13 lines
- [`mod.rs`](xtask/src/policy/mod.rs) &mdash; 103 lines
- [`run_deterministic_examples.rs`](xtask/src/policy/run_deterministic_examples.rs) &mdash; 169 lines
- [`workspace.rs`](xtask/src/policy/workspace.rs) &mdash; 542 lines

</details>

<details>
<summary><code>xtask/tests/</code> &mdash; 12 file(s)</summary>

- [`check_adr_coverage.rs`](xtask/tests/check_adr_coverage.rs) &mdash; 51 lines
- [`check_chain_patch_eligibility.rs`](xtask/tests/check_chain_patch_eligibility.rs) &mdash; 45 lines
- [`check_deny_unknown_fields.rs`](xtask/tests/check_deny_unknown_fields.rs) &mdash; 46 lines
- [`check_enum_policy.rs`](xtask/tests/check_enum_policy.rs) &mdash; 51 lines
- [`check_msrv_notice.rs`](xtask/tests/check_msrv_notice.rs) &mdash; 37 lines
- [`check_panic_allowlist.rs`](xtask/tests/check_panic_allowlist.rs) &mdash; 192 lines
- [`check_property_citations.rs`](xtask/tests/check_property_citations.rs) &mdash; 84 lines
- [`check_workspace_versions.rs`](xtask/tests/check_workspace_versions.rs) &mdash; 26 lines
- [`classify_release.rs`](xtask/tests/classify_release.rs) &mdash; 103 lines
- [`openapi_coverage.rs`](xtask/tests/openapi_coverage.rs) &mdash; 162 lines
- [`registry_confirm.rs`](xtask/tests/registry_confirm.rs) &mdash; 157 lines
- [`vendor_openapi.rs`](xtask/tests/vendor_openapi.rs) &mdash; 104 lines

</details>

<details>
<summary><code>xtask/tests/common/</code> &mdash; 1 file(s)</summary>

- [`mod.rs`](xtask/tests/common/mod.rs) &mdash; 158 lines

</details>


