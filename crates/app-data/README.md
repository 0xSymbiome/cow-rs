# cow-sdk-app-data

[CoW Protocol](https://cow.fi) app-data generation, schema validation,
CID conversion, and IPFS transport seams.

`appData` is the canonical metadata attached to every CoW Protocol order.
This crate produces deterministic app-data documents, validates them
against the versioned app-data schema, and converts between the 32-byte
hex hash form and both the latest (CIDv1 + raw + keccak-256) and legacy
(CIDv0 + dag-pb + sha2-256) CID encodings. It also defines the fetch and
upload transport seams so consumers can provide their own IPFS client
without coupling the SDK to a specific HTTP stack.

## Install

```toml
[dependencies]
cow-sdk-app-data = "0.1"
```

## Minimal example

```rust
use cow_sdk_app_data::{app_data_hex_to_cid, cid_to_app_data_hex};

let hex = "0x0000000000000000000000000000000000000000000000000000000000000000";
let cid = app_data_hex_to_cid(hex).unwrap();
let roundtrip = cid_to_app_data_hex(&cid).unwrap();
assert_eq!(roundtrip, hex);
```

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [CID Dependency Audit](https://github.com/cowdao-grants/cow-rs/blob/main/docs/audit/cid-dependency-audit.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
