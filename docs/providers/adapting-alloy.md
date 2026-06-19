# Adapting Alloy

`cow-rs` ships native Alloy adapters as opt-in crates. Use them when a
native application wants Alloy-backed chain RPC, local private-key signing, or
both through the same client.

The native Alloy adapter family ships as three crates so a consumer can pull
only the capabilities they exercise: `cow-sdk-alloy-provider` for read-only
RPC, `cow-sdk-alloy-signer` for local private-key signing, and `cow-sdk-alloy`
for the composed read-plus-sign flow that most trading applications need. The
provider leaf stays free of signing-crypto features, and the signer leaf stays
free of transport plumbing.

## Crates And Features

| Need | Crate or facade feature |
| --- | --- |
| Read-only chain RPC through `Provider` | `cow-sdk-alloy-provider` or `cow-sdk` feature `alloy-provider` |
| Local private-key signing through `Signer` | `cow-sdk-alloy-signer` or `cow-sdk` feature `alloy-signer` |
| Composed provider plus signer for `Trading` async helpers | `cow-sdk-alloy` or `cow-sdk` feature `alloy` |

The default `cow-sdk` facade remains provider-neutral. Native Alloy support is
explicitly enabled by feature or direct crate dependency and is not available
on `wasm32-unknown-unknown`; browser applications use the `cow-sdk-wasm` package
to bridge signing and RPC access to the host wallet through the EIP-1193 request
callback.

## Umbrella Client

`cow-sdk-alloy::AlloyClient` combines an Alloy HTTP provider with an Alloy
local private-key signer. The client implements `Provider`, `LogProvider`,
and `SigningProvider`; the owned signer handle returned by `create_signer`
implements `Signer` and remains usable after the parent client is
dropped.

```rust,no_run
use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::{SigningProvider, SupportedChainId};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = AlloyClient::builder()
    .http("https://example.invalid/rpc")?
    .private_key("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")?
    .chain_id(SupportedChainId::Sepolia)
    .build_checked()
    .await?;
# let _ = client;
# Ok(())
# }
```

The umbrella signer handle signs CoW EIP-712 typed-data payloads directly,
normalizes ECDSA `v` values through the shared contracts helper, sends
transactions through Alloy's wallet-filler provider, and returns the broadcast
transaction hash as `TransactionBroadcast`. The `Signer` surface does not carry
raw transaction signing; on-chain execution goes through `send_transaction`,
where nonce, fee, chain, and broadcast context are available.

The umbrella composes its provider and signer through Alloy's wallet-filler
provider pattern rather than reimplementing transaction filling, signing, or
dispatch. Reusing that composition keeps the adapter small and avoids
re-deriving nonce, chain id, fee, and signature filling. The adapter's role is
to bridge SDK domain types into the Alloy types the wallet-filler consumes and
to preserve the SDK-owned error and cancellation contracts.

Canonical typed-data signing preserves the payload's `primary_type`
end-to-end. That matters for CoW orders because the settlement contract expects
the canonical `"Order"` type; signing the same fields under a placeholder type
would produce a valid-looking signature over the wrong digest.

## Leaf Adapters

Use the provider leaf when a flow needs read-only chain RPC without a signer:

```rust,no_run
use cow_sdk_alloy_provider::RpcAlloyProvider;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let provider = RpcAlloyProvider::builder()
    .http("https://example.invalid/rpc")?
    .build()?;
# let _ = provider;
# Ok(())
# }
```

Use the signer leaf when a flow needs local message or typed-data signing but
does not need provider-backed transaction submission:

```rust,no_run
use cow_sdk_alloy_signer::LocalAlloySigner;
use cow_sdk_core::SupportedChainId;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let signer = LocalAlloySigner::from_private_key(
    "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    SupportedChainId::Sepolia,
)?;
# let _ = signer;
# Ok(())
# }
```

The signer leaf returns provider-required errors for transaction submission and
gas estimation because it does not own RPC state. Use the umbrella client when
the same runtime must both sign and submit transactions.

## RPC Resilience

By default every RPC request is issued once and a transient transport failure —
such as a public-endpoint `429` — is surfaced to the caller, keeping the SDK
runtime-neutral. The consumer owns chain-RPC resilience.

Both the provider leaf and the umbrella client expose an opt-in `retry`
setter that wraps the JSON-RPC client in a bounded exponential backoff layer for
transient, rate-limited reads:

```rust,no_run
use cow_sdk_alloy_provider::{RetryConfig, RpcAlloyProvider};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let provider = RpcAlloyProvider::builder()
    .http("https://example.invalid/rpc")?
    .retry(RetryConfig::default())
    .build()?;
# let _ = provider;
# Ok(())
# }
```

`RetryConfig` is re-exported from `cow_sdk_alloy` for the umbrella builder. The
REST transports (orderbook, subgraph, IPFS) carry their own shared
`TransportPolicy` retry and are configured separately on their builders.

## Transaction Lifecycle

`AlloyClientSignerHandle::send_transaction` submits through Alloy's
wallet-filler provider and reads the already accepted hash through
`pending.tx_hash()`. It returns after broadcast acknowledgement and does not
wait for `eth_getTransactionReceipt`.

Receipt observation is an explicit provider operation. The Alloy provider leaf
maps Alloy receipts into `TransactionReceipt`, including EIP-658 success /
reverted status when present, block number, block hash, gas used, sender, and
recipient. Pre-Byzantium post-state receipts keep `status` empty rather than
being coerced into success.

## Trading Integration

The async helper paths on `Trading` accept the umbrella through the core
traits. Typical uses include allowance reads, approval submission, pre-sign
transaction construction, and on-chain cancellation.

```rust,no_run
use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::SupportedChainId;
use cow_sdk_trading::Trading;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = AlloyClient::builder()
    .http("https://example.invalid/rpc")?
    .private_key("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")?
    .chain_id(SupportedChainId::Sepolia)
    .build_checked()
    .await?;

let trading = Trading::builder()
    .chain_id(SupportedChainId::Sepolia)
    .app_code("cow-rs/alloy-adapter")
    .build()?;

let signer = client.create_signer("local").await?;
# let _ = (trading, signer);
# Ok(())
# }
```

For runnable scenarios, see the Alloy examples listed in
[`examples/native/README.md`](../../examples/native/README.md).

## Stability Boundary

Only the documented exported types and methods are part of the consumer API and
subject to semver guarantees. Doc-hidden internals exist for cross-crate
composition inside the SDK family and may change in any minor release without
notice. Consumers who need a capability that is not in the documented surface
should open an issue requesting a stable API rather than reaching into
doc-hidden items through private rustdoc tooling.

The internal composition shape and the rationale for keeping it hidden are
recorded in the adapter ADRs and standing adapter audits.

Under [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md),
the cow-named identity and numeric public types resolve to cow-owned
`#[repr(transparent)]` newtypes around the corresponding `alloy_primitives`
types; the adapter bridges cow types into alloy types at zero runtime cost
via `From::from(addr).into()` or the `as_alloy` / `into_alloy` accessors. The signer leaf's typed-data
signing path consumes the cow `TypedDataDomain` struct directly; the cow
struct emits the canonical EIP-1193 `eth_signTypedData_v4` wire shape
through its own `Serialize` impl and bridges to
`alloy_sol_types::Eip712Domain` at the EIP-712 hashing seam through the
`to_alloy_domain()` adapter at `crates/alloy-signer/src/conversion.rs`.
