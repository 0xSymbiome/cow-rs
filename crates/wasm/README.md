# cow-sdk-wasm

TypeScript-callable wasm-bindgen leaf for the CoW Protocol Rust SDK.

This crate exposes deterministic CoW Protocol logic — order signing,
EIP-1271 envelope construction, app-data hashing, orderbook,
subgraph, IPFS, trading — to JavaScript and TypeScript consumers
through typed DTOs and explicit JS callbacks for wallet, signer, and
HTTP transport. It wraps the existing native SDK helpers; it does
not reimplement any protocol primitive.

## Runtime support

| Runtime | Support tier | HTTP transport |
| --- | --- | --- |
| Browser bundlers (Vite, webpack, Next.js, Rollup, Parcel, esbuild) | Default-supported | Default `fetch` |
| Node.js (Active LTS) | Tested | JS callback transport |
| Cloudflare Workers (workerd) | Tested | JS callback transport |
| Deno | Optional / experimental | JS callback transport |
| Bun, Vercel Edge Functions, Fly.io Machines | Best effort | JS callback transport |

The `Default-supported` tier uses `cow-sdk-transport-wasm`, which
requires a `Window`-scoped `fetch`. The `Tested` and
`Optional / experimental` tiers route HTTP through the
`JsCallbackHttpTransport`, an explicit JavaScript callback so the
host runtime owns request dispatch.

## Install

The canonical npm package is published from the staging tree under
`crates/wasm/npm/`. The package name is selected at publish time;
consult the project release notes for the current name.

```bash
# Replace <package-name> with the published npm package name.
npm install <package-name>
```

For the Rust crate:

```toml
[dependencies]
cow-sdk-wasm = "0.1"
```

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Integrations Guide](https://github.com/cowdao-grants/cow-rs/blob/main/docs/integrations.md)
- [Architecture Overview](https://github.com/cowdao-grants/cow-rs/blob/main/docs/architecture.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
