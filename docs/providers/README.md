# Provider Adapters

`cow-sdk-core` exposes four public traits that describe the runtime
boundary between the SDK and a caller-supplied signer or RPC backend:

- [`Signer`](https://docs.rs/cow-sdk-core/latest/cow_sdk_core/traits/trait.Signer.html)
  for async-first signers such as browser wallets, hosted signers, and native
  key stores.
- [`Provider`](https://docs.rs/cow-sdk-core/latest/cow_sdk_core/traits/trait.Provider.html)
  for read-only async-first RPC providers.
- [`SigningProvider`](https://docs.rs/cow-sdk-core/latest/cow_sdk_core/traits/trait.SigningProvider.html)
  for async-first providers that can create signers.
- [`LogProvider`](https://docs.rs/cow-sdk-core/latest/cow_sdk_core/traits/trait.LogProvider.html)
  for providers that can additionally fetch event logs. This is an opt-in
  capability supertrait layered on `Provider` (the same shape as
  `SigningProvider`): read-only adapters implement only `Provider`, while a
  log-capable adapter also implements `LogProvider`. `get_logs` is the
  single-call entry point — one backend query over a caller-bounded block
  range, returning raw logs for the fail-closed decoders, never a watcher or
  indexer loop.

This directory documents shipped and custom adapter paths against those trait
surfaces. Consumers who use `cow-sdk-trading` should pick the native Alloy
adapter on native targets or the browser-wallet leaf on wasm. Consumers
building a generic Ethereum application without trading helpers should use
Alloy directly; the adapter exists to wire native Alloy into the SDK's trading
and signing contracts.

## Available Worked Examples

- [Adapting Alloy](adapting-alloy.md) — using the opt-in
  `cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, and `cow-sdk-alloy`
  crates against the `Provider`, `SigningProvider`, and
  `Signer` contracts.

## Design: Trait Seams Over Concrete Adapters

The SDK keeps provider ecosystems out of the default facade. Native Alloy
support ships as explicit leaf crates, and other ecosystems can still integrate
by implementing the same `cow-sdk-core` traits. Leaf crates such as
`cow-sdk-browser-wallet` implement the trait surface directly for the
runtimes they own.

JavaScript and TypeScript consumers may use `cow-sdk-wasm` for specialized
cases: deterministic Rust signing parity, single-source-of-truth Rust +
TypeScript embedding, and Cloudflare Workers (size-compatible with the current
Workers Free compressed-size limit at the time of measurement; the
`cloudflare` flavor is built and tested end-to-end in CI (Workers Vitest
plus the Cloudflare gateway example), within the Workers compressed-size
budget). The wasm package keeps
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

The trait contract is identity-type-agnostic: under
[ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md), the
cow identity and numeric types interoperate with alloy types at zero runtime
cost through `From::from(...)` and the `as_alloy` / `into_alloy` accessors, so adapter authors bridge
cow domain types at the adapter boundary without distorting the trait
surface.
