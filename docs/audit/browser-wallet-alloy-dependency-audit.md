# Browser-Wallet Alloy Dependency Audit

Status: Current  
Last reviewed: 2026-04-21  
Owning surface: `cow-sdk-browser-wallet` typed EIP-1193 contract-call bridge and its `alloy-primitives` / `alloy-dyn-abi` / `alloy-json-abi` ABI helpers  
Refresh trigger: Upstream movement in the alloy family (new major, dropped transitive dependency), a new reviewed warning surfacing through the alloy toolchain, or a new maintained successor to the affected proc-macro deps  
Related docs:
- [Dependency Gate Audit](dependency-gate-audit.md)
- [CID Dependency Audit](cid-dependency-audit.md)
- [ADR 0007](../adr/0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md)

## Scope

This audit covers:

- the `alloy-primitives`, `alloy-dyn-abi`, and `alloy-json-abi` dependency
  family used by `cow-sdk-browser-wallet` for typed EIP-1193 contract-call
  encoding and response decoding
- the reachable-only-through-alloy RustSec advisories this adoption brings,
  namely the `derivative` and `paste` proc-macro advisories
- the build-time-only scope of those advisories
- fail-closed handling of unsupported or malformed ABI inputs at the
  contract-call bridge

It does not cover the published CID dependency posture, transport-layer
TLS choices, or any advisory outside the narrow alloy subtree reachable
from the browser-wallet contract-call bridge.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Maintained ABI family | `alloy-primitives`, `alloy-dyn-abi`, and `alloy-json-abi` replace the previously unmaintained `ethabi` dependency at the browser-wallet contract-call bridge | Conforms |
| Public API exposure | No `alloy_*` type appears in any `pub fn` signature across the workspace; the bridge stays typed in `cow-sdk` public wrappers | Conforms |
| Reachable advisories | `RUSTSEC-2024-0388` (derivative unmaintained) and `RUSTSEC-2024-0436` (paste unmaintained) reach this workspace only through the alloy toolchain | Reviewed warning |
| Proc-macro scope | Both advisories apply to build-time proc-macro crates only; no runtime code path is affected | Conforms |

## Current Contract

### Maintained ABI Family

The current browser-wallet contract-call bridge uses:

- `alloy-primitives` for `Address`, `U256`, `I256`, `B256`, and related
  validated primitives
- `alloy-dyn-abi` for `DynSolType`, `DynSolValue`, `FunctionExt`, and
  `JsonAbiExt`, which own the canonical dynamic ABI encode and decode
  surface
- `alloy-json-abi` for parsing ABI JSON into a typed `JsonAbi` and
  looking up functions by name

These three crates replace the previously unmaintained `ethabi 18.0.0`
dependency. The swap is isolated to `crates/browser-wallet/src/provider.rs`
plus the matching workspace and crate manifests.

### Public API Boundary

The alloy types stay private to `cow-sdk-browser-wallet`. The workspace-level
public API never exposes an `alloy_*` type in any `pub fn` signature. The
bridge keeps its public shape on `cow_sdk_core::{AsyncProvider,
ContractCall, ContractHandle, ...}` so consumers reach typed results
without pulling alloy into their own surfaces.

### Reachable Advisories

Two RustSec advisories reach this workspace transitively through alloy:

- `RUSTSEC-2024-0388`: `derivative 2.2.0` is unmaintained. Reachable
  through `alloy-primitives -> ruint -> ark-ff -> derivative`. `derivative`
  is a proc-macro crate; no runtime code from it is compiled into any
  shipped artifact.
- `RUSTSEC-2024-0436`: `paste 1.0.15` is unmaintained. Reachable through
  `alloy-sol-macro -> syn-solidity -> paste` and through `alloy-primitives
  -> ruint -> ark-ff -> paste`. `paste` is also a proc-macro crate with no
  runtime footprint.

Neither advisory is a known security vulnerability. Both are lifecycle
status records on upstream crates that are widely used across the Rust
ecosystem.

### Gate Posture

The `cargo audit` gate continues to block every other unsound and
unmaintained advisory while explicitly tolerating these two identifiers
plus the unrelated `RUSTSEC-2026-0097` covered by the
[CID Dependency Audit](cid-dependency-audit.md). The ignore list lives in
`.github/config/deny.toml` under `[advisories].ignore` with per-entry
revisit comments, and the same ignores are mirrored on the `cargo audit`
command line in `.github/workflows/ci.yml` and
`.github/workflows/release-readiness.yml`.

### Advisory Posture

Revisit trigger for `RUSTSEC-2024-0388`:

- Drop the ignore when `ruint`, `ark-ff`, or an intermediate upstream
  release removes `derivative` from the transitive graph reached by
  `alloy-primitives`.
- Calendar floor: re-review every 90 days and update both
  `.github/config/deny.toml` and this audit's `Last reviewed`.

Revisit trigger for `RUSTSEC-2024-0436`:

- Drop the ignore when `alloy-sol-macro`, `syn-solidity`, `ruint`, or
  `ark-ff` releases no longer reach `paste`, or when a maintained
  successor replaces `paste` in the affected subtrees.
- Calendar floor: same 90-day re-review rhythm as
  `RUSTSEC-2024-0388`.

If either trigger fires, refresh this audit, remove the matching ignore
from `.github/config/deny.toml`, and remove the `--ignore` flags from
`.github/workflows/ci.yml` and `.github/workflows/release-readiness.yml`.

## Evidence

Primary implementation points:

- `crates/browser-wallet/src/provider.rs`
- `crates/browser-wallet/Cargo.toml`
- `Cargo.toml` (workspace dependencies)
- `.github/config/deny.toml`

Primary regression coverage:

- `crates/browser-wallet/tests/provider_contract.rs`
- `crates/browser-wallet/tests/wallet_contract.rs`
- `crates/browser-wallet/tests/state_machine_contract.rs`
- `e2e/browser-wallet/tests/browser-wallet-console.spec.ts`

Validation surface:

```text
cargo tree -p cow-sdk-browser-wallet -d
cargo deny check bans licenses sources --config .github/config/deny.toml
cargo audit --deny unsound --deny unmaintained \
  --ignore RUSTSEC-2026-0097 \
  --ignore RUSTSEC-2024-0388 \
  --ignore RUSTSEC-2024-0436
cargo test -p cow-sdk-browser-wallet
cargo clippy -p cow-sdk-browser-wallet --all-targets --all-features -- -D warnings
```
