# SDK Verification Console

Verify the `cow-sdk` facade, app-data pipeline, CID roundtrip, orderbook
helpers, and subgraph escape hatch behave identically under
`wasm32-unknown-unknown` as under native Rust.

## What this shows

- Deterministic capability, app-data, CID, order-envelope, EIP-1271, approval,
  and trading-default outputs exercised through a browser WebAssembly bundle
- The typed orderbook transport and subgraph GraphQL escape hatch inspected
  from a real browser without bypassing the reviewed Rust SDK
- Manual-network panes (`Latest Competition`, `Order`, `Order Trades`,
  `App Data`, `Quote`, `Subgraph Totals`) that call public endpoints only
  when the reviewer clicks them
- Supported-chain identity, wrapped-native metadata, and settlement
  deployment data rendered for each reviewed chain and environment pairing
- Sample order and approval inputs seeded so reviewers can verify typed-data
  structure, calldata shape, and app-data roundtrip without composing
  protocol fields by hand

## Modes

The panel split separates deterministic proof from optional network probes.
Capability, app-data, CID, order-envelope, EIP-1271, approval, and
trading-default outputs run entirely inside the WebAssembly bundle and stay
deterministic for the same inputs. Quote, orderbook, and subgraph panes call
network-style APIs only when the reviewer clicks a button. Static
browser-live smoke defaults to `staging`; production browser-live orderbook
actions are disabled on the shipped static page and require a proxy-enabled
deployment instead of the default local or Pages-style serving path.

## Build

```text
wasm-pack build --target web
```

## Serve

Serve this directory over HTTP, for example:

```text
bunx serve . --listen 8080
python -m http.server 8080
```

Open [http://localhost:8080](http://localhost:8080). Do not open `index.html`
with `file://`.

## Validation

```text
wasm-pack test --headless --chrome
```

## Hosted build

```text
https://<owner>.github.io/<repo>/sdk-verification-console/
```

## Related

- [Browser Wallet Console](../browser-wallet-console/README.md)
- [Native examples](../../native/README.md)
- [Examples catalogue](../../../docs/examples.md)
- [Browser-runtime proof posture](../../../docs/browser-runtime-proof-posture.md)
