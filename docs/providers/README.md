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
surfaces. Consumers who use `cow-sdk-trading` should pick the native Alloy
adapter on native targets or the browser-wallet leaf on wasm. Consumers
building a generic Ethereum application without trading helpers should use
Alloy directly; the adapter exists to wire native Alloy into the SDK's trading
and signing contracts.

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

JavaScript and TypeScript consumers may use `cow-sdk-wasm` for specialized
cases: deterministic Rust signing parity, single-source-of-truth Rust +
TypeScript embedding, and Cloudflare Workers (size-compatible with the current
Workers Free compressed-size limit at the time of measurement; full Workers
support pending release-bundle and startup validation). The wasm package keeps
runtime objects (viem, ethers, wagmi, raw EIP-1193 providers, fetch) behind
typed callbacks instead of asking adapter authors to store JavaScript handles
inside Rust trait objects.

For most browser dapps, web apps, CowSwap-style UIs, and standard TypeScript
applications, the upstream
[`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk)
TypeScript SDK is the recommended choice; it is substantially smaller at
equivalent feature subsets.

Those traits are the runtime-neutral contract. A single trading helper can
drive native Alloy, the browser-wallet leaf, or a custom adapter because the
provider and signer seams live in `cow-sdk-core` rather than in a concrete
runtime crate.

The transaction lifecycle is split across the same traits. Signers return
`TransactionBroadcast` after a backend accepts a transaction hash for broadcast;
providers return `Option<TransactionReceipt>` when a transaction is visible to
receipt lookup. Adapter authors should populate receipt fields when the runtime
exposes them and should keep receipt polling out of `send_transaction`.
