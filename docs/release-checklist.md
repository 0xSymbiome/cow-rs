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

## Depth Reporting

`test-depth.yml` is the maintained depth-reporting lane. It publishes read-only coverage and mutation reports for follow-up work; it does not replace the release gates above and it does not introduce threshold-based branch protection.

Coverage uses an explicit nightly toolchain because doctest coverage is still an unstable rustdoc path:

```text
cargo +nightly llvm-cov --workspace --all-features --doctests --json --summary-only --output-path target/coverage-summary.json --ignore-filename-regex "(^|/)(tests|examples|e2e)(/|$)|crates/subgraph/src/query_documents/|crates/subgraph/tests/schema_evidence/"
cargo +nightly llvm-cov report --lcov --output-path target/coverage-lcov.info --ignore-filename-regex "(^|/)(tests|examples|e2e)(/|$)|crates/subgraph/src/query_documents/|crates/subgraph/tests/schema_evidence/"
```

Interpretation rules:

- the report covers deterministic crate tests and doctests only
- test sources, example shells, browser automation, and generated subgraph query or schema evidence are excluded from the reported file set
- the workflow publishes summaries and artifacts; it does not define minimum percentage gates

Mutation stays manual in the first cut and is intentionally targeted to narrow deterministic helper families:

```text
cargo mutants -p cow-sdk-contracts -p cow-sdk-signing -p cow-sdk-app-data --output target/mutants-report
```

```text
cargo mutants -p cow-sdk-orderbook -p cow-sdk-trading --file crates/orderbook/src/request.rs --file crates/orderbook/src/transform.rs --file crates/trading/src/order.rs --file crates/trading/src/slippage.rs --annotations none --no-times --re "decoded_body|execute_with|calculate_total_fee|add_decimal_strings|sanitize_protocol_fee_bps|partner_fee_bps|calculate_unique_order_id|adjust_buy_amount" --output target/mutants-report-orderbook-trading
```

```text
cargo mutants -p cow-sdk-subgraph -p cow-sdk-browser-wallet --file crates/subgraph/src/api.rs --file crates/subgraph/src/types.rs --file crates/browser-wallet/src/wallet.rs --file crates/browser-wallet/src/provider.rs --file crates/browser-wallet/src/error.rs --annotations none --no-times --re "run_query_with_config|config_with_override|base_url_for|deserialize_string_or_number|deserialize_optional_string_or_number|deserialize_u64_from_string_or_number|value_to_string|single_wallet|wallet_at|requires_explicit_selection|refresh_session|switch_or_add_chain|switch_chain_request|add_chain_request|validate_wallet_text|validate_wallet_url|query_accounts|query_chain_id|reset_session|parse_chain_id_value|parse_quantity_to_decimal|parse_address_array|transaction_to_rpc|from_rpc" --output target/mutants-report-subgraph-browser-wallet
```

Interpretation rules:

- surviving mutants are explicit follow-up work items, not a branch-protection threshold
- orderbook and trading mutation runs stay scoped to explicit decode, transform, slippage, and order-id helper families so transport and orchestration results remain interpretable
- subgraph and browser-wallet mutation runs stay scoped to explicit query execution, scalar decoding, discovery selection, RPC classification, session refresh, and typed provider request-shaping helpers
- the full `mutants.out/` report is preserved as an artifact so surviving and unviable cases can be inspected directly
- live extension flows, WASM example packaging, and other environment-sensitive surfaces stay outside the helper-family mutation lanes

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

Run the deterministic browser-wallet console checks:

```text
cargo test -p cow-sdk-browser-wallet
bun install --cwd e2e/browser-wallet
bun run --cwd e2e/browser-wallet playwright install chromium
bun run --cwd e2e/browser-wallet test
```

Browser-wallet validation is intentionally split:

- deterministic proof comes from `cargo test -p cow-sdk-browser-wallet`, mock-wallet console mode, the browser-wallet console WASM build, and the committed browser-wallet console automation using local EIP-6963 fixtures plus route-mocked orderbook requests
- live extension-backed connect, sign, quote, submit, and cancel checks remain optional because authorization persistence, vendor prompts, chain inventory, and wallet-specific behavior are controlled by the installed extension rather than normalized by the SDK

## Optional Validation Smoke

Use the smoke kit when a change needs live service confirmation, live extension-backed wallet confirmation, or deployed-page inspection in addition to the deterministic proof surfaces above.

The smoke kit is intentionally opt-in:

- it is not part of routine CI
- it does not join branch protection
- it distinguishes unavailable services and missing local prerequisites from actual regressions

Commands:

```text
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- orderbook-live
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- subgraph-live
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- browser-wallet-live --url http://127.0.0.1:8081
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- wasm-pages --sdk-verification-url https://<owner>.github.io/<repo>/sdk-verification-console/ --browser-wallet-url https://<owner>.github.io/<repo>/browser-wallet-console/
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- all
```

Interpretation rules:

- exit code `0` means every selected smoke surface passed
- exit code `2` means at least one selected surface was unavailable because a local host, deployed page, credential, or remote service was not reachable
- exit code `1` means the selected surface responded but broke an expected contract such as payload shape, page markers, or live example behavior
- the browser-wallet live check confirms local page readiness and stable markers before operator-driven injected-wallet actions; it does not claim extension automation
- the full environment and URL contract lives in `scripts/validation-smoke/README.md`

## Manual Confirmation Before Publish

- Serve the WASM examples over HTTP and confirm that the built artifacts load correctly.
- If `examples/wasm/browser-wallet-console/` changed, run an extension-backed spot check on a supported chain and confirm the deterministic fixture path and mock-wallet path still behave as documented.
- If GitHub Pages content changed, inspect the deployed `sdk-verification-console/` and `browser-wallet-console/` pages after `wasm-pages.yml` completes.
- If parity inputs changed, confirm that the pinned SHAs in `parity/source-lock.yaml` still match the intended upstream revisions and that fixture provenance remains aligned.
