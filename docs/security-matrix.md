# Security And Validation Matrix

This matrix maps `cow-rs` validation evidence by crate, example surface, and workflow lane. It is a navigation aid, not a claim that tests prove the absence of bugs.

Canonical references:

- [Validation Scope](validation-scope.md)
- [Release Checklist](release-checklist.md)
- [Verification Guide](verification-guide.md)
- [Parity Matrix](parity-matrix.md)

## Core SDK Crates

| Crate | Boundary | Deterministic evidence | Primary command |
| --- | --- | --- | --- |
| `cow-sdk-core` | Shared chain config, domain types, and runtime traits | `config_contract.rs`, `types_contract.rs`, `traits_contract.rs` | `cargo test -p cow-sdk-core` |
| `cow-sdk-contracts` | Contract constants, ABI-shaped order helpers, hashing, settlement/vault/proxy/reader helpers | `order_contract.rs`, `signature_contract.rs`, `deployment_contract.rs`, `settlement_contract.rs`, `vault_contract.rs`, `proxy_contract.rs`, `reader_contract.rs`, `swap_contract.rs`, `interaction_contract.rs` | `cargo test -p cow-sdk-contracts` |
| `cow-sdk-signing` | EIP-712 order signing, EIP-1271 payloads, cancellation signing, domain separation | `order_signing_contract.rs`, `eip1271_contract.rs`, `cancellation_contract.rs`, `domain_contract.rs` | `cargo test -p cow-sdk-signing` |
| `cow-sdk-app-data` | App-data schema handling, CID conversion, fail-closed CID/app-data encoding, fetch, and pinning seams | `app_data_info_contract.rs`, `cid_contract.rs`, `schema_contract.rs`, `fetch_contract.rs`, `pinning_contract.rs`, `docs/audit/cid-dependency-audit.md` | `cargo test -p cow-sdk-app-data` |
| `cow-sdk-orderbook` | Typed orderbook transport, retry/status behavior, DTO conversion, source-schema evidence | `api_contract.rs`, `request_contract.rs`, `transform_contract.rs`, `types_contract.rs`, `schema_source_contract.rs` | `cargo test -p cow-sdk-orderbook` |
| `cow-sdk-trading` | Quote, post, allowance, approval, cancellation, slippage, and SDK workflow orchestration | `quote_contract.rs`, `post_contract.rs`, `allowance_contract.rs`, `cancel_contract.rs`, `onchain_contract.rs`, `slippage_contract.rs`, `sdk_contract.rs`, `parity_contract.rs` | `cargo test -p cow-sdk-trading` |
| `cow-sdk-subgraph` | Read-only GraphQL query construction, typed responses, source-schema evidence | `api_contract.rs`, `query_contract.rs`, `types_contract.rs`, `schema_source_contract.rs` | `cargo test -p cow-sdk-subgraph` |
| `cow-sdk-browser-wallet` | EIP-1193 browser wallet provider/signer boundaries, deterministic mock proof, typed session updates, typed chain management, and typed-data transport | `provider_contract.rs`, `wallet_contract.rs` | `cargo test -p cow-sdk-browser-wallet` |
| `cow-sdk` | Thin facade exports and public package surface | `public_api.rs` | `cargo test -p cow-sdk` |

## Examples And Runtime Surfaces

| Surface | Boundary | Deterministic evidence | Environment-sensitive or manual boundary | Primary command |
| --- | --- | --- | --- | --- |
| Native examples | Deterministic consumer scenarios for app-data, signing, orderbook, quote-only, limit-order, native-sell / EthFlow, pre-sign, off-chain cancellation, on-chain cancellation, and subgraph behavior | `examples/native/tests/scenario_contract.rs` plus runnable scenario binaries including `ethflow_transaction_simulation.rs` and `onchain_order_actions_simulation.rs` | `subgraph_live_query` remains opt-in because it depends on external configuration. | `cargo test --manifest-path examples/native/Cargo.toml` |
| Native scenario binaries | Readable command output for the complete native trading workflow surface without live order placement | `examples/native/scenarios/*.rs` | None beyond the explicit opt-in live subgraph scenario. | `cargo check --manifest-path examples/native/Cargo.toml --examples` |
| SDK WASM verification console | WASM-compatible SDK verification surface with deterministic exports | `examples/wasm/sdk-verification-console/tests/deterministic_exports.rs`, `wasm-pack test --headless --chrome`, `sdk-verification-e2e.yml` | Quote, orderbook, and subgraph actions remain manual when pointed at live endpoints. | `wasm-pack test --headless --chrome` |
| Browser wallet WASM console | Browser wallet verification shell that separates deterministic mock mode from injected-provider execution | `cargo test -p cow-sdk-browser-wallet`, mock-wallet console mode, the browser-wallet console WASM build, and `browser-wallet-e2e.yml` with local EIP-6963 fixtures plus route-mocked orderbook requests | Live extension-backed connect, sign, quote, submit, and cancel remain environment-sensitive because they depend on the installed wallet, authorization state, and vendor-specific behavior. | `bun run --cwd e2e/browser-wallet test` |

