# Provider Adapters

`cow-sdk-core` exposes five public traits that describe the runtime
boundary between the SDK and a caller-supplied signer or RPC backend:

- [`Signer`](https://docs.rs/cow-sdk-core/latest/cow_sdk_core/traits/trait.Signer.html)
  for synchronous native signers.
- [`AsyncSigner`](https://docs.rs/cow-sdk-core/latest/cow_sdk_core/traits/trait.AsyncSigner.html)
  for async-first signers such as browser wallets and hosted signers.
- [`Provider`](https://docs.rs/cow-sdk-core/latest/cow_sdk_core/traits/trait.Provider.html)
  for synchronous native RPC providers.
- [`AsyncProvider`](https://docs.rs/cow-sdk-core/latest/cow_sdk_core/traits/trait.AsyncProvider.html)
  for read-only async-first RPC providers such as browser-hosted runtimes.
- [`AsyncSigningProvider`](https://docs.rs/cow-sdk-core/latest/cow_sdk_core/traits/trait.AsyncSigningProvider.html)
  for async-first providers that can create signers.

This directory holds worked examples showing how to adapt an external
provider or signer library to those trait surfaces. The examples are
seam demonstrations: a reviewer or consumer can read a single page and
see what an idiomatic implementation looks like against the
`cow-sdk-core` contract.

## Available Worked Examples

- [Adapting alloy](adapting-alloy.md) — implementing `AsyncProvider`,
  `AsyncSigningProvider`, and `AsyncSigner` against `alloy::providers::Provider` and
  `alloy::signers::Signer`.

## Design: Trait Seams Over Concrete Adapters

The SDK ships trait seams rather than per-ecosystem adapter crates.
Consumers select the ecosystem integration that fits their runtime and
version cadence, and the default `cow-sdk` facade stays independent of
any specific external provider or signer library. Leaf crates such as
`cow-sdk-browser-wallet` implement the async trait surface directly
for the runtimes they own.
