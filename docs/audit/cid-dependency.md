# CID Dependency Audit

Status: Current  
Last reviewed: 2026-04-10

## Scope

This audit covers:

- the CID and multihash dependencies used by `cow-sdk-app-data`
- latest and legacy app-data CID construction paths
- fail-closed handling for malformed app-data hex and unsupported CID
  encodings

It does not cover broader workspace crypto duplication outside the app-data CID
boundary.

## Decision Summary

| Area | Decision |
| --- | --- |
| Latest CID conversion | Keep `cid`, `multihash`, and `multibase` as the maintained path |
| Legacy CID generation | Remove `ipfs-cid` and generate the legacy CID through `sha2` plus the maintained `cid` and `multihash` stack |
| Avoidable duplicate CID stacks | Remove them with `ipfs-cid` |
| Remaining duplicate crypto support crates | Accept them because they come from distinct `sha2` and `sha3` support paths |
| Unsupported CID encodings | Reject them through typed errors |

## Current Contract

The current app-data crate uses:

- `cid` for CID parsing and construction
- `multihash` for explicit multihash wrapping
- `multibase` for lowercase base16 CID rendering
- `sha2` for the legacy compatibility digest path
- `sha3` for deterministic latest app-data digest generation

Supported CID inputs are intentionally narrow:

- latest app-data CID: CIDv1, raw codec (`0x55`), keccak-256 multihash
  (`0x1b`), 32-byte digest
- legacy compatibility CID: CIDv0 / `Qm...`, dag-pb codec (`0x70`),
  sha2-256 multihash (`0x12`), 32-byte digest

Rejected inputs include malformed app-data hex, malformed CID strings, wrong
digest lengths, unsupported multicodec values, and unsupported multihash
values.

## Evidence

Validation commands:

```text
cargo tree -p cow-sdk-app-data -d
cargo test -p cow-sdk-app-data
cargo clippy -p cow-sdk-app-data --all-targets --all-features -- -D warnings
```

Relevant contract coverage:

- `crates/app-data/tests/cid_contract.rs::latest_and_legacy_cid_conversion_match_upstream_samples`
- `crates/app-data/tests/cid_contract.rs::cid_digest_extraction_supports_latest_and_legacy_inputs`
- `crates/app-data/tests/cid_contract.rs::invalid_app_data_hex_inputs_fail_closed`
- `crates/app-data/tests/cid_contract.rs::unsupported_and_malformed_cids_are_rejected`
- `crates/app-data/tests/app_data_info_contract.rs::legacy_info_flow_remains_explicit_and_compatible`
