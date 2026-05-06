# cow-sdk-alloy

Native composed Alloy adapter package for the `cow-rs` SDK.

This crate is the umbrella package for applications that want native Alloy
provider and signer support through one opt-in dependency. It re-exports the
leaf package namespaces while keeping the default `cow-sdk` facade free of
native Alloy runtime dependencies.

## Capability Boundary

This crate is native-only. Wasm applications should use
[`cow-sdk-browser-wallet`](https://docs.rs/cow-sdk-browser-wallet) for browser
wallet signing and inject browser RPC access through the supported browser
transport surfaces.

The package boundary is intentionally narrow in this release. Read-only provider
support is owned by
[`cow-sdk-alloy-provider`](https://docs.rs/cow-sdk-alloy-provider), signing
support is owned by [`cow-sdk-alloy-signer`](https://docs.rs/cow-sdk-alloy-signer),
and this package is the composed namespace for consumers that want both.
`AlloyClient` implements `AsyncProvider` and `AsyncSigningProvider`; the owned
signer handle returned by `create_signer` implements `AsyncSigner`, signs CoW
EIP-712 typed-data payloads directly, submits transactions through Alloy's
wallet-filler provider, and reports the broadcast transaction hash.

## Install

```toml
[dependencies]
cow-sdk-alloy = "0.1"
```

## Quick Start

```rust,no_run
use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::SupportedChainId;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = AlloyClient::builder()
    .http("https://example.invalid/rpc")?
    .private_key("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")?
    .chain_id(SupportedChainId::Sepolia)
    .build()
    .await?;
# let _ = client;
# Ok(())
# }
```

Raw `sign_transaction` is intentionally unsupported in this release because
the relevant Alloy provider path asks the remote JSON-RPC peer to sign. Use
`send_transaction` for wallet-filler submission or the signer leaf for local
message and typed-data signatures.

## Related Crates

- [`cow-sdk-alloy-provider`](https://docs.rs/cow-sdk-alloy-provider) owns
  read-only provider support.
- [`cow-sdk-alloy-signer`](https://docs.rs/cow-sdk-alloy-signer) owns native
  signing support.
- [`cow-sdk`](https://docs.rs/cow-sdk) is the curated facade for most SDK users.

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE) file for
the full text.
