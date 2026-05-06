# Adapting Alloy

`cow-rs` ships native Alloy adapters as opt-in crates. Use them when a
native application wants Alloy-backed chain RPC, local private-key signing, or
both through the same client.

## Crates And Features

| Need | Crate or facade feature |
| --- | --- |
| Read-only chain RPC through `AsyncProvider` | `cow-sdk-alloy-provider` or `cow-sdk` feature `alloy-provider` |
| Local private-key signing through `AsyncSigner` | `cow-sdk-alloy-signer` or `cow-sdk` feature `alloy-signer` |
| Composed provider plus signer for `TradingSdk` async helpers | `cow-sdk-alloy` or `cow-sdk` feature `alloy` |

The default `cow-sdk` facade remains provider-neutral. Native Alloy support is
explicitly enabled by feature or direct crate dependency and is not available
on `wasm32-unknown-unknown`; browser applications should use
`cow-sdk-browser-wallet` for signing and inject browser RPC access through the
browser runtime.

## Umbrella Client

`cow-sdk-alloy::AlloyClient` combines an Alloy HTTP provider with an Alloy
local private-key signer. The client implements `AsyncProvider` and
`AsyncSigningProvider`; the owned signer handle returned by `create_signer`
implements `AsyncSigner` and remains usable after the parent client is
dropped.

```rust,no_run
use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::{AsyncSigningProvider, SupportedChainId};

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

The umbrella signer handle signs CoW EIP-712 typed-data payloads directly,
normalizes ECDSA `v` values through the shared contracts helper, sends
transactions through Alloy's wallet-filler provider, and returns the broadcast
transaction hash. Raw `sign_transaction` is intentionally unsupported because
the relevant Alloy provider path delegates to the remote JSON-RPC peer rather
than producing a local signed payload.

## Leaf Adapters

Use the provider leaf when a flow needs read-only chain RPC without a signer:

```rust,no_run
use cow_sdk_alloy_provider::RpcAlloyProvider;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let provider = RpcAlloyProvider::builder()
    .http("https://example.invalid/rpc")?
    .build()
    .await?;
# let _ = provider;
# Ok(())
# }
```

Use the signer leaf when a flow needs local message or typed-data signing but
does not need provider-backed transaction submission:

```rust,no_run
use cow_sdk_alloy_signer::LocalAlloyKeystoreSigner;
use cow_sdk_core::SupportedChainId;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let signer = LocalAlloyKeystoreSigner::builder()
    .private_key("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")?
    .chain_id(SupportedChainId::Sepolia)
    .build()?;
# let _ = signer;
# Ok(())
# }
```

The signer leaf returns provider-required errors for transaction submission and
gas estimation because it does not own RPC state. Use the umbrella client when
the same runtime must both sign and submit transactions.

## Trading Integration

The async helper paths on `TradingSdk` accept the umbrella through the core
traits. Typical uses include allowance reads, approval submission, pre-sign
transaction construction, and on-chain cancellation.

```rust,no_run
use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::SupportedChainId;
use cow_sdk_trading::TradingSdk;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = AlloyClient::builder()
    .http("https://example.invalid/rpc")?
    .private_key("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")?
    .chain_id(SupportedChainId::Sepolia)
    .build()
    .await?;

let sdk = TradingSdk::builder()
    .with_chain_id(SupportedChainId::Sepolia)
    .build_helper_only()?;

let signer = client.create_signer("local").await?;
# let _ = (sdk, signer);
# Ok(())
# }
```

For runnable scenarios, see the Alloy examples listed in
[`examples/native/README.md`](../../examples/native/README.md).
