# cow-sdk-cow-shed

COW Shed helpers for CoW Protocol: deterministic proxy-address derivation,
EIP-712 hook signing, factory calldata encoding, and a high-level signing
orchestrator — plus the generated ABI bindings underneath.

## What COW Shed is

COW Shed is a **user-owned ERC-1967 proxy** deployed at a **deterministic
CREATE2 address** (the user's address is the salt). Because the address is
derivable before deployment, a user can name it as the receiver of a CoW
Protocol order and attach **pre/post hooks** the proxy executes during
settlement. The owner authorizes a batch of `Call`s with a single EIP-712
signature (`ExecuteHooks`); on-chain, the factory verifies that signature
(ECDSA for EOAs, ERC-1271 for contracts) before running the hooks, and the
proxy is deployed lazily on first use.

Typical uses: **pre-hooks** (just-in-time approvals, unstaking, permit, claim)
and **post-hooks** (bridge proceeds, stake, repay). The whole sequence settles
atomically with the swap, or not at all.

## When to use it

- You want to attach account-abstraction hooks to a CoW order.
- You need the deterministic proxy ("shed") address for a user, before or after
  it is deployed.
- You want to sign and encode `executeHooks` without building the EIP-712
  payload or ABI calldata yourself.

Reach it through the facade with the opt-in feature
(`cow-sdk = { features = ["cow-shed"] }` → `cow_sdk::cow_shed`), or depend on
`cow-sdk-cow-shed` directly. It is **off the default `cow-sdk` closure** and is
never a dependency of the trading or orderbook crates.

## Two layers

- **Building blocks** — deterministic, provider-free primitives: `proxy_of` /
  `proxy_for`, `cow_shed_eip712_domain`, `execute_hooks_signing_hash`,
  `execute_hooks_typed_data_payload`, and the calldata encoders
  (`encode_execute_hooks_calldata_signed` for EOAs,
  `encode_execute_hooks_calldata_with_signature` for any owner including EIP-1271).
- **Orchestrator** — `CowShedHooks`, which composes those blocks plus an owned
  `Signer` into one `sign` call returning a `SignedCowShedCall`.

### Deterministic proxy address (compiled example)

`proxy_for` resolves the chain's factory for you (including Gnosis Chain's
distinct deployment); `proxy_of` takes an explicit factory. The same inputs
always produce the same proxy address:

```rust
use cow_sdk_cow_shed::{CowShedVersion, ProxyAddress, proxy_of};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let factory: ProxyAddress = "0x312f92fe5f1710408B20D52A374fa29e099cFA86".parse()?;
let user: ProxyAddress = "0x76b0340e50BD9883D8B2CA5fd9f52439a9e7Cf58".parse()?;

let proxy = proxy_of(CowShedVersion::V1_0_1, factory, user);
assert_eq!(
    proxy,
    "0x66545B93A314e5BdEC9E5Ff9c4D2C7054e6afb04".parse::<ProxyAddress>()?,
);
# Ok(())
# }
```

### Discovering a user's proxies across versions

A user may own a proxy under any deployed version. `CowShedVersion::ALL`
enumerates every supported version (current generation first), so deriving every
candidate proxy is one allocation-free step; your provider layer then checks
which are deployed:

```rust,ignore
use cow_sdk::cow_shed::{CowShedVersion, proxy_for};
use cow_sdk::core::SupportedChainId;

// `user` is the owner address; `chain` is a SupportedChainId or DeploymentChainId.
let candidates = CowShedVersion::ALL
    .map(|version| (version, proxy_for(SupportedChainId::Mainnet, version, user)));
// candidates == [(V1_0_1, <proxy>), (V1_0_0, <proxy>)]
// Then, per candidate, read on-chain code with your provider to find the live one(s).
```

### Sign hooks and attach to an order (orchestrator)

`CowShedHooks::sign` resolves the owner from the signer, derives the proxy,
signs the `ExecuteHooks` EIP-712 payload through the owned `Signer`, and encodes
`factory.executeHooks` calldata. The resulting `SignedCowShedCall` can be
submitted directly or turned into an app-data hook for an order:

