# COW Shed Invariants

Status: Accepted
Last reviewed: 2026-05-15
Owning surface: COW Shed source artifacts and deployment registry
Refresh trigger: Refresh when COW Shed proxy bytecode, factory addresses, or EIP-712 hook type strings change upstream.

## Invariants

- The default COW Shed version is `1.0.1`. The deployed bytecode of every
  supported chain returns `"1.0.1"` from `VERSION()` even though source
  HEAD has been advanced to `"2.1.0"`. The SDK signs against deployed
  reality.
- Proxy creation-code artifacts are stored as raw byte files with adjacent
  SHA-256 digest files, validated at build time by
  `crates/contracts/build.rs::validate_cow_shed_proxy_artifacts`.
- The v1.0.1 factory is `0x312f92fe5f1710408B20D52A374fa29e099cFA86` and the
  implementation is `0xa2704cf562ad418bf0453f4b662ebf6a2489ed88`.
- The canonical v1.0.1 user vector maps
  `0x76b0340e50BD9883D8B2CA5fd9f52439a9e7Cf58` to proxy
  `0x66545B93A314e5BdEC9E5Ff9c4D2C7054e6afb04`.
- Hook EIP-712 type strings are byte-identical to the upstream Solidity
  sources and contain no whitespace between commas in declaration order.
  The canonical strings are
  `Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)`
  and
  `ExecuteHooks(Call[] calls,bytes32 nonce,uint256 deadline)Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)`.
- EOA signature byte order is `r || s || v` (not the standard
  `v || r || s`). The signed-hook signature field is a fixed-length 65-byte
  array in that order; the byte order is enforced at the type level in
  future executable helpers via a trybuild compile-fail fixture.
- Delegate calls (`isDelegateCall = true`) are opt-in only via an explicit
  builder method that requires a `// SAFETY:` comment in the immediately
  preceding three lines of the call site. A compile-fail fixture rejects
  use without the safety comment.
- The `COWShedForComposableCoW` forwarder is deployed on Gnosis Chain
  (chain id 100) only. Helpers that construct or interact with the
  forwarder on any other chain id must return the typed
  `CowShedError::COWShedForComposableCoWGnosisOnly { chain }` variant.
- The `cow-shed-ens` Cargo feature (default off) gates the ENS-record
  helper surface so that builds that do not need ENS resolution do not
  pull in the ENS resolver dependency closure. The feature is declared on
  the COW Shed helper crate manifest and consumed by its public surface
  through `#[cfg(feature = "cow-shed-ens")]` guards.
- The `cow-shed-gnosis` Cargo feature gates the Gnosis-only forwarder
  surface for the same reason; non-Gnosis builds may opt out entirely.
- The version selected by the caller threads through every internal
  builder. No helper may construct a downstream object that drops the
  caller-selected `CowShedVersion`. A regression test in the future
  executable helper landing asserts that distinct version variants
  produce distinct CREATE2 proxy addresses.
