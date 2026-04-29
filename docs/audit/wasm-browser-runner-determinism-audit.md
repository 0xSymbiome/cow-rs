# WASM Browser Runner Determinism Audit

Status: Current
Last reviewed: 2026-04-29
Owning surface: Pinned Chrome-for-Testing runner used by browser-targeted WASM validation lanes
Refresh trigger: Changes to the pinned WASM browser runner config, Chrome-for-Testing refresh cadence, wasm-runner setup or freshness commands, wasm-pack workflow lanes, or browser-targeted WASM evidence requirements
Related docs:
- [ADR 0007](../adr/0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md)
- [ADR 0009](../adr/0009-wasm-verification-consoles-hybrid-extensibility-and-two-tier-proof.md)
- [WASM Example Proof-Posture Audit](wasm-example-proof-posture-audit.md)
- [Browser-Runtime Proof Posture](../browser-runtime-proof-posture.md)
- [Validation Scope](../validation-scope.md)
- [Verification Matrix](../verification-matrix.md)

## Scope

This audit covers:

- the committed Chrome-for-Testing pin that supplies the browser and
  chromedriver versions for WASM browser tests
- the setup command that downloads the pinned browser runner and writes the
  webdriver configuration consumed by `wasm-pack test`
- the release-readiness freshness check that rejects stale pins before release
- the workflow posture that runs browser-targeted WASM tests against the
  pinned runner instead of ambient runner images

It does not cover vendor wallet extension behavior, live production endpoint
availability, browser support beyond the pinned headless Chrome validation
lane, or the application-specific assertions owned by each WASM console.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Runner pin | The committed WASM browser runner config records one Chrome-for-Testing stable version, release timestamp, platform URLs, and platform checksums | Conforms |
| Runner setup | Workflow lanes invoke `cargo wasm-runner-setup` before browser-targeted `wasm-pack test` steps and pass the generated webdriver configuration through `WEBDRIVER_JSON` | Conforms |
| Freshness gate | Release-readiness runs `cargo check-wasm-runner-freshness` and blocks release when the pin falls outside the accepted age window | Conforms |
| Browser-test determinism | WASM compatibility lanes no longer rely on the hosted runner image's ambient Chrome or chromedriver installation | Conforms |
| Refresh path | The public refresh command can regenerate the pin from Chrome-for-Testing metadata while preserving the checksum-bearing config shape | Conforms |

## Current Contract

### Pinned Runner

`.github/config/wasm-test-versions.yaml` is the committed browser-runner
authority for WASM browser tests. It records Chrome-for-Testing Stable
`148.0.7778.56`, released on `2026-04-28T20:36:36.653Z`, plus platform-specific
Chrome and chromedriver archive URLs and SHA-256 checksums for Linux, macOS,
and Windows.

The config is intended to move deliberately: it is refreshed at every `0.x.0`
release candidate and any time release-readiness would otherwise see a pin
older than the accepted freshness window.

### Setup Command

`cargo wasm-runner-setup --webdriver-json <path>` reads the committed pin,
downloads the platform-specific Chrome and chromedriver archives, verifies the
checksums, extracts the binaries under the build target directory, and writes a
webdriver JSON report. Browser-targeted workflow steps pass the same report
path through `WEBDRIVER_JSON`, which lets `wasm-bindgen-test` discover the
pinned runner consistently.

The WASM compatibility workflow runs setup immediately before every
browser-targeted `wasm-pack test` step for browser-wallet, transport-wasm,
trading, signing, and both WASM verification consoles. The browser-wallet E2E
workflow uses the same setup command before its direct bridge and console
browser tests.

### Freshness Gate

`cargo check-wasm-runner-freshness` is part of release-readiness. It reads the
same committed pin and rejects stale or malformed release timestamps before a
release candidate can pass. This keeps the runner reproducible without letting
the pinned browser fall silently behind current Chrome-for-Testing releases.

### Refresh Path

`cargo wasm-runner-refresh --source online --output .github/config/wasm-test-versions.yaml`
is the public refresh path. It reads the current Chrome-for-Testing Stable
metadata, resolves platform downloads, hashes archives when needed, and writes
the checksum-bearing YAML used by setup and freshness validation.

## Evidence

Primary implementation points:

- `.github/config/wasm-test-versions.yaml`
- `.github/workflows/wasm.yml`
- `.github/workflows/browser-wallet-e2e.yml`
- `.github/workflows/release-readiness.yml`
- `scripts/validation-smoke/src/wasm_runner.rs`
- `scripts/policy-maintainer/src/check_wasm_runner_freshness.rs`

Primary regression coverage:

- `scripts/validation-smoke/tests/wasm_runner.rs`
- `scripts/policy-maintainer/tests/check_wasm_runner_freshness.rs`
- `crates/browser-wallet/tests/wasm_bridge_contract.rs`
- `crates/transport-wasm/tests/wasm.rs`
- `examples/wasm/browser-wallet-console/tests/wasm_deterministic.rs`
- `examples/wasm/sdk-verification-console/tests/deterministic_exports.rs`

Validation surface:

```text
cargo wasm-runner-refresh --source online --output .github/config/wasm-test-versions.yaml
cargo wasm-runner-setup --webdriver-json target/wasm-runner/webdriver.json
cargo check-wasm-runner-freshness
cd crates/browser-wallet && wasm-pack test --headless --chrome
cd examples/wasm/sdk-verification-console && wasm-pack test --headless --chrome
```