```rust,ignore
use cow_sdk::cow_shed::{Call, CowShedHooks};
use cow_sdk::app_data::{AppDataParams, HookList};
use cow_sdk::core::{AppCode, Signer, SupportedChainId};
use alloy_primitives::{Address, B256, U256};

async fn attach_pre_hook<S: Signer>(signer: &S) -> Result<(), Box<dyn std::error::Error>>
where
    S::Error: std::fmt::Display,
{
    // `new` accepts the `SupportedChainId` your trading flow already holds, or a
    // `DeploymentChainId` directly (the deployment domain, which also covers Lens).
    let hooks = CowShedHooks::new(SupportedChainId::Mainnet);

    // One hook call: e.g. `weth.approve(spender, amount)` executed from the shed.
    let calls = vec![Call::new(spender_call_target(), U256::ZERO, approve_calldata())];

    // Caller-managed: a unique nonce per authorization and a bounded deadline
    // (Unix seconds). `U256::MAX` is the non-expiring sentinel.
    let nonce = B256::random();
    let deadline = U256::from(unix_now() + 3600);

    let signed = hooks.sign(signer, &calls, nonce, deadline).await?;

    // Attach to a CoW order's pre-hooks (gas limit is caller-chosen).
    let pre_hook = signed.to_app_data_hook(500_000);
    let _params = AppDataParams::new(AppCode::new("my-app")?)
        .with_hooks(HookList::new(vec![pre_hook], vec![]));
    // ... thread `_params` through `TradeAdvancedSettings` and post the order.
    Ok(())
}
# fn spender_call_target() -> Address { Address::ZERO }
# fn approve_calldata() -> alloy_primitives::Bytes { alloy_primitives::Bytes::new() }
# fn unix_now() -> u64 { 0 }
```

For the digest-only path (sign a precomputed hash with a raw signer), compute
`execute_hooks_signing_hash(&cow_shed_eip712_domain(chain, version, proxy), …)`
and encode with `encode_execute_hooks_calldata_signed`.

### Smart-contract (EIP-1271) owners

`CowShedHooks::sign` is the externally-owned-account path and returns a 65-byte
ECDSA signature. A smart-contract-wallet owner instead builds the payload, has
the wallet produce its EIP-1271 signature, and encodes that blob directly:

```rust,ignore
let hooks = CowShedHooks::new(chain);
let payload = hooks.typed_data_payload(owner, &calls, nonce, deadline);
let signature = sc_wallet.sign(&payload).await?; // owner's EIP-1271 signature blob
let calldata =
    encode_execute_hooks_calldata_with_signature(&calls, nonce, deadline, owner, signature);
```

On-chain the proxy runs ECDSA recovery for a 65-byte signature and the owner's
`isValidSignature` otherwise, so one encoder serves both owner kinds. The
contract signature may be validated with the `cow-sdk-contracts` EIP-1271
helpers.

## Versions

`CowShedVersion::V1_0_1` is the default and matches the implementation deployed
across the supported chains (it reports `1.0.1` from `VERSION()`). The EIP-712
domain and the EOA signature must target the version deployed on the chain you
are signing for; signing against a different version produces a signature the
proxy rejects. Keep the default unless you explicitly need another deployment,
which you select with `CowShedVersion`.

## Gnosis Chain

Gnosis Chain (chain id 100) carries a **distinct factory and implementation**
from the other chains, so the same user has a different proxy address there.
`proxy_for` / `cow_shed_factory` / `cow_shed_implementation` resolve this
automatically. The Gnosis-only `COWShedForComposableCoW` forwarder is gated
behind the `cow-shed-gnosis` feature, and helpers targeting any other chain
return `CowShedError::COWShedForComposableCoWGnosisOnly`.

## Gotchas

- **Nonces** are caller-managed for replay protection and revocation; use a
  unique nonce per authorization (sequential nonces save gas via the on-chain
  bitmap).
- **Deadlines** are Unix seconds; prefer a bounded deadline over the
  non-expiring `U256::MAX` sentinel.
- **`isDelegateCall`** is opt-in only via `Call::delegate_call`; per ADR 0049
  each call site must carry a `// SAFETY:` comment — delegatecall runs in the
  proxy's own storage context.
- **Partially-fillable orders** run pre-hooks on the first fill only; post-hooks
  run on every fill.
- **Gas**: the app-data hook's `gas_limit` is caller-chosen; the SDK does not
  estimate gas (it owns no provider).

## Parity with `@cowprotocol/cow-sdk`

The crate mirrors the upstream `CowShedSdk` / `CoWShedHooks` surface: proxy
derivation, the EIP-712 `ExecuteHooks` types and domain, factory calldata
encoding, and the `signCalls`-style sign-and-encode flow via `CowShedHooks`.
The caller-selected `CowShedVersion` is threaded through proxy derivation and
signing, and the Gnosis Chain factory/implementation deployment is resolved by
the chain-keyed lookups. The crate additionally exposes
`encode_execute_pre_signed_hooks_calldata` for the on-chain
`executePreSignedHooks` path.

## Feature flags

| Feature | Effect |
| --- | --- |
| `cow-shed-gnosis` | Exposes the Gnosis-only `COWShedForComposableCoW` forwarder surface. |
| `cow-shed-ens` (alias `with_ens`) | Reserved for ENS-oriented factory getters. |

Default builds pull in no provider or signer adapters. The MSRV is Rust 1.94.0.
This crate runs no service loops, persistence, polling, or submission; consumers
build those layers above the pure helpers.
