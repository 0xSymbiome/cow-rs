# Validation Smoke

This directory contains the optional smoke runner for environment-sensitive validation of the current `cow-rs` surfaces.

Use it when you want a disciplined manual confirmation pass for:

- live orderbook reachability through the SDK surface
- live subgraph reachability through the SDK surface
- injected-wallet confirmation in the browser-wallet console
- deployed WASM page inspection for the browser-wallet and SDK verification consoles
- deployment-registry bytecode confirmation against configured RPC endpoints
- pinned Chrome-for-Testing setup for WASM browser tests

This kit is intentionally separate from routine deterministic validation. It does not belong in branch protection and it does not replace the maintained deterministic proof surfaces documented in the repository root and validation docs.

## Commands

```text
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- orderbook-live
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- subgraph-live
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- browser-wallet-live --url http://127.0.0.1:8081
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- wasm-pages --sdk-verification-url https://<owner>.github.io/<repo>/sdk-verification-console/ --browser-wallet-url https://<owner>.github.io/<repo>/browser-wallet-console/
cargo registry-confirm --mode local --check --chain-ids 1,100
cargo wasm-runner-refresh --source fallback --fallback-path scripts/validation-smoke/data/cft-fallback.json
cargo wasm-runner-setup --webdriver-json target/wasm-runner/webdriver.json
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- all
```

## Exit Codes

- `0`: every selected check passed
- `1`: at least one selected check failed in a way that indicates a likely regression or broken contract
- `2`: at least one selected check was unavailable or incomplete because required local hosts, credentials, or deployed pages were not reachable

## Environment

### Orderbook

- `COW_SMOKE_ORDERBOOK_ENV`
  - optional
  - accepted values: `prod`, `staging`
  - default: `prod`
- `COW_SMOKE_ORDERBOOK_CHAIN_ID`
  - optional
  - default: `1`
- `COW_SMOKE_ORDERBOOK_BASE_URL`
  - optional explicit base URL override
- `COW_SMOKE_ORDERBOOK_API_KEY`
  - optional partner API key for partner endpoint resolution

### Subgraph

- `COW_SMOKE_THE_GRAPH_API_KEY`
  - optional smoke-specific alias for `THE_GRAPH_API_KEY`
- `THE_GRAPH_API_KEY`
  - required for the live subgraph query example when the smoke-specific alias is not supplied
- `COW_SMOKE_SUBGRAPH_CHAIN_ID`
  - optional smoke-specific alias for `COW_SUBGRAPH_CHAIN_ID`
- `COW_SUBGRAPH_CHAIN_ID`
  - optional
  - default: mainnet

### Browser And Deployed Pages

- `COW_SMOKE_BROWSER_WALLET_URL`
  - local browser-wallet console URL for injected-wallet confirmation readiness
  - default: `http://127.0.0.1:8081`
- `COW_SMOKE_BROWSER_WALLET_PAGES_URL`
  - optional deployed browser-wallet console URL
- `COW_SMOKE_SDK_VERIFICATION_PAGES_URL`
  - optional deployed SDK verification console URL

### Deployment Registry Confirmation

- `RPC_<chain_id>`
  - required for each selected production chain in `registry-confirm --mode release`
  - optional in `registry-confirm --mode local`; missing RPC endpoints are reported as skipped
- `RPC_MAINNET`, `RPC_GNOSIS`, `RPC_ARBITRUM`, `RPC_BASE`, `RPC_POLYGON`, `RPC_AVALANCHE`, `RPC_BNB`, `RPC_SEPOLIA`, `RPC_PLASMA`, `RPC_LINEA`, `RPC_INK`
  - accepted aliases for the corresponding supported chain ids

Use `--check` for read-only confirmation. Use `--write` only when intentionally refreshing committed deployment evidence.

### WASM Browser Runner

- `WEBDRIVER_JSON`
  - fallback output path for `wasm-runner-setup` when `--webdriver-json` is not supplied

`wasm-runner-refresh --source fallback` is offline-deterministic and uses the committed Chrome-for-Testing snapshot. `--source online` reads the live Chrome-for-Testing Stable metadata and hashes downloaded archives when a matching checksum is not already present in the pinned YAML.

## Interpretation

- unavailable local hosts, missing credentials, and offline services are reported as `unavailable`, not as deterministic regressions
- unexpected successful responses with broken payload shape, missing expected page markers, or failing live example logic are reported as `fail`
- the browser-wallet step checks that the console page is reachable and exposes the expected stable markers before handing off to operator-driven injected-wallet actions
