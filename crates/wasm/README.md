# cow-sdk-wasm

TypeScript-callable wasm-bindgen bindings for the CoW Protocol Rust
SDK. The crate exposes deterministic protocol logic to JavaScript and
TypeScript consumers through typed DTOs and explicit JS callbacks for
wallet, signer, and HTTP transport while reusing the existing native
SDK helpers.

## Runtime support

| Runtime | Support tier | HTTP transport |
| --- | --- | --- |
| Browser bundlers (Vite, webpack, Next.js, Rollup, Parcel, esbuild) | Default-supported | Default `fetch` |
| Node.js 24 LTS | Tested | JS callback transport |
| Cloudflare Workers (workerd) | Tested | JS callback transport |
| Deno | Opt-in / experimental | JS callback transport |

The `Default-supported` tier uses `cow-sdk-transport-wasm`, which
requires a `Window`-scoped `fetch`. The `Tested` and
`Optional / experimental` tiers route HTTP through the
`JsCallbackHttpTransport`, an explicit JavaScript callback so the
host runtime owns request dispatch.

## Install

The package name is selected at publish time and resolved by the
package rendering script.

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
- [WASM Surface Audit](https://github.com/cowdao-grants/cow-rs/blob/main/docs/audit/cow-sdk-wasm-surface-audit.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
