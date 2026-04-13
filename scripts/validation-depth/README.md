# Validation Depth

This directory contains the maintained Rust-native helper for the non-blocking
`test-depth.yml` lane.

The lane stays report-first:

- coverage reports deterministic crate-test and doctest depth
- mutation reports targeted helper-family mutation outcomes
- neither report introduces branch-protection thresholds

The tool keeps retained trend reporting explicit and reviewable by comparing
the current report to the latest stored snapshot artifact when one is
available.

## Snapshot Artifacts

- `test-depth-coverage-trend`
- `test-depth-mutation-core-trend`
- `test-depth-mutation-orderbook-trading-trend`
- `test-depth-mutation-subgraph-browser-wallet-trend`

Each snapshot captures the aggregate view needed for the next run:

- totals and cluster movement for coverage
- outcome counts and surviving-mutant movement for mutation

## Local Usage

Fetch the latest stored snapshot when GitHub credentials are available:

```text
cargo run --manifest-path scripts/validation-depth/Cargo.toml -- fetch-previous-artifact --repo <owner/repo> --workflow test-depth.yml --artifact-name test-depth-coverage-trend --output-dir artifacts/coverage/previous --branch <default-branch>
```

Build a coverage trend summary from the current `llvm-cov` output:

```text
cargo run --manifest-path scripts/validation-depth/Cargo.toml -- coverage-trend --current artifacts/coverage/coverage-summary.json --output-md artifacts/coverage/summary.md --output-json artifacts/coverage/coverage-trend.json --previous artifacts/coverage/previous/coverage-trend.json --repo-root .
```

Build a mutation trend summary from a `cargo-mutants` report:

```text
cargo run --manifest-path scripts/validation-depth/Cargo.toml -- mutation-trend --scope core --current target/mutants-report/mutants.out/outcomes.json --output-md artifacts/mutation/summary.md --output-json artifacts/mutation/mutation-trend.json --previous artifacts/mutation/previous/mutation-trend.json --exit-code 0
```
