# Security And Test Matrix

This matrix maps `cow-rs` test evidence by crate and review surface. It is a navigation aid for reviewers, not a claim that tests prove the absence of bugs.

## Core SDK Crates

| Crate | Boundary | Evidence | Primary command |
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

## Examples And Browser Surfaces

| Surface | Boundary | Evidence | Primary command |
| --- | --- | --- | --- |
| Native examples | Deterministic consumer scenarios for app-data, signing, orderbook, trading, and subgraph behavior | `examples/native/tests/scenario_contract.rs` plus runnable scenario binaries | `cargo test --manifest-path examples/native/Cargo.toml` |
| Native scenario binaries | Reviewer-readable command output without live order placement | `examples/native/scenarios/*.rs` | `cargo check --manifest-path examples/native/Cargo.toml --examples` |
| SDK WASM verification console | WASM-compatible SDK verification surface with deterministic exports; network-backed quote, orderbook, and subgraph controls stay manual verification surfaces | `examples/wasm/sdk-verification-console/tests/deterministic_exports.rs` | `wasm-pack test --headless --chrome` |
| Browser wallet WASM console | Browser wallet verification shell that separates deterministic mock mode from injected-provider inspection | `examples/wasm/browser-wallet-console` build | `cargo build --target wasm32-unknown-unknown --manifest-path examples/wasm/browser-wallet-console/Cargo.toml` |

## Workspace Gates

| Gate | Purpose |
| --- | --- |
| `cargo fmt --all --check` | Formatting gate for consistent public diffs |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Lint gate across crates and test targets |
| `cargo test --workspace` | Main workspace test gate |
| `cargo doc --workspace --no-deps` | Public rustdoc build gate |

## Review Boundaries

- Required tests and examples avoid private keys, seed phrases, live wallet authorization, and live order submission.
- Mocked transports should assert request shape and failure behavior where those paths are part of the reviewed surface.
- WASM/browser evidence is separated from native examples so browser runtime assumptions stay visible.
- Live quote, orderbook, subgraph, and wallet checks stay manual unless explicitly promoted into a deterministic routed or injected test.
- Schema-derived evidence stays test-only and outside the public SDK API.
- `cow-sdk-browser-wallet` tests and mock console mode provide deterministic proof. Injected-provider execution remains environment-sensitive because authorization, chain inventory, and wallet UX are controlled by the browser extension.
