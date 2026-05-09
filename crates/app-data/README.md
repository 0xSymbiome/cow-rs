# cow-sdk-app-data

[CoW Protocol](https://cow.fi) app-data generation, schema validation,
CID conversion, and IPFS transport seams.

`appData` is the canonical metadata attached to every CoW Protocol order.
This crate produces deterministic app-data documents, validates them
against the versioned app-data schema, and converts between the 32-byte
hex hash form and the supported CID encoding (CIDv1 + raw + keccak-256).
It also defines the fetch and upload transport seams so consumers can
provide their own IPFS client without coupling the SDK to a specific
HTTP stack.

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

## IPFS fetch transport

The fetch seam is async so native and browser runtimes can supply their own
HTTP implementation without blocking the caller.

`cow-sdk-wasm` uses the same async fetch seam for IPFS reads when JavaScript
provides HTTP dispatch through `CowFetchCallback`, so browser and non-browser
wasm runtimes share the same app-data contract.

```rust,no_run
use cow_sdk_app_data::{AppDataError, IpfsFetchTransport, fetch_doc_from_cid};

struct IpfsClient;

#[async_trait::async_trait]
impl IpfsFetchTransport for IpfsClient {
    async fn get(&self, uri: &str) -> Result<String, AppDataError> {
        let _ = uri;
        Ok(r#"{"version":"1.4.0","metadata":{}}"#.to_owned())
    }
}

# async fn example(client: &IpfsClient) -> Result<(), AppDataError> {
let doc = fetch_doc_from_cid("bafybeiany", client, None).await?;
assert_eq!(doc["version"], "1.4.0");
# Ok(())
# }
```

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [CID Dependency Audit](https://github.com/cowdao-grants/cow-rs/blob/main/docs/audit/cid-dependency-audit.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
