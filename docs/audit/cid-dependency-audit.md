# CID Dependency Audit

Status: Current
Last reviewed: 2026-06-08
Owning surface: `cow-sdk-app-data` CID encoding, canonical-JSON serialisation, and published dependency boundary
Refresh trigger: Changes to CID dependencies, the canonical-JSON dependency, the supported CID encoding, or the published dependency posture for the app-data and core stacks
Related docs:
- [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md)
- [Dependency Gate Audit](dependency-gate-audit.md)
- [Verification Guide](../verification.md)
- [Verification Matrix](../verification.md)

## Scope

This audit covers:

- the CID and multihash dependencies used by `cow-sdk-app-data`
- the canonical-JSON dependency that the CID derivation rides on
- the supported app-data CID construction path
- the canonical app-data CID conversion path in `cow-sdk-app-data`
- published-upstream dependency posture for the maintained CID stack
- fail-closed handling for malformed app-data hex and unsupported CID
  encodings

It does not cover broader workspace TLS, HTTP, or non-CID dependency policy
outside the app-data boundary.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Supported CID conversion | Keep `cid`, `multihash`, and `multibase` as the maintained path | Conforms |
| Published upstream dependency posture | `cid 0.11.3` no longer reaches the yanked `core2` dependency path | Conforms |
| Unsupported CID encodings | Reject malformed or unsupported inputs, including CIDv0 (`Qm...` / dag-pb / sha2-256) and CIDv1 raw CIDs with non-keccak256 multihashes, through typed errors | Conforms |

## Current Contract

### Maintained CID Path

The current app-data and core crates use:

- `cid` for CID parsing and construction (workspace dep, consumed by
  `cow-sdk-app-data`)
- `multihash` for explicit multihash wrapping (workspace dep, consumed
  by `cow-sdk-app-data`)
- `multibase` for lowercase base16 CID rendering (workspace dep,
  consumed by `cow-sdk-app-data`)
- `alloy_primitives::keccak256` (the canonical primitive layer per
  [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md))
  for deterministic app-data digest generation
- `serde_jcs` for RFC 8785 canonical-JSON serialisation of the document
  whose bytes feed the `alloy_primitives::keccak256` digest input

The cow `AppDataHash` type is a cow-owned `#[repr(transparent)]` newtype
over `alloy_primitives::B256` per
[ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md). The
canonical CID conversion lives in the `cow-sdk-app-data` `cid` module: the
forward path builds the CIDv1 raw-keccak256 string form documented under
"Supported Input Boundary" below, and the reverse parser closes the
round-trip seam against the same shape. Both routes go through the
maintained `cid` + `multihash` + `multibase` trio (rather than a
hand-rolled byte assembly). `cow-sdk-core` carries no CID dependency or
CID surface of its own.

The canonical-JSON pass runs through `serde_jcs::to_string` so the key
ordering follows the RFC 8785 UTF-16 code-unit rule; this closes a latent
divergence with the upstream `@cowprotocol/cow-sdk` TypeScript canonical
form for documents whose object keys carry non-ASCII code points where
UTF-16 ordering and UTF-8 byte ordering disagree. ASCII-only documents
serialise byte-identically under both orderings.

### Supported Input Boundary

The supported CID input is intentionally narrow:

- app-data CID: CIDv1, raw codec (`0x55`), keccak-256 multihash (`0x1b`),
  32-byte digest

Rejected inputs include malformed app-data hex, malformed CID strings, wrong
digest lengths, unsupported multicodec values, unsupported multihash
values, and every non-CIDv1 version. CIDv0 (`Qm...` / dag-pb /
sha2-256), CIDv1 raw sha2-256, CIDv1 raw sha3-512, and CIDv1 raw
blake2b-256 are surfaced as typed rejections at the decoder boundary.

### Published Upstream Dependency Posture

The app-data crate now carries `cid 0.11.3`, which removes the prior
`cid 0.11.1` to `core2 0.4.0` transitive path. The CID boundary remains on
published crates and no longer needs a reviewed yanked-upstream exception for
the CID stack.

### Advisory Posture

The `cargo-deny` advisory gate denies yanked crates, and the canonical RustSec
ignore register no longer includes the prior CID-chain exceptions. The remaining
`cargo audit` ignores belong to the browser-wallet alloy helper posture and are
tracked in [Browser-Wallet Alloy Dependency Audit](browser-wallet-alloy-dependency-audit.md).
`RUSTSEC-2026-0105` is no longer tolerated because `core2` is no longer
reachable from the app-data dependency graph.
The workspace-wide RustSec command is recorded in
[Dependency Gate Audit](dependency-gate-audit.md); this CID audit does not own
any `cargo audit --ignore` entry.

Revisit trigger:

- Refresh this audit whenever `cid`, `multihash`, or `multibase` move again, or
  if a new advisory reaches the supported CID conversion path.

## Evidence

Primary implementation points:

- `crates/app-data/src/lib.rs`
- `crates/app-data/src/cid.rs`

Primary regression coverage:

- `crates/app-data/tests/cid_contract.rs::latest_cid_conversion_matches_upstream_samples`
- `crates/app-data/tests/cid_contract.rs::cid_digest_extraction_supports_the_supported_cid_shape`
- `crates/app-data/tests/cid_contract.rs::invalid_app_data_hex_inputs_fail_closed`
- `crates/app-data/tests/cid_contract.rs::unsupported_and_malformed_cids_are_rejected`
- `crates/app-data/tests/cid_contract.rs::cid_rejects_non_keccak256_multihash_codecs`
- `crates/app-data/tests/canonical_json_contract.rs::canonical_json_utf16_corpus_serialises_to_expected_canonical_bytes`
- `parity/fixtures/app_data/canonical_json_utf16.json`

Validation surface:

```text
cargo tree -p cow-sdk-app-data -d
cargo tree -p cow-sdk-app-data -e normal
cargo deny check --config .github/config/deny.toml
cargo test -p cow-sdk-app-data
cargo clippy -p cow-sdk-app-data --all-targets --all-features -- -D warnings
```
