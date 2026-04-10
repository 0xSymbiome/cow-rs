# CID Dependency Audit

Last reviewed: 2026-04-10

This audit records the `cow-sdk-app-data` CID dependency surface, the current dependency decision, and the validation evidence for latest and legacy app-data CID behavior.

## Audit Scope

Covered in this revision:

- direct CID and multihash dependencies used by `cow-sdk-app-data`,
- latest and legacy app-data CID construction paths,
- fail-closed handling for malformed app-data hex and unsupported CID encodings.

Not covered in this revision:

- broader workspace duplicate crypto crates outside the app-data CID boundary,
- transport or schema dependencies unrelated to CID encoding.

## Current Result

| Area | Status | Decision |
| --- | --- | --- |
| Latest CID conversion | Current | Keep `cid`, `multihash`, and `multibase` as the maintained conversion path. |
| Legacy CID generation | Current | Remove `ipfs-cid` and generate the explicit legacy CID through `sha2` plus the same maintained `cid`/`multihash` stack. |
| Duplicate CID and multihash versions reaching `cow-sdk-app-data` | Addressed | The avoidable older `cid` and `multihash` chains were removed with `ipfs-cid`. |
| Remaining duplicate crypto support crates | Accepted | Keep the remaining `digest` and `block-buffer` duplicates because they come from distinct `sha2` and `sha3` support paths rather than from duplicated CID infrastructure. |
| Unsupported CID encodings | Addressed | Reject unsupported multicodec, unsupported multihash, wrong digest length, and malformed CID inputs through typed errors. |

## Dependency Decision

The current app-data crate uses:

- `cid` for CID parsing and construction,
- `multihash` for explicit multihash wrapping,
- `multibase` for the latest lowercase base16 CID rendering,
- `sha2` for the legacy compatibility digest path,
- `sha3` for deterministic latest app-data digest generation.

This keeps latest and legacy CID behavior on one maintained Rust CID stack instead of splitting the legacy path through `ipfs-cid` and its older transitive `cid` and `multihash` versions.

The remaining duplicate entries in `cargo tree -d` are expected support crates for separate hash families:

- `sha2` remains necessary for the explicit legacy CIDv0 compatibility path,
- `sha3` remains necessary for the current keccak-based app-data path.

That remaining duplication is not treated as a local hardening defect because it does not reintroduce multiple CID stacks, does not change the public contract, and would require churn across upstream crypto ecosystems rather than a focused app-data boundary fix.

## Behavior Boundary

Supported CID inputs are intentionally narrow:

- latest app-data CID: CIDv1, raw codec (`0x55`), keccak-256 multihash (`0x1b`), 32-byte digest,
- legacy compatibility CID: CIDv0 / `Qm...`, dag-pb codec (`0x70`), sha2-256 multihash (`0x12`), 32-byte digest.

Rejected inputs include:

- malformed app-data hex strings,
- malformed CID strings,
- wrong digest lengths,
- unsupported multicodec values,
- unsupported multihash values.

## Evidence

Validation commands used for this audit:

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
