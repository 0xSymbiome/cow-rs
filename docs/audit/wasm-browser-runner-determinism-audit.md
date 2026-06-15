# WASM Browser Runner Determinism Audit

Status: Current
Last reviewed: 2026-06-10
Owning surface: Headless Firefox runner used by browser-targeted WASM validation lanes
Refresh trigger: Changes to the wasm-pack browser lanes, the pinned geckodriver or Firefox setup actions, or browser-targeted WASM evidence requirements
Related docs:
- [ADR 0007](../adr/0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md)
- [Browser-Runtime Proof Posture](../browser-runtime-proof-posture.md)
- [Validation Scope](../verification.md)

## Scope

This audit covers:

- the headless Firefox runner that the browser-targeted WASM lanes provision
  instead of relying on the ambient runner image
- the browser-wallet bridge tests whose determinism comes from in-test mock
  state rather than a live extension
- the boundary between deterministic browser-wallet automation and manual
  live-extension confirmation

It does not cover vendor wallet extension behavior, live production endpoint
availability, browser support beyond the headless Firefox validation lane, or
the application-specific assertions owned by each WASM console.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Runner provisioning | Browser-targeted WASM lanes provision a headless Firefox runner through pinned setup actions rather than the ambient runner image's browser install | Conforms |
| WebDriver pin | geckodriver is pinned to a fixed version in the workflow; the Firefox browser tracks the `latest-esr` channel rather than a fixed version | Conforms |
| Browser-test determinism | Browser-wallet bridge tests use deterministic mock-wallet state and EIP-6963 serde round trips, so results do not depend on live wallet state | Conforms |
| Browser-wallet live boundary | Live extension checks are excluded from the deterministic lanes and documented as a manual canary with an explicit runbook | Conforms |

## Current Contract

### Headless Firefox Runner

Browser-targeted WASM tests run under headless Firefox. The compatibility
lanes (`.github/workflows/wasm.yml` and
`.github/workflows/browser-wallet-wasm.yml`) install Firefox with
`browser-actions/setup-firefox` on the `latest-esr` channel and geckodriver at
a pinned version with `browser-actions/setup-geckodriver`, then run
`wasm-pack test --headless --firefox`.

Provisioning the runner through these setup actions keeps the browser lanes off
the ambient runner image's drifting browser install and routes `wasm-pack`
through the pinned geckodriver. The browser channel itself is `latest-esr`, so
this lane pins the WebDriver and the provisioning path rather than a fixed
browser build. The lanes use Firefox because Chrome 148 with
wasm-bindgen-test 0.3.x SIGKILLs ChromeDriver mid-handshake on the hosted
runners; the same release-profile binary runs cleanly under Firefox and
geckodriver.

### Browser-Wallet Bridge Determinism

The browser-wallet bridge proof includes deterministic mock-wallet session
transitions and EIP-6963 discovery-event serde round trips. Those are
browser-targeted `wasm_bindgen_test` cases that exercise the bridge against
in-test mock state, so their determinism does not depend on the browser
version, a live extension, or a live chain.

### Live Boundary

Extension-backed checks depend on installed wallet state, authorization
prompts, chain inventory, and vendor-specific behavior, so they remain manual
canary evidence rather than deterministic CI. That acceptance window and its
operator steps are exercised manually and are environment-sensitive.

## Evidence

Primary implementation points:

- `.github/workflows/wasm.yml`
- `.github/workflows/browser-wallet-wasm.yml`

Primary regression coverage:

- `crates/browser-wallet/tests/wasm_bridge_contract.rs`
- `crates/browser-wallet/tests/wasm_bridge_contract.rs::mock_wallet_console_state_machine_is_deterministic`
- `crates/browser-wallet/tests/wasm_bridge_contract.rs::eip6963_discovery_event_serde_roundtrip`
- `crates/wasm/tests/transport_fetch_smoke.rs`

Validation surface:

```text
wasm-pack test --headless --firefox crates/wasm
cd crates/browser-wallet && wasm-pack test --headless --firefox
```
