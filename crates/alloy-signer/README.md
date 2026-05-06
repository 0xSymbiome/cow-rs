# cow-sdk-alloy-signer

Native Alloy-backed signing adapter package for the `cow-rs` SDK.

This crate is the signing leaf for applications that want the `cow-rs` signer
boundary backed by Alloy local signing on native targets. It is published as a
separate opt-in crate so the default `cow-sdk` facade does not pull native Alloy
signer dependencies.

## Capability Boundary

This crate is native-only. Wasm applications should use
[`cow-sdk-browser-wallet`](https://docs.rs/cow-sdk-browser-wallet) for browser
wallet signing.

The package boundary is intentionally narrow in this release. Provider access is
owned by [`cow-sdk-alloy-provider`](https://docs.rs/cow-sdk-alloy-provider), and
combined provider plus signer composition is owned by
[`cow-sdk-alloy`](https://docs.rs/cow-sdk-alloy).

## Install

```toml
[dependencies]
cow-sdk-alloy-signer = "0.1"
```

## Related Crates

- [`cow-sdk-alloy`](https://docs.rs/cow-sdk-alloy) composes provider and signer
  support behind one native package.
- [`cow-sdk-alloy-provider`](https://docs.rs/cow-sdk-alloy-provider) owns
  read-only provider support.
- [`cow-sdk`](https://docs.rs/cow-sdk) is the curated facade for most SDK users.

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE) file for
the full text.
