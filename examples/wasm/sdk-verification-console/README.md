# SDK Verification Console

Browser console for deterministic `cow-sdk` WASM verification.

The local capability, app-data, CID, order-envelope, EIP-1271, approval, and
trading-default outputs are deterministic. Quote, orderbook, and subgraph
controls call network-style APIs only when used manually.

Static browser-live smoke checks default to `staging`. Production browser-live
orderbook calls are disabled on the shipped static page and require a
proxy-enabled deployment instead of the default local or Pages-style serving
path.

The orderbook pane no longer ships stale live lookup defaults. `Latest
Competition` seeds the order UID, owner, and app-data hash from a supported
current-network endpoint before `Order`, `Order Trades`, and `App Data` become
available. The EIP-1271 control starts with a valid sample signature and is
gated again if that signature is cleared or made invalid.

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

Open [http://localhost:8080](http://localhost:8080). Do not open
`index.html` with `file://`.

## Validation

```text
wasm-pack test --headless --chrome
```

## Deployed Page

```text
https://<owner>.github.io/<repo>/sdk-verification-console/
```
