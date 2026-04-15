# WASM Examples

Standalone WASM examples for `cow-rs`.

If you are new to `cow-rs`, start with
[Getting Started](../../docs/getting-started.md) first. Come to the WASM
surfaces after the deterministic native onboarding flow is clear or when you
specifically need browser-hosted verification.

## Surfaces

| Example | Purpose |
| --- | --- |
| `sdk-verification-console/` | Deterministic SDK verification and browser inspection for WASM-compatible surfaces |
| `browser-wallet-console/` | Mock-wallet proof plus explicit injected-wallet flows for browser-runtime support |

`cow-sdk` remains the default facade for pure SDK flows. Browser-wallet support
is additive behind the `browser-wallet` feature and is intentionally separated
from the deterministic native onboarding path.

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
