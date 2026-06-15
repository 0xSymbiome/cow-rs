# cow-sdk-alloy-signer

Native Alloy-backed local signing adapter package for the `cow-rs` SDK.

This crate is the signing leaf for native applications that want
`cow_sdk_core::Signer` backed by Alloy local private-key signing. It is
published as a separate opt-in crate so read-only provider users and the default
`cow-sdk` facade do not pull native local-signing dependencies.

## Capability Boundary

This crate is native-only. Wasm applications should use
[`cow-sdk-browser-wallet`](https://docs.rs/cow-sdk-browser-wallet) for browser
wallet signing.

The native-only boundary is enforced at compile time on `wasm32` targets. That
keeps browser signing on the audited EIP-1193/browser-wallet path and avoids
shipping local-key native dependencies into browser builds.

The package boundary is intentionally narrow:

- `LocalAlloySigner` implements `cow_sdk_core::Signer`.
- It signs EIP-191 messages and EIP-712 typed-data payloads.
- Canonical typed-data signing preserves the payload primary type.
- Canonical typed-data signing accepts nested multi-type payloads whose fields
  reference other structs declared in the type map, directly or as arrays.
- ECDSA signatures are normalized through the shared `cow-sdk-contracts`
  signature helper before they are returned.
- `sign_transaction`, `send_transaction`, and `estimate_gas` return
  `SignerError::ProviderRequired` because a standalone local signer cannot
  fill nonce, fee, chain, or transaction-type context.
- Provider-backed transaction submission is owned by `cow-sdk-alloy`, whose
  signer handle returns `TransactionBroadcast`; receipt observation is a
  provider lookup, not a local-signing concern.

The typed-data path preserves the caller's primary type because CoW Protocol
order signing depends on the `Order` domain shape matching the payload. The
payload form is the only typed-data entry point: it carries the domain, the
full type map, the primary-type name, and the message in one value, so the
signer never has to guess a placeholder type.

The cow `TypedDataDomain` is a cow-owned `#[non_exhaustive]` struct per
[ADR 0052](https://github.com/0xSymbiome/cow-rs/blob/main/docs/adr/0052-alloy-primitives-canonical-primitive-layer.md);
the `conversion` module bridges `TypedDataDomain` to
`alloy_sol_types::Eip712Domain` at the alloy-signer seam where the
alloy-primitive form is needed for ECDSA signing.

ECDSA `v` normalization is centralized through the contracts helper shared by
the SDK. Keeping normalization in one helper prevents provider-specific recovery
id formats from leaking through the public signing API.

Public formatting is redacted by construction and the signer error type does
not derive `Debug`. The manual debug implementation delegates through the same
redaction contract used by display formatting, so private keys, RPC URLs, and
transport internals are not exposed through ordinary error reporting.

Provider access is owned by
[`cow-sdk-alloy-provider`](https://docs.rs/cow-sdk-alloy-provider), and combined
provider plus signer composition is owned by
[`cow-sdk-alloy`](https://docs.rs/cow-sdk-alloy).

## Install

```toml
[dependencies]
cow-sdk-alloy-signer = "0.1"
```

The crate enables EIP-712 signing by default. Disable default features only if
your application needs the EIP-191 message path without typed-data support.

```toml
[dependencies]
cow-sdk-alloy-signer = { version = "0.1", default-features = false }
```

## Example

```rust
use cow_sdk_alloy_signer::LocalAlloySigner;
use cow_sdk_core::{Signer, SupportedChainId};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let signer = LocalAlloySigner::builder()
    .private_key("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")?
    .chain_id(SupportedChainId::Mainnet)
    .build()?;

let owner = signer.address().await?;
let signature = signer.sign_message(b"hello cow").await?;
# let _ = (owner, signature);
# Ok(())
# }
```

The builder requires both key source and chain id before `build()` is available.
Its typestate markers are sealed, so external code cannot construct a completed
builder state by hand.

## Errors

`SignerError` is non-exhaustive and exposes a stable
`SignerErrorClass` partition:

- `validation`
- `signing`
- `provider_required`
- `unsupported`
- `cancelled`
- `internal`

Validation, signing, and internal details are redacted in public formatting.
`From<cow_sdk_core::Cancelled>` is implemented so
`Cancellable::cancel_with(...).await?` propagates cancellation through this
crate's error type.

## Related Crates

- [`cow-sdk-alloy`](https://docs.rs/cow-sdk-alloy) composes provider and signer
  support behind one native package.
- [`cow-sdk-alloy-provider`](https://docs.rs/cow-sdk-alloy-provider) owns
  read-only provider support.
- [`cow-sdk`](https://docs.rs/cow-sdk) is the curated facade for most SDK users.

## License

Licensed under GPL-3.0-or-later. See the workspace
[LICENSE](https://github.com/0xSymbiome/cow-rs/blob/main/LICENSE) file for
the full text.
