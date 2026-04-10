# SDK Verification Console

Browser console for `cow-sdk` WASM verification.

The local capability, app-data, CID, order-envelope, EIP-1271, approval, and
trading-default outputs are deterministic. Quote, orderbook, and subgraph
controls call network-style APIs only when used manually.

Build:

```text
wasm-pack build --target web
```

Serve this directory over HTTP, for example:

```text
bunx serve . --listen 8080
```

```text
python -m http.server 8080
```

Open [http://localhost:8080](http://localhost:8080).

Do not open `index.html` with `file://`.

WASM validation:

```text
wasm-pack test --headless --chrome
```

Live quote, orderbook, and subgraph checks are manual smoke checks. Subgraph
manual checks require a The Graph API key.

When deployed through GitHub Pages, open:

```text
https://<owner>.github.io/<repo>/sdk-verification-console/
```
