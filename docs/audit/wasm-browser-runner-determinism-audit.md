# WASM Browser Runner Determinism Audit

Status: Current
Last reviewed: 2026-06-16
Owning surface: Headless Firefox runner used by browser-targeted WASM validation lanes
Refresh trigger: Changes to the wasm-pack browser lanes, the pinned geckodriver or Firefox setup actions, or browser-targeted WASM evidence requirements
Related docs:
- [ADR 0039](../adr/0039-typescript-callable-wasm-sdk-surface.md)
- [Validation Scope](../verification.md)

## Scope

This audit covers:

- the headless Firefox runner that the browser-targeted WASM lanes provision
  instead of relying on the ambient runner image
- the `cow-sdk-wasm` browser-targeted tests whose determinism comes from
  in-test mock state and serde round trips rather than a live host environment

It does not cover host wallet stack behavior, live production endpoint
availability, browser support beyond the headless Firefox validation lane, or
the application-specific assertions owned by each WASM console.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Runner provisioning | Browser-targeted WASM lanes provision a headless Firefox runner through pinned setup actions rather than the ambient runner image's browser install | Conforms |
| WebDriver pin | geckodriver is pinned to a fixed version in the workflow; the Firefox browser tracks the `latest-esr` channel rather than a fixed version | Conforms |
| Browser-test determinism | The `cow-sdk-wasm` browser-targeted tests use deterministic in-test state and serde round trips, so results do not depend on live host state | Conforms |

## Current Contract

### Headless Firefox Runner

Browser-targeted WASM tests run under headless Firefox. The compatibility
lane (`.github/workflows/wasm.yml`) installs Firefox with
`browser-actions/setup-firefox` on the `latest-esr` channel and geckodriver at
a pinned version with `browser-actions/setup-geckodriver`, then run
`wasm-pack test --headless --firefox`.

Provisioning the runner through these setup actions keeps the browser lane off
the ambient runner image's drifting browser install and routes `wasm-pack`
through the pinned geckodriver. The browser channel itself is `latest-esr`, so
this lane pins the WebDriver and the provisioning path rather than a fixed
browser build. The lane uses Firefox because Chrome 148 with
wasm-bindgen-test 0.3.x SIGKILLs ChromeDriver mid-handshake on the hosted
runners; the same release-profile binary runs cleanly under Firefox and
geckodriver.

### Browser-Target Test Determinism

The `cow-sdk-wasm` browser-targeted tests run as `wasm_bindgen_test` cases
that exercise the crate's callback boundary against in-test state and serde
round trips, so their determinism does not depend on the browser version, a
live host wallet, or a live chain.

## Evidence

Primary implementation points:

- `.github/workflows/wasm.yml`

Primary regression coverage:

- `crates/wasm/tests/transport_fetch_smoke.rs`

Validation surface:

```text
wasm-pack test --headless --firefox crates/wasm
```
