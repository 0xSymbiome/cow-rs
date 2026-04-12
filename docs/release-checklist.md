# Release Checklist

Use this checklist before tagging or publishing a release that changes the public `cow-rs` surface.

## Native Quality Gates

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets --all-features --message-format short -- -W clippy::pedantic -W clippy::cargo -A clippy::multiple_crate_versions
cargo test --workspace
cargo nextest run --workspace --all-features --config-file .github/config/nextest.toml
typos --config .github/config/typos.toml
cargo deny check bans licenses sources --config .github/config/deny.toml
cargo audit --deny warnings --ignore RUSTSEC-2026-0097
```

## Documentation And Public API Gates

```text
cargo test --workspace --doc
cargo test --all-features --workspace --doc
cargo doc --workspace --all-features --no-deps
RUSTFLAGS="-Wmissing-docs -Wmissing-debug-implementations -Wunreachable-pub -Wunnameable-types" cargo check --workspace --all-features
```

`docs-quality.yml` extends the same contract with a nightly docs.rs-style lane:

```text
DOCS_RS=1 RUSTDOCFLAGS="--cfg docsrs -D warnings -Zunstable-options --generate-link-to-definition --show-type-layout --enable-index-page" cargo +nightly doc --workspace --all-features --no-deps
```

## Compatibility And Host Coverage

```text
cargo +1.94.0 check --workspace --all-features
cargo +1.94.0 test --workspace
cargo hack check --workspace --feature-powerset --depth 1
```

Workflow expectations:

- `ci.yml` includes the compatibility-floor lane and the routine native validation contract.
- `ci.yml` also runs a light Windows stable lane with `cargo check --workspace --all-features` and `cargo test --workspace --lib --tests`.
- `crate-checks.yml` is the maintenance-depth lane for crate isolation and `--each-feature` assumptions.
- `codeql.yml` remains the dedicated semantic security-analysis workflow for Rust and GitHub Actions.

## Repo-Local Parity And Publication Proof

This repository keeps repo-local publication proof separate from provenance-sensitive parity proof.

First, validate the committed parity contract from the current checkout:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml
```

Then run the published package-family dry-run in release order:

```text
cargo package -p cow-sdk-core --allow-dirty
cargo package -p cow-sdk-contracts --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo package -p cow-sdk-app-data --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo package -p cow-sdk-orderbook --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo package -p cow-sdk-signing --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'"
cargo package -p cow-sdk-subgraph --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo package -p cow-sdk-trading --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-signing.path='crates/signing'" --config "patch.crates-io.cow-sdk-app-data.path='crates/app-data'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'"
cargo package -p cow-sdk-browser-wallet --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'"
cargo package -p cow-sdk --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-signing.path='crates/signing'" --config "patch.crates-io.cow-sdk-app-data.path='crates/app-data'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'" --config "patch.crates-io.cow-sdk-trading.path='crates/trading'" --config "patch.crates-io.cow-sdk-browser-wallet.path='crates/browser-wallet'"
```

`ci.yml` covers the repo-local contract on routine changes. `release-readiness.yml` reruns it in the expanded release path.

## Provenance-Sensitive Parity Proof

Use this lane when the release needs explicit proof against pinned upstream repositories rather than only the committed fixture contract.

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- provision-upstreams --source-lock parity/source-lock.yaml --output-root <path>
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml --cow-sdk-root <path>/cow-sdk --contracts-root <path>/contracts --services-root <path>/services
```

Rules:

- The supplied roots must be independent git checkouts or worktrees at the pinned commits.
- Same-checkout directory copies are not valid provenance evidence.
- `release-readiness.yml` owns the routine provenance-sensitive automation path.

## WASM And Browser Surfaces

Build the WASM surfaces explicitly:

```text
cargo build --target wasm32-unknown-unknown -p cow-sdk
cargo build --target wasm32-unknown-unknown -p cow-sdk --features browser-wallet
cargo build --target wasm32-unknown-unknown -p cow-sdk-app-data
cargo build --target wasm32-unknown-unknown --manifest-path examples/wasm/browser-wallet-console/Cargo.toml
```

Run the deterministic SDK verification console checks:

```text
cd examples/wasm/sdk-verification-console
wasm-pack test --headless --chrome
```

```text
bun install --cwd e2e/sdk-verification
bun run --cwd e2e/sdk-verification playwright install chromium
bun run --cwd e2e/sdk-verification test
```

Browser-wallet validation is intentionally split:

- deterministic proof comes from `cargo test -p cow-sdk-browser-wallet`, mock-wallet console mode, and the browser-wallet console WASM build
- injected-provider connect, sign, quote, submit, and cancel flows remain environment-sensitive because they depend on browser extensions, authorization state, and wallet-specific behavior

## Manual Confirmation Before Publish

- Serve the WASM examples over HTTP and confirm that the built artifacts load correctly.
- If `examples/wasm/browser-wallet-console/` changed, run an injected-wallet spot check on a supported chain and confirm the mock-wallet path still behaves as documented.
- If GitHub Pages content changed, inspect the deployed `sdk-verification-console/` and `browser-wallet-console/` pages after `wasm-pages.yml` completes.
- If parity inputs changed, confirm that the pinned SHAs in `parity/source-lock.yaml` still match the intended upstream revisions and that fixture provenance remains aligned.
