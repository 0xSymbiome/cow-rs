# CID Dependency Audit

Status: Current  
Last reviewed: 2026-04-16  
Owning surface: `cow-sdk-app-data` CID encoding and published dependency boundary  
Refresh trigger: Changes to CID dependencies, supported CID encodings, legacy compatibility logic, or the published dependency posture for the app-data stack, or a new `cid` or `core2` release that moves the reviewed warning state  
Related docs:
- [Dependency Gate Audit](dependency-gate-audit.md)
- [Verification Guide](../verification-guide.md)
- [Verification Matrix](../verification-matrix.md)

## Scope

This audit covers:

- the CID and multihash dependencies used by `cow-sdk-app-data`
- latest and legacy app-data CID construction paths
- published-upstream dependency posture for the maintained CID stack
- fail-closed handling for malformed app-data hex and unsupported CID
  encodings

It does not cover broader workspace TLS, HTTP, or non-CID dependency policy
outside the app-data boundary.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Latest CID conversion | Keep `cid`, `multihash`, and `multibase` as the maintained path | Conforms |
| Legacy CID generation | Keep `ipfs-cid` removed and generate the legacy CID through `sha2` plus the maintained `cid` and `multihash` stack | Conforms |
| Published upstream dependency posture | Carry the current `cid 0.11.1` to `core2 0.4.0` yanked reachability as an explicit reviewed warning until a published replacement exists | Reviewed warning |
| Unsupported CID encodings | Reject malformed or unsupported inputs through typed errors | Conforms |

## Current Contract

### Maintained CID Path

The current app-data crate uses:

- `cid` for CID parsing and construction
- `multihash` for explicit multihash wrapping
- `multibase` for lowercase base16 CID rendering
- `sha2` for the legacy compatibility digest path
- `sha3` for deterministic latest app-data digest generation

### Supported Input Boundary

Supported CID inputs are intentionally narrow:

- latest app-data CID: CIDv1, raw codec (`0x55`), keccak-256 multihash
  (`0x1b`), 32-byte digest
- legacy compatibility CID: CIDv0 / `Qm...`, dag-pb codec (`0x70`),
  sha2-256 multihash (`0x12`), 32-byte digest

Rejected inputs include malformed app-data hex, malformed CID strings, wrong
digest lengths, unsupported multicodec values, and unsupported multihash
values.

### Published Upstream Dependency Posture

The refreshed published dependency path now carries the current `multihash`
release, but the remaining `core2 0.4.0` reachability still comes from the
latest published `cid 0.11.1` release. The repository therefore records that
state as a reviewed warning instead of replacing the published dependency with
an unreleased override.

### Advisory Posture

RustSec advisory [`RUSTSEC-2026-0097`](https://rustsec.org/advisories/RUSTSEC-2026-0097)
reaches this workspace only through the published `cid 0.11.1` to `core2 0.4.0`
chain documented above. The `cargo audit` gate therefore blocks every other
unsound and unmaintained advisory while explicitly tolerating this single
identifier through `--ignore RUSTSEC-2026-0097`. The same identifier is
recorded in `.github/config/deny.toml` under `[advisories].ignore` with a
matching expiry comment so the policy lives in one reviewable place instead of
hiding inside a CI command line.

Revisit trigger for this advisory:

- Drop the ignore the first time a published `cid` release no longer reaches
  `core2 0.4.0` through any transitive path, or the first time `core2`
  publishes a maintained successor that unblocks the maintained CID path.
- Calendar floor: re-review the advisory and the upstream state every 90
  days even if no upstream movement has occurred, and update
  `Last reviewed` together with the deny.toml comment.
- If either trigger fires, refresh this audit, remove the `cargo audit
  --ignore` flag from `.github/workflows/ci.yml` and
  `.github/workflows/release-readiness.yml`, and remove the matching entry
  from `.github/config/deny.toml`.

## Evidence

Primary implementation points:

- `crates/app-data/src/lib.rs`
- `crates/app-data/src/cid.rs`

Primary regression coverage:

- `crates/app-data/tests/cid_contract.rs::latest_and_legacy_cid_conversion_match_upstream_samples`
- `crates/app-data/tests/cid_contract.rs::cid_digest_extraction_supports_latest_and_legacy_inputs`
- `crates/app-data/tests/cid_contract.rs::invalid_app_data_hex_inputs_fail_closed`
- `crates/app-data/tests/cid_contract.rs::unsupported_and_malformed_cids_are_rejected`
- `crates/app-data/tests/app_data_info_contract.rs::legacy_info_flow_remains_explicit_and_compatible`

Validation surface:

```text
cargo tree -p cow-sdk-app-data -d
cargo tree -i core2 -e normal
cargo audit --deny unsound --deny unmaintained --ignore RUSTSEC-2026-0097
cargo test -p cow-sdk-app-data
cargo clippy -p cow-sdk-app-data --all-targets --all-features -- -D warnings
```
