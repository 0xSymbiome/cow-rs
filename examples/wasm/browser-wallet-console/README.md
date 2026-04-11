# Browser Wallet Console

Wallet-backed browser console for `cow-sdk`.

This example keeps browser-wallet support tiers separate:

- `Mock Wallet`: deterministic proof of async wallet signing, approval, quote, submit, and cancel flows through the public SDK without an extension dependency.
- `Injected Wallet`: live EIP-1193 injected-provider flow for connect, sign, quote, submit, and cancel on supported chains.

The injected-provider path requires explicit user authorization and depends on the browser extension for wallet prompts, chain availability, and provider-specific behavior. The mock path is the deterministic contract proof.

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
bunx serve --listen 8081 .
```

Open `http://localhost:8081`.

When deployed through GitHub Pages, open:

```text
https://<owner>.github.io/<repo>/browser-wallet-console/
```
