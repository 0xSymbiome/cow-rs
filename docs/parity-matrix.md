# Parity Matrix

This matrix maps the main TypeScript surfaces to the current Rust workspace as of 2026-04-09.

| TypeScript surface | Rust crate(s) | Status | Current scope | Public evidence |
| --- | --- | --- | --- | --- |
| `@cowprotocol/cow-sdk` root entrypoint | `cow-sdk` | Implemented | Thin facade over the stable Rust SDK family; browser wallet remains feature-gated | `crates/sdk/src/lib.rs`, `crates/sdk/tests/public_api.rs` |
| Trading workflows | `cow-sdk-trading` | Implemented | Quote, build, sign, submit, cancel, allowance, approval, slippage, and SDK-style orchestration | `crates/trading/tests/quote_contract.rs`, `crates/trading/tests/post_contract.rs`, `crates/trading/tests/cancel_contract.rs`, `crates/trading/tests/allowance_contract.rs`, `crates/trading/tests/sdk_contract.rs` |
| Signing helpers | `cow-sdk-signing` | Implemented | EIP-712 domain helpers, order signing, cancellation signing, UID generation, EIP-1271 payloads | `crates/signing/tests/order_signing_contract.rs`, `crates/signing/tests/cancellation_contract.rs`, `crates/signing/tests/eip1271_contract.rs` |
| Contract helpers | `cow-sdk-contracts` | Implemented | Order hashing, settlement encoding, swap helpers, deployment, proxy, vault, and storage-reader helpers | `crates/contracts/tests/order_contract.rs`, `crates/contracts/tests/settlement_contract.rs`, `crates/contracts/tests/proxy_contract.rs`, `crates/contracts/tests/vault_contract.rs` |
| App-data | `cow-sdk-app-data` | Implemented | Schema validation, deterministic document generation, CID conversion, fetch and pinning seams | `crates/app-data/tests/schema_contract.rs`, `crates/app-data/tests/cid_contract.rs`, `crates/app-data/tests/pinning_contract.rs` |
| Orderbook transport | `cow-sdk-orderbook` | Implemented | Typed API transport, request policy, quote/order/trade decoding, total-fee and EthFlow transforms | `crates/orderbook/tests/api_contract.rs`, `crates/orderbook/tests/request_contract.rs`, `crates/orderbook/tests/transform_contract.rs` |
| Subgraph queries | `cow-sdk-subgraph` | Implemented as a separate crate | Read-only totals, day-volume, hour-volume, and custom query execution | `crates/subgraph/tests/api_contract.rs`, `crates/subgraph/tests/query_contract.rs`, `examples/native/scenarios/subgraph_query_roundtrip.rs` |
| WASM target | `cow-sdk`, `cow-sdk-app-data`, `cow-sdk-browser-wallet` | Implemented | `wasm32-unknown-unknown` builds for SDK, app-data, and browser wallet surfaces | `examples/wasm/sdk-verification-console/`, `examples/wasm/browser-wallet-console/` |
| Browser wallet integration | `cow-sdk-browser-wallet`, `cow-sdk` with `browser-wallet` feature | Implemented | Async EIP-1193 provider and signer, wallet session control, mock and injected wallet flows | `crates/browser-wallet/tests/provider_contract.rs`, `crates/browser-wallet/tests/wallet_contract.rs`, `examples/wasm/browser-wallet-console/README.md` |
| Examples and release posture | Workspace | Implemented | Native showcase examples, WASM consoles, and package dry-run validation | `examples/native/README.md`, `examples/README.md`, `examples/wasm/README.md` |

## Notes

- `cow-sdk-subgraph` is intentionally separate from the root `cow-sdk` facade.
- Browser wallet support is additive; native consumers do not pay for browser-only dependencies.
- The workspace is structured as a crate family, not a monolith, so leaf crates can be used directly when needed.
