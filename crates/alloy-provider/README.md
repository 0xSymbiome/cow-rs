# cow-sdk-alloy-provider

Native Alloy-backed read-only provider adapter for the `cow-rs` SDK.

This crate wraps an Alloy HTTP RPC provider and exposes it through
`cow_sdk_core::AsyncProvider`. It is intentionally read-only: it does not
create signers, sign messages, sign transactions, or submit transactions.

## Install

```toml
[dependencies]
cow-sdk-alloy-provider = "0.1"
```

## Build A Provider

```rust,no_run
use std::time::Duration;

use cow_sdk_alloy_provider::RpcAlloyProvider;
use cow_sdk_core::AsyncProvider;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let provider = RpcAlloyProvider::builder()
    .timeout(Duration::from_secs(20))
    .http("https://example.invalid/rpc")?
    .build()
    .await?;

let chain_id = provider.get_chain_id().await?;
# let _ = chain_id;
# Ok(())
# }
```

The builder stores the RPC URL behind `cow_sdk_core::Redacted` before the
transport state becomes visible through debug output. Invalid URLs return a
typed builder error without echoing the supplied value.

## Capability Boundary

`RpcAlloyProvider` implements `AsyncProvider` only. The crate provides all
read methods required by the core trait:

- `get_chain_id`
- `get_code`
- `get_transaction_receipt`
- `get_storage_at`
- `call`
- `read_contract`
- `get_block`
- `get_contract`

`read_contract` parses the supplied JSON ABI, resolves a single non-overloaded
function, ABI-encodes JSON arguments with `alloy-dyn-abi`, dispatches
`eth_call`, decodes the output, and returns the JSON value string expected by
`cow_sdk_core::ContractCall`.

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

Provider failures use `AsyncProviderError`. Transport details are redacted,
remote JSON-RPC errors preserve their code and message, caller input failures
are typed as validation errors, and cooperative cancellation propagates through
the `Cancelled` variant when callers use `cow_sdk_core::Cancellable`.

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE) file for
the full text.
