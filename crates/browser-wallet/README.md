# cow-sdk-browser-wallet

Browser-wallet integration for the [CoW Protocol](https://cow.fi) Rust
SDK. Exposes typed EIP-1193 provider, signer, discovery, and session
contracts for WASM consumers plus a deterministic mock transport for
tests and review flows.

> ⚠️ **Alpha — `0.1.0-alpha`.** Pre-release and not security-audited; the public
> API may change before `0.1.0`. It is published as a pre-release, so Cargo
> selects it only when you opt in (`cow-sdk-browser-wallet = "0.1.0-alpha.1"`).
> Review it yourself before relying on it with real funds.

This crate is the browser-runtime leaf of the `cow-rs` package family.
The public API stays Rust-native and typed; raw JavaScript payloads
remain local to the crate. Most consumers reach this crate through the
[`cow-sdk`](https://crates.io/crates/cow-sdk) facade's `browser-wallet`
feature flag; depend on it directly when building a WASM integration
that does not use the trading facade. Injected-wallet behavior is
environment-sensitive: authorization prompts, provider inventory, and
vendor-specific support are controlled by the browser runtime rather
than normalized into universal SDK guarantees.

For TypeScript applications that already use viem, ethers, wagmi, or another
JavaScript wallet stack, prefer `cow-sdk-wasm`. It exposes the SDK through
typed callbacks and keeps JavaScript wallet objects outside Rust trait storage.
This crate remains the Rust-native browser-wallet leaf.

Transaction submission through the browser wallet returns
`TransactionBroadcast` with the hash accepted by the injected provider.
Receipt observation is a separate provider lookup. When an EIP-1193 receipt is
available, this crate populates `TransactionReceipt` fields for status, block,
gas, sender, and recipient; absent optional fields remain empty, while present
malformed fields fail closed with a typed browser-wallet error.

Typed-data signing consumes `cow_sdk_core::TypedDataDomain` directly; the
cow struct emits the canonical EIP-1193 `eth_signTypedData_v4`
second-parameter wire shape through its own `Serialize` impl per
[ADR 0052](https://github.com/0xSymbiome/cow-rs/blob/main/docs/adr/0052-alloy-primitives-canonical-primitive-layer.md),
pinned by `PROP-BWL-007` against
`parity/fixtures/signing/eth_sign_typed_data_request.json`.

## What it provides

- **Typed EIP-1193 provider, signer, and session** — `Eip1193Provider`
  (implements `Provider` + `SigningProvider`), `Eip1193Signer` (implements
  `Signer`), and `WalletSession`, usable by the trading crate like any other
  signer/provider.
- **Bounded injected-wallet discovery** — EIP-6963 first with a `window.ethereum`
  fallback, never auto-selecting when more than one wallet is present
  (`InjectedWalletDiscovery`).
- **Typed chain management** — `add_chain` / `switch_chain` / `switch_or_add_chain`
  with `WalletChainParameters` validation (http(s) URLs, non-empty names) before
  any RPC, and success confirmed against a refreshed session.
- **Origin-trust gating** — anonymous (non-EIP-6963) providers must opt in via
  `Eip1193ProviderBuilder::trusted_origin`, or `build()` fails with
  `UntrustedProviderOrigin`.
- **Chain-bound signing** — `signer_for_chain` revalidates the session chain and
  the typed-data domain chain before signing.
- **A deterministic mock transport** — `MockEip1193Transport`, panic-free and
  scriptable, for tests and review without a browser.
- **Normalized RPC errors** — raw JS codes map to typed variants (`4001`, `4900`,
  `4901`, `4902`, `-32601`); a `4001` rejection round-trips as
  `cow_sdk_core::UserRejection` so the signing path can route it.

## Install

```toml
[dependencies]
cow-sdk-browser-wallet = "0.1.0-alpha.1"
```

Or enable the feature through the facade:

```toml
[dependencies]
cow-sdk = { version = "0.1.0-alpha.1", features = ["browser-wallet"] }
```

## Minimal example

```rust
use cow_sdk_browser_wallet::{BrowserWallet, MockEip1193Transport, Origin};

let transport = MockEip1193Transport::sepolia().with_label("example wallet");
let origin = Origin::new("test://example-wallet").expect("example origin must be valid");
let _wallet = BrowserWallet::from_trusted_transport(transport, origin)
    .expect("trusted example transport must build");
```

## Feature flags

| Feature | Default | Enables |
| --- | --- | --- |
| `tracing` | off | `tracing` spans on `BrowserWallet` methods and origin-trust warnings. |

## Where this fits

This is the Rust-native browser-runtime leaf. It exposes only the typed
`Eip1193Transport` seam — raw JS payloads stay private — and no `alloy_*` type
appears in its public API. Real discovery and detection exist only on `wasm32`;
native builds compile (so tests link) but return empty/`None`. For TypeScript
apps already on viem, ethers, or wagmi, prefer
[`cow-sdk-wasm`](https://github.com/0xSymbiome/cow-rs/blob/main/crates/wasm/README.md)
(published to npm); this crate is for Rust-in-browser (Yew, Leptos, Dioxus)
integrations. Reach it through the
[`cow-sdk`](https://crates.io/crates/cow-sdk) facade's `browser-wallet` feature
as `cow_sdk::browser_wallet`.

## Where to next

- [Browser-Wallet Example](https://github.com/0xSymbiome/cow-rs/tree/main/examples/wasm/cow-trader-dioxus)
- [cow-sdk-wasm README](https://github.com/0xSymbiome/cow-rs/blob/main/crates/wasm/README.md)
- [Architecture](https://github.com/0xSymbiome/cow-rs/blob/main/docs/architecture.md)
- [Workspace README](https://github.com/0xSymbiome/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-or-later. See the workspace
[LICENSE](https://github.com/0xSymbiome/cow-rs/blob/main/LICENSE)
file for the full text.
