# WASM Browser Runner Determinism Audit

Status: Current
Last reviewed: 2026-06-03
Owning surface: Pinned Chrome-for-Testing runner used by browser-targeted WASM validation lanes
Refresh trigger: Changes to the pinned WASM browser runner config, Chrome-for-Testing refresh cadence, wasm-runner setup or freshness commands, wasm-pack workflow lanes, or browser-targeted WASM evidence requirements
Related docs:
- [ADR 0007](../adr/0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md)
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
- the boundary between deterministic browser-wallet automation and manual
  live-extension confirmation

It does not cover vendor wallet extension behavior, live production endpoint
availability, browser support beyond the pinned headless Chrome validation
lane, or the application-specific assertions owned by each WASM console.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Runner pin | The committed WASM browser runner config records one Chrome-for-Testing stable version, release timestamp, platform URLs, and platform checksums | Conforms |
| Runner setup | The browser runner is provisioned directly by the wasm-pack browser lanes, which pin Chrome-for-Testing to the committed config before browser-targeted `wasm-pack test` steps | Conforms |
| Freshness gate | Release-readiness runs `cargo check-wasm-runner-freshness` and blocks release when the pin falls outside the accepted age window | Conforms |
| Browser-test determinism | WASM compatibility lanes no longer rely on the hosted runner image's ambient Chrome or chromedriver installation | Conforms |
| Browser-wallet bridge proof | Browser-wallet WASM bridge tests run in a headless browser and include deterministic mock-wallet state plus EIP-6963 event serialization coverage | Conforms |
| Browser-wallet live boundary | Live extension checks are excluded from the deterministic lanes and documented as a manual canary with an explicit runbook | Conforms |
| Refresh path | The pin is refreshed by hand from current Chrome-for-Testing metadata while preserving the checksum-bearing config shape | Conforms |

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

### Runner Provisioning

The browser runner is provisioned directly by the wasm-pack browser lanes.
Each lane reads the committed pin, installs the platform-specific
Chrome-for-Testing Chrome and chromedriver at the recorded version with verified
checksums, and points `wasm-bindgen-test` at the pinned chromedriver so the
runner stays consistent across runs.

The WASM compatibility workflow provisions the runner immediately before its
browser-targeted `wasm-pack test` steps for the WASM-facing SDK crates.

The browser-wallet bridge proof includes deterministic mock-wallet session
transitions and EIP-6963 discovery-event serialization round trips. Those
tests are browser-targeted `wasm_bindgen_test` cases that run against a pinned
browser runner to avoid ambient driver drift.

Extension-backed checks depend on installed wallet state, authorization
prompts, chain inventory, and vendor-specific behavior, so they remain manual
canary evidence rather than deterministic CI. That acceptance window and its
operator steps are exercised manually and are environment-sensitive.

### Freshness Gate

`cargo check-wasm-runner-freshness` is part of release-readiness. It reads the
same committed pin and rejects stale or malformed release timestamps before a
release candidate can pass. This keeps the runner reproducible without letting
the pinned browser fall silently behind current Chrome-for-Testing releases.

### Refresh Path

`.github/config/wasm-test-versions.yaml` is refreshed by hand from the current
Chrome-for-Testing Stable metadata: resolve the platform downloads, hash the
archives, and update the checksum-bearing YAML used by runner provisioning and
freshness validation.

## Evidence

Primary implementation points:

- `.github/config/wasm-test-versions.yaml`
- `.github/workflows/wasm.yml`
- `.github/workflows/browser-wallet-wasm.yml`
- `.github/workflows/release-readiness.yml`
- `scripts/validation-smoke/src/wasm_runner.rs`
- `scripts/policy-maintainer/src/check_wasm_runner_freshness.rs`

Primary regression coverage:

- `scripts/validation-smoke/tests/wasm_runner.rs`
- `scripts/policy-maintainer/tests/check_wasm_runner_freshness.rs`
- `crates/browser-wallet/tests/wasm_bridge_contract.rs`
- `crates/browser-wallet/tests/wasm_bridge_contract.rs::mock_wallet_console_state_machine_is_deterministic`
- `crates/browser-wallet/tests/wasm_bridge_contract.rs::eip6963_discovery_event_serde_roundtrip`
- `crates/transport-wasm/tests/wasm.rs`

Validation surface:

```text
cargo check-wasm-runner-freshness
cd crates/browser-wallet && wasm-pack test --headless --chrome --chromedriver <chromedriver matching .github/config/wasm-test-versions.yaml>
```
