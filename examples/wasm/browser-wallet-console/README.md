# Browser Wallet Console

Wallet-backed browser console for `cow-sdk`.

This example keeps browser-wallet support tiers separate:

- `Mock Wallet`: deterministic proof of async wallet signing, approval, quote, submit, and cancel flows through the public SDK without an extension dependency.
- `Injected Wallet`: explicit EIP-1193 injected-provider flow for connect, sign, quote, submit, and cancel on supported chains.

The injected-provider path requires explicit user authorization and depends on the browser extension for wallet prompts, chain availability, and provider-specific behavior. The mock path is the deterministic contract proof.

The repository also includes a deterministic browser automation lane for the injected-wallet pane. That lane uses local EIP-6963 provider fixtures, route-mocked CoW API responses, and stable DOM markers in the console instead of a live wallet extension or public endpoint.

`Detect` caches discovered injected-wallet candidates. When more than one candidate is present, `Confirm Wallet` records the provider the console is allowed to use. `Connect / Reconnect` uses that confirmed provider or the retained selected wallet handle. `Rescan` performs a fresh discovery round and either revalidates or clears the confirmed provider choice. `Reset Session` clears console session state while keeping the selected wallet and confirmed provider available for status and refresh actions. `Forget Wallet` clears both from the console. Wallet authorization stays managed by the extension.

The page keeps `Mock Wallet` and `Injected Wallet` output panes separate, renders each action result as JSON, and exposes a stable injected-wallet contract-state panel for browser automation and human inspection.

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