## Workspace Gates

| Gate | Purpose |
| --- | --- |
| `cargo fmt --all --check` | Formatting gate for consistent public diffs |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Lint gate across crates and test targets |
| `cargo test --workspace` | Main workspace test gate |
| `cargo test --workspace --doc` | Explicit doctest gate for rustdoc examples |
| `cargo test --all-features --workspace --doc` | All-feature doctest gate for the public docs contract |
| Windows stable lane (`windows-latest`) | Light native host compatibility gate with `cargo check --workspace --all-features` and `cargo test --workspace --lib --tests` |
| `cargo doc --workspace --all-features --no-deps` | Public rustdoc build gate |
| `docs-quality.yml` | Nightly docs.rs-style rustdoc lane with `DOCS_RS=1`, `--cfg docsrs`, and nightly rustdoc presentation flags |
| `RUSTFLAGS="-Wmissing-docs -Wmissing-debug-implementations -Wunreachable-pub -Wunnameable-types" cargo check --workspace --all-features` | Blocking public API rustc lint gate for the published crate family |
| `codeql.yml` | Dedicated semantic security-analysis gate for Rust and GitHub Actions |
| `cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml` | Repo-local parity fixture and source-lock gate for committed publication evidence |
| `ci-success` | Aggregate routine CI status for branch protection across the required native validation and publication jobs |

## Publication Gates

- `ci.yml` runs the repo-local publication contract: `parity/source-lock.yaml` validation plus the full published package-family dry-run from the current workspace.
- `release-readiness.yml` reruns that repo-local contract and then provisions pinned independent upstream clones from `parity/source-lock.yaml` before explicit-root provenance validation.
- Same-checkout copies are not treated as provenance evidence for upstream parity sources.

## Validation Boundaries

- Required tests and examples avoid private keys, seed phrases, live wallet authorization, and live order submission.
- Doctests stay deterministic and are limited to local examples that do not require live-network or host-specific behavior.
- The nightly docs-quality lane stays documentation-only. It exercises docs.rs-style rustdoc flags and all-feature doctests without widening validation into browser-extension, live-network, or host-sensitive behavior.
- The Windows stable lane stays intentionally narrow and does not absorb browser-target, WASM, or publication-only validation.
- CodeQL complements dependency policy by scanning Rust and GitHub Actions semantics; it does not replace `cargo-deny` or `cargo-audit`.
- Routine native validation workflows and the dedicated WASM workflows disable checkout credential persistence and use explicit timeout budgets per job. `wasm-pages.yml` scopes elevated Pages permissions to the deployment job.
- Mocked transports should assert request shape and failure behavior where those paths are part of the validated surface.
- WASM/browser evidence is separated from native examples so browser runtime assumptions stay visible.
- Live quote, orderbook, subgraph, and wallet checks stay manual unless explicitly promoted into a deterministic routed or injected test.
- Schema-derived evidence stays test-only and outside the public SDK API.
- `cow-sdk-browser-wallet` tests, mock console mode, and the committed browser-wallet console automation provide deterministic proof without a live extension, public RPC endpoint, or external website.
- Extension-backed injected-provider execution remains environment-sensitive because authorization, chain inventory, wallet UX, and vendor-specific behavior are controlled by the installed extension.
- The public rustc lint gate applies to `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-subgraph`, `cow-sdk-trading`, `cow-sdk-browser-wallet`, and the `cow-sdk` facade.
