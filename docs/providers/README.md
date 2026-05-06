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

This directory documents shipped and custom adapter paths against those trait
surfaces. Consumers can use the native Alloy adapters directly, or implement
the same traits for another provider ecosystem.

## Available Worked Examples

- [Adapting Alloy](adapting-alloy.md) — using the opt-in
  `cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, and `cow-sdk-alloy`
  crates against the `AsyncProvider`, `AsyncSigningProvider`, and
  `AsyncSigner` contracts.

## Design: Trait Seams Over Concrete Adapters

The SDK keeps provider ecosystems out of the default facade. Native Alloy
support ships as explicit leaf crates, and other ecosystems can still integrate
by implementing the same `cow-sdk-core` traits. Leaf crates such as
`cow-sdk-browser-wallet` implement the async trait surface directly for the
runtimes they own.
