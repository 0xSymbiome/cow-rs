# Browser Wallet Console

Sign, submit, and cancel CoW Protocol orders in a browser with a deterministic
mock wallet or a real injected EIP-1193 provider.

## What this shows

- `Mock Wallet` pane: deterministic proof of message signing, approval
  construction, quote composition, order submission, and off-chain
  cancellation without requiring a browser extension
- `Injected Wallet` pane: explicit EIP-1193 flow for wallet discovery,
  selection, confirmation, connect, sign, live quote, submit, and cancel on
  supported chains
- Chain-coherence gating that blocks live quote, signing, submission, and
  cancellation until the connected wallet session reports the same chain as
  the selected console chain
- Single-wallet auto-confirmation and explicit multi-wallet selection so the
  console behaves honestly whether the visitor has one injected provider or
  many
- `Clear Console Wallet` semantics that only clear console-retained wallet
  selection and session state without claiming to revoke extension
  authorization

## Modes

The two panes separate deterministic proof from environment-sensitive
behavior. The mock wallet is the deterministic contract proof and never
touches a browser extension. The injected wallet pane requires explicit user
authorization and depends on the extension for prompts, chain availability,
and provider-specific behavior. Static browser-live smoke defaults to
`staging`; production browser-live orderbook actions are disabled on the
shipped static page and require a proxy-enabled deployment instead of the
default local or Pages-style serving path. The repository also ships
deterministic browser automation for the injected-wallet pane that uses
local EIP-6963 provider fixtures, route-mocked CoW API responses, and
chain-switch coverage in place of a live wallet extension.

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

## Validation

```text
wasm-pack test --headless --chrome
bun run --cwd ../../../e2e/browser-wallet test
```

After serving the page locally, the static smoke runner confirms the console
is reachable and still exposes the expected injected-wallet markers before
extension-backed actions run:

```text
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- browser-wallet-live --url http://127.0.0.1:8081
```

## Hosted build

```text
https://<owner>.github.io/<repo>/browser-wallet-console/
```

## Related

- [SDK Verification Console](../sdk-verification-console/README.md)
- [Native examples](../../native/README.md)
- [Examples catalogue](../../../docs/examples.md)
- [Browser-runtime proof posture](../../../docs/browser-runtime-proof-posture.md)
