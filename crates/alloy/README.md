# cow-sdk-alloy

Native composed Alloy adapter package for the `cow-rs` SDK.

This crate is the umbrella package for applications that want native Alloy
provider and signer support through one opt-in dependency. It re-exports the
leaf package namespaces while keeping the default `cow-sdk` facade free of
native Alloy runtime dependencies.

The three-crate split is intentional: `cow-sdk-alloy-provider` owns read-only
RPC access, `cow-sdk-alloy-signer` owns local-key signing, and this crate owns
the composed client for applications that need both behind the SDK's runtime
neutral core traits.

## Capability Boundary

This crate is native-only. Wasm applications should use
[`cow-sdk-browser-wallet`](https://docs.rs/cow-sdk-browser-wallet) for browser
wallet signing and inject browser RPC access through the supported browser
transport surfaces.

The native-only boundary is enforced at compile time on `wasm32` targets. That
keeps browser builds on the audited browser-wallet path instead of surfacing
deep transitive native-runtime errors from Alloy networking or local-key
dependencies.

The package boundary is intentionally narrow in this release. Read-only provider
support is owned by
[`cow-sdk-alloy-provider`](https://docs.rs/cow-sdk-alloy-provider), signing
support is owned by [`cow-sdk-alloy-signer`](https://docs.rs/cow-sdk-alloy-signer),
and this package is the composed namespace for consumers that want both.
`AlloyClient` implements `Provider`, `LogProvider`, and `SigningProvider`; the
owned signer handle returned by `create_signer` implements `Signer`, signs CoW
EIP-712
typed-data payloads directly, submits transactions through Alloy's
wallet-filler provider, and reports `TransactionBroadcast` with the broadcast
transaction hash.

Transaction filling remains Alloy-owned. The SDK builds the local signer and
provider composition, then delegates nonce, fee, chain, and transaction-type
filling to Alloy's wallet-filler provider before broadcasting.

Signer handles own reference-counted client state, so a handle returned from
`create_signer` remains usable after the parent `AlloyClient` value is dropped.
Canonical typed-data signing preserves the payload primary type for CoW order
signing, ECDSA signatures are normalized through the shared contracts helper,
and public error formatting follows the same redaction contract as the provider
and signer leaves.

The cow-named public types interoperate with their `alloy_primitives`
counterparts at zero runtime cost via `.0` access or
`From::from(value).into()` per
[ADR 0052](https://github.com/0xSymbiome/cow-rs/blob/main/docs/adr/0052-alloy-primitives-canonical-primitive-layer.md);
the adapter consumes cow values directly without per-call conversion.

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
    .build_checked()
    .await?;
# let _ = client;
# Ok(())
# }
```

`build_checked()` performs an RPC `eth_chainId` check and rejects a mismatch
between the configured SDK chain and the remote node. Use `build()` only when
the caller intentionally defers chain verification, then call
`verify_chain_id().await` before using the client for chain-sensitive work.

RPC retries are off by default. Pass a `cow_sdk_alloy::RetryConfig` to the
builder's `retry` setter to opt into bounded exponential backoff for
transient, rate-limited reads; the umbrella reuses the provider leaf's retry
policy.

## Signing And Submission

Raw `sign_transaction` is intentionally unsupported in this release because
the relevant Alloy provider path asks the remote JSON-RPC peer to sign. Use
`send_transaction` for wallet-filler submission or the signer leaf for local
message and typed-data signatures.

`send_transaction` returns `TransactionBroadcast` carrying the broadcast
transaction hash read from Alloy's pending transaction handle. It does not
prove block inclusion or execution success and does not poll for a receipt.

Use `Provider::get_transaction_receipt` on the client when mined state is
needed. Receipt lookup delegates to the provider leaf and returns
`TransactionReceipt` with optional status, block number, block hash, gas used,
sender, and recipient fields when the chain response exposes them.

## Maintenance

The release is pinned to an explicit Alloy runtime and Alloy Core ABI
compatibility matrix. The workspace lockfile invariant checks those families
separately so a runtime-only update cannot silently pull ABI decoding behavior
forward, and an ABI-only update cannot silently change runtime transport
behavior.

Two implementation pairs are intentionally mirrored across the leaves and the
umbrella: `crates/alloy-provider/src/read_contract.rs` with the read-contract
path in `crates/alloy/src/client.rs`, and
`crates/alloy-signer/src/conversion.rs` with `crates/alloy/src/conversion.rs`.
Run `cargo test -p cow-rs-workspace-tests --test alloy_read_contract_parity_invariant`
when changing the read-contract path so both adapters keep byte-for-byte output
parity for supported ABI values.

Public consumers should rely on the documented client, builder, provider,
signer, and error classes. Lower-level conversion and re-export plumbing is
hidden from docs and may change as the Alloy integration evolves.

## Related Crates

- [`cow-sdk-alloy-provider`](https://docs.rs/cow-sdk-alloy-provider) owns
  read-only provider support.
- [`cow-sdk-alloy-signer`](https://docs.rs/cow-sdk-alloy-signer) owns native
  signing support.
- [`cow-sdk`](https://docs.rs/cow-sdk) is the curated facade for most SDK users.

## License

Licensed under GPL-3.0-or-later. See the workspace
[LICENSE](https://github.com/0xSymbiome/cow-rs/blob/main/LICENSE) file for
the full text.
