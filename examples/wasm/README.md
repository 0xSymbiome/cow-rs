# WASM Examples

Standalone WASM examples using `cow-sdk`.

- `sdk-verification-console/`
  - browser verification console for deterministic SDK capabilities plus manual quote/orderbook/subgraph checks
- `browser-wallet-console/`
  - wallet-backed browser console for connect, sign, quote, submit, and cancel flows

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

Python can also be used when preferred:

```text
python -m http.server 8080
```

## WASM Validation

SDK verification console:

```text
cd examples/wasm/sdk-verification-console
wasm-pack test --headless --chrome
```

Quote, orderbook, and subgraph actions remain manual smoke checks.

## GitHub Pages

The `wasm-example-pages` workflow builds both examples, assembles a static Pages
artifact, and deploys it from generated output. Generated `pkg/`, `target/`, and
`dist/` directories are not committed.

Published paths:

```text
https://<owner>.github.io/<repo>/sdk-verification-console/
https://<owner>.github.io/<repo>/browser-wallet-console/
```
