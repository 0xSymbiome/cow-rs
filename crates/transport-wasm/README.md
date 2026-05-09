# cow-sdk-transport-wasm

WebAssembly browser transport adapter for cow-sdk. Implements the
`cow_sdk_core::transport::HttpTransport` trait via the browser `fetch`
API.

This crate owns browser HTTP dispatch only. Use it when a
`wasm32-unknown-unknown` integration needs `FetchTransport` without
pulling native HTTP defaults into the browser build.

JavaScript and TypeScript consumers that need Node.js, Cloudflare Workers,
Deno, or custom HTTP control should use `cow-sdk-wasm` and its
`JsCallbackHttpTransport` instead. That callback transport is the
runtime-neutral peer to this browser-only fetch adapter.

## Install

```toml
[dependencies]
cow-sdk-transport-wasm = "0.1"
```

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Transport Guide](https://github.com/cowdao-grants/cow-rs/blob/main/docs/transport.md)
- [cow-sdk-wasm README](https://github.com/cowdao-grants/cow-rs/blob/main/crates/wasm/README.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
