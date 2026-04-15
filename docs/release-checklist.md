# Release Checklist

Use this checklist before tagging or publishing a release that changes the
public `cow-rs` surface.

## 1. Native Quality Gates

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets --all-features --message-format short -- -W clippy::pedantic -W clippy::cargo -A clippy::multiple_crate_versions
cargo test --workspace
cargo nextest run --workspace --all-features --config-file .github/config/nextest.toml
typos --config .github/config/typos.toml
cargo deny check bans licenses sources --config .github/config/deny.toml
cargo audit --deny unsound --deny unmaintained --ignore RUSTSEC-2026-0097
```

`cargo audit` is the blocking RustSec gate for published advisories. It keeps
vulnerabilities, unsound advisories, and unmaintained advisories blocking while
leaving yanked-only published-upstream cases reviewable through public audit
evidence until a published replacement exists.

## 2. Documentation And Public API Gates

```text
cargo test --workspace --doc
cargo test --all-features --workspace --doc
cargo doc --workspace --all-features --no-deps
RUSTFLAGS="-Wmissing-docs -Wmissing-debug-implementations -Wunreachable-pub -Wunnameable-types" cargo check --workspace --all-features
```

Nightly docs.rs-style lane:

```text
DOCS_RS=1 RUSTDOCFLAGS="--cfg docsrs -D warnings -Zunstable-options --generate-link-to-definition --show-type-layout --enable-index-page" cargo +nightly doc --workspace --all-features --no-deps
```

If the release diff materially touches a surface covered by `docs/audit/`,
confirm that the affected audit is still `Current`. If the reviewed surface
changed, refresh or supersede the audit in the same change set before tagging.

## 3. Compatibility And Host Coverage

```text
cargo +1.94.0 check --workspace --all-features
cargo +1.94.0 test --workspace
cargo hack check --workspace --feature-powerset --depth 1
```

Expected workflow coverage:

- `ci.yml` for routine native validation and the compatibility floor
- Windows stable lane for light native host coverage
- `crate-checks.yml` for maintenance-depth crate isolation
- `codeql.yml` for semantic security analysis

## 4. Depth Reporting

`test-depth.yml` is the maintained depth-reporting lane. It publishes coverage
and mutation reports for follow-up work without defining threshold-based branch
protection.

Coverage:

```text
cargo +nightly llvm-cov --workspace --all-features --doctests --json --summary-only --output-path target/coverage-summary.json --ignore-filename-regex "(^|/)(tests|examples|e2e)(/|$)|crates/subgraph/src/query_documents/|crates/subgraph/tests/schema_evidence/"
cargo +nightly llvm-cov --workspace --all-features --doctests --lcov --output-path target/coverage-lcov.info --ignore-filename-regex "(^|/)(tests|examples|e2e)(/|$)|crates/subgraph/src/query_documents/|crates/subgraph/tests/schema_evidence/"
```

Mutation:

```text
cargo mutants -p cow-sdk-contracts -p cow-sdk-signing -p cow-sdk-app-data --output target/mutants-report
cargo mutants -p cow-sdk-orderbook -p cow-sdk-trading --file crates/orderbook/src/request.rs --file crates/orderbook/src/transform.rs --file crates/trading/src/order.rs --file crates/trading/src/slippage.rs --annotations none --no-times --re "decoded_body|execute_with|calculate_total_fee|add_decimal_strings|sanitize_protocol_fee_bps|partner_fee_bps|calculate_unique_order_id|adjust_buy_amount" --output target/mutants-report-orderbook-trading
cargo mutants -p cow-sdk-subgraph -p cow-sdk-browser-wallet --file crates/subgraph/src/api.rs --file crates/subgraph/src/types.rs --file crates/browser-wallet/src/wallet.rs --file crates/browser-wallet/src/provider.rs --file crates/browser-wallet/src/error.rs --annotations none --no-times --re "run_query_with_config|config_with_override|base_url_for|deserialize_string_or_number|deserialize_optional_string_or_number|deserialize_u64_from_string_or_number|value_to_string|single_wallet|wallet_at|requires_explicit_selection|refresh_session|switch_or_add_chain|switch_chain_request|add_chain_request|validate_wallet_text|validate_wallet_url|query_accounts|query_chain_id|reset_session|parse_chain_id_value|parse_quantity_to_decimal|parse_address_array|transaction_to_rpc|from_rpc" --output target/mutants-report-subgraph-browser-wallet
```

## 5. Repo-Local Parity And Publication Proof

Reserved-placeholder `0.0.1-reserved.0` crates.io and docs.rs entries may
already be live for published crate names. Treat those publishes as
name-reservation perimeter only; they do not satisfy the functional release
contract described in this checklist.

Validate the committed parity contract from the current checkout:

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

## 6. Provenance-Sensitive Parity Proof

Use this lane when the release needs explicit proof against pinned upstream
repositories instead of only the committed fixture contract.

Quick setup:

```text
git clone https://github.com/cowprotocol/cow-sdk.git <path>/cow-sdk
git clone https://github.com/cowprotocol/contracts.git <path>/contracts
git clone https://github.com/cowprotocol/services.git <path>/services
```

If you prefer the parity maintainer to create the sibling checkouts from the
pinned source lock, run:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- provision-upstreams --source-lock parity/source-lock.yaml --output-root <path>
```

Then validate against those independent roots:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml --cow-sdk-root <path>/cow-sdk --contracts-root <path>/contracts --services-root <path>/services
```

Rules:

- supplied roots must be independent git checkouts or worktrees
- same-checkout directory copies are not valid provenance evidence
- `release-readiness.yml` owns the routine automated provenance-sensitive lane

## 7. WASM And Browser Surfaces

Build the WASM surfaces:

```text
cargo build --target wasm32-unknown-unknown -p cow-sdk
cargo build --target wasm32-unknown-unknown -p cow-sdk --features browser-wallet
cargo build --target wasm32-unknown-unknown -p cow-sdk-app-data
cargo build --target wasm32-unknown-unknown --manifest-path examples/wasm/browser-wallet-console/Cargo.toml
```

Deterministic SDK verification console checks:

```text
cd examples/wasm/sdk-verification-console
wasm-pack test --headless --chrome
```

```text
bun install --cwd e2e/sdk-verification
bun run --cwd e2e/sdk-verification playwright install chromium
bun run --cwd e2e/sdk-verification test
```

Deterministic browser-wallet console checks:

```text
cargo test -p cow-sdk-browser-wallet
bun install --cwd e2e/browser-wallet
bun run --cwd e2e/browser-wallet playwright install chromium
bun run --cwd e2e/browser-wallet test
```

## 8. Optional Validation Smoke

Use the smoke kit when a change needs live service confirmation, live
extension-backed wallet confirmation, or deployed-page inspection in addition
to the deterministic proof surfaces above.

```text
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- orderbook-live
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- subgraph-live
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- browser-wallet-live --url http://127.0.0.1:8081
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- wasm-pages --sdk-verification-url https://<owner>.github.io/<repo>/sdk-verification-console/ --browser-wallet-url https://<owner>.github.io/<repo>/browser-wallet-console/
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- all
```

## 9. Manual Confirmation Before Publish

- serve the WASM examples over HTTP and confirm that the built artifacts load
- if the browser-wallet console changed, run an extension-backed spot check on
  a supported chain
- if GitHub Pages content changed, inspect the deployed console pages after the
  Pages workflow completes
- if parity inputs changed, confirm that the pinned SHAs and fixture provenance
  still align
