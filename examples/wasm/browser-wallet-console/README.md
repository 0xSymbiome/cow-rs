# Browser Wallet Console

Wallet-backed browser console for `cow-sdk`.

It keeps two paths separate:

- `Mock Wallet`: deterministic proof of async wallet signing, approval, quote, submit, and cancel flows through the public SDK.
- `Injected Wallet`: real `window.ethereum` flow for connect, sign, quote, submit, and cancel on supported chains.

`Reset Session` clears local browser-console state. Wallet authorization stays managed by the extension.

Build:

```text
wasm-pack build --target web
```

Serve this directory over HTTP, for example:

```text
python -m http.server 8081
```

```text
bunx serve . --listen 8081
```

Open `http://localhost:8081`.
