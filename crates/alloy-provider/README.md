# cow-sdk-alloy-provider

Native Alloy-backed provider adapter package for the `cow-rs` SDK.

This crate is the read-only provider leaf for applications that want the
`cow-rs` provider boundary backed by Alloy on native targets. It is published
as a separate opt-in crate so the default `cow-sdk` facade does not pull native
Alloy provider dependencies.

## Capability Boundary

This crate is native-only. Wasm applications should use
[`cow-sdk-browser-wallet`](https://docs.rs/cow-sdk-browser-wallet) for browser
wallet signing and inject browser RPC access through the supported browser
transport surfaces.

The package boundary is intentionally narrow in this release. Full provider
methods are implemented in the provider adapter crate surface, while the
top-level `cow-sdk` facade only exposes this crate when its native Alloy feature
is enabled.

## Install

```toml
[dependencies]
cow-sdk-alloy-provider = "0.1"
```

## Related Crates

- [`cow-sdk-alloy`](https://docs.rs/cow-sdk-alloy) composes provider and signer
  support behind one native package.
- [`cow-sdk-alloy-signer`](https://docs.rs/cow-sdk-alloy-signer) owns native
  signing support.
- [`cow-sdk`](https://docs.rs/cow-sdk) is the curated facade for most SDK users.

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE) file for
the full text.
