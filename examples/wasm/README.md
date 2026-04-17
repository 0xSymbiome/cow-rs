# WASM Examples

Standalone WASM examples for `cow-rs`.

If you are new to `cow-rs`, start with
[Getting Started](../../docs/getting-started.md) first. Come to the WASM
surfaces after the deterministic native onboarding flow is clear or when you
specifically need browser-hosted verification.

## Surfaces

| Console | Package | Purpose |
| --- | --- | --- |
| `sdk-verification-console/` | `cow-sdk-verification-console` | Deterministic SDK verification and browser inspection for WASM-compatible surfaces |
| `browser-wallet-console/` | `cow-sdk-browser-wallet-console-wasm` | Mock-wallet proof plus explicit injected-wallet flows for browser-runtime support |

`cow-sdk` remains the default facade for pure SDK flows. Browser-wallet support
is additive behind the `browser-wallet` feature and is intentionally separated
from the deterministic native onboarding path.

Each console is self-contained and served from its own directory. The palette,
layout, and polish primitives live inline in each console's `index.html` behind
a `shared polish primitives — keep synchronized across consoles` comment marker
so the rendered surface works unchanged under every serve flow (per-console
`bunx serve`, `python -m http.server`, and the Playwright `serve:console`
command). When both consoles need the same primitive, update the marker block
in both files in the same change so the two consoles stay visually identical
on the polish surface.

## Local Use

Build and serve each example from its own directory:

```text
cd examples/wasm/sdk-verification-console
wasm-pack build --target web
bunx serve . --listen 8080
```

```text
cd examples/wasm/browser-wallet-console
wasm-pack build --target web
bunx serve . --listen 8081
```

Open the served HTTP URL. Browsers do not load the generated WASM modules from
`file://` origins.

## Validation

SDK verification console:

```text
cd examples/wasm/sdk-verification-console
wasm-pack test --headless --chrome
```

Browser-wallet console:

```text
bun run --cwd e2e/browser-wallet test
```

## GitHub Pages

Published paths:

```text
https://<owner>.github.io/<repo>/sdk-verification-console/
https://<owner>.github.io/<repo>/browser-wallet-console/
```
