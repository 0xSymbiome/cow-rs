# Browser Wallet Console

Wallet-backed browser console for `cow-sdk`.

This example keeps browser-wallet support tiers separate:

- `Mock Wallet`: deterministic proof of signing, approval, quote, submit, and
  cancel flows without an extension dependency
- `Injected Wallet`: explicit EIP-1193 flow for connect, sign, quote, submit,
  and cancel on supported chains

The injected-provider path requires explicit user authorization and depends on
the browser extension for prompts, chain availability, and provider-specific
behavior. The mock path is the deterministic contract proof.

When exactly one injected wallet is detected, the console auto-confirms that
provider instead of requiring a redundant manual confirmation step. Live quote,
order signing, submission, and cancellation stay gated until the connected
wallet session reports the same chain as the selected console chain.

`Clear Console Wallet` only clears console-retained wallet selection and
session state. Extension authorization remains managed by the wallet UI.

Static browser-live smoke checks default to `staging`. Production browser-live
orderbook calls are disabled on the shipped static page and require a
proxy-enabled deployment instead of the default local or Pages-style serving
path.

The repository also includes deterministic browser automation for the
injected-wallet pane. That automation uses local EIP-6963 provider fixtures,
route-mocked CoW API responses, chain-switch coverage, and stable DOM markers
instead of a live wallet extension or public endpoint.

## Build

```text
wasm-pack build --target web
```

## Serve

Serve this directory over HTTP, for example:

```text
bunx serve --listen 8081 .
python -m http.server 8081
```

Open [http://localhost:8081](http://localhost:8081).

## Optional Smoke Check

After serving the page locally, use the smoke runner to confirm that the
console is reachable and still exposes the expected stable injected-wallet
markers before performing extension-backed actions:

```text
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- browser-wallet-live --url http://127.0.0.1:8081
```

That check verifies page readiness only. It does not automate extension-backed
wallet authorization.

## Deployed Page

```text
https://<owner>.github.io/<repo>/browser-wallet-console/
```
