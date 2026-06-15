# cow-sdk-alloy-provider

Native Alloy-backed read-only provider adapter for the `cow-rs` SDK.

> ⚠️ **Alpha — `0.1.0-alpha`.** Pre-release and not security-audited; the public
> API may change before `0.1.0`. It is published as a pre-release, so Cargo
> selects it only when you opt in (`cow-sdk-alloy-provider = "0.1.0-alpha.1"`).
> Review it yourself before relying on it with real funds.

This crate wraps an Alloy HTTP RPC provider and exposes it through
`cow_sdk_core::Provider`. It is intentionally read-only: it does not
create signers, sign messages, sign transactions, or submit transactions.

## Install

```toml
[dependencies]
cow-sdk-alloy-provider = "0.1.0-alpha.1"
```

## Build A Provider

```rust,no_run
use std::time::Duration;

use cow_sdk_alloy_provider::RpcAlloyProvider;
use cow_sdk_core::Provider;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let provider = RpcAlloyProvider::builder()
    .timeout(Duration::from_secs(20))
    .http("https://example.invalid/rpc")?
    .build()?;

let chain_id = provider.get_chain_id().await?;
# let _ = chain_id;
# Ok(())
# }
```

The builder stores the RPC URL behind `cow_sdk_core::Redacted` before the
transport state becomes visible through debug output. Invalid URLs return a
typed builder error without echoing the supplied value.

## Opt-In Retry

By default the provider issues each RPC request once and surfaces a transient
transport failure — such as a public-endpoint `429 Too Many Requests` — directly
to the caller. This keeps the default runtime-neutral: the consumer owns
chain-RPC resilience.

To opt into transparent retries for transient, rate-limited reads, pass a
`RetryConfig` to `retry`. It wraps the JSON-RPC client in a bounded
exponential backoff layer:

```rust,no_run
use cow_sdk_alloy_provider::{RetryConfig, RpcAlloyProvider};
use cow_sdk_core::Provider;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let provider = RpcAlloyProvider::builder()
    .http("https://example.invalid/rpc")?
    .retry(RetryConfig::default())
    .build()?;

let chain_id = provider.get_chain_id().await?;
# let _ = chain_id;
# Ok(())
# }
```

The policy retries only rate-limit-class transport errors and never
re-broadcasts a transaction, so write nonce-safety is unaffected. The same
`retry` setter is available on the composed `cow-sdk-alloy` `AlloyClient`
builder.

## Capability Boundary

`RpcAlloyProvider` implements `Provider` only. The crate provides all
read methods required by the core trait:

- `get_chain_id`
- `get_code`
- `get_transaction_receipt`
- `call`
- `read_contract`
- `get_block`

`read_contract` parses the supplied JSON ABI, resolves a single non-overloaded
function, ABI-encodes JSON arguments with `alloy-dyn-abi`, dispatches
`eth_call`, decodes the output, and returns the JSON value string expected by
`cow_sdk_core::ContractCall`.

`get_transaction_receipt` maps Alloy receipts into
`cow_sdk_core::TransactionReceipt`. The adapter populates the transaction hash,
EIP-658 success / reverted status when present, block number, block hash, gas
used, sender, and recipient. Contract creation keeps `to` empty, and
pre-Byzantium post-state receipts keep `status` empty rather than coercing the
post-state root into a success value.

## Native Only

This crate hard-fails on `wasm32` targets. Browser applications should use
`cow-sdk-browser-wallet` for wallet-backed signing and provide RPC access
through the browser's EIP-1193 provider surface.

The compile-time failure is deliberate. It keeps browser builds on the
browser-wallet adapter and fails with a direct SDK message instead of allowing
native Alloy HTTP transport dependencies to fail later with platform-specific
errors.

## Companion Crates

- `cow-sdk-alloy-signer` owns native local-key signing support.
- `cow-sdk-alloy` composes provider and signer support for consumers that want
  one native Alloy client.
- `cow-sdk` remains the curated facade and exposes Alloy support only when the
  matching opt-in features are enabled.

## Error Model

Provider failures use `ProviderError`. Transport details are redacted,
remote JSON-RPC errors preserve their code and message, caller input failures
are typed as validation errors, and cooperative cancellation propagates through
the `Cancelled` variant when callers use `cow_sdk_core::Cancellable`.

## License

Licensed under GPL-3.0-or-later. See the workspace
[LICENSE](https://github.com/0xSymbiome/cow-rs/blob/main/LICENSE) file for
the full text.
