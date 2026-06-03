# cow-sdk-app-data

[CoW Protocol](https://cow.fi) app-data document generation, schema
validation, CID conversion, and the IPFS read transport seam.

`appData` is the canonical metadata attached to every CoW Protocol order.
This crate builds deterministic app-data documents, validates them against
the versioned app-data JSON schema, computes their keccak256 digest, and
converts between the 32-byte hex hash and the supported CID encoding
(CIDv1 + raw codec + keccak-256 multihash).

Registering a document is an orderbook concern: hash it locally with this
crate, then submit the full document through `OrderbookApi::upload_app_data`,
which stores it under its hash. The IPFS read seam is the secondary path, for
resolving a document by hash directly from a gateway when it is not available
through the orderbook.

## Install

```toml
[dependencies]
cow-sdk-app-data = "0.1"
```

## Minimal example

Tag a document with a validated `AppCode`, validate it against the bundled
JSON schema, and compute the canonical content and keccak256 digest in a
single call. Chain `with_*` setters for environment, signer, hooks, flashloan
hints, or open-ended metadata before the terminal `into_validated`:

```rust
use cow_sdk_core::AppCode;
use cow_sdk_app_data::AppDataParams;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let code = AppCode::new("my-app")?;
let validated = AppDataParams::new(code)
    .with_environment("production")
    .into_validated()?;

// Ready to register through the orderbook:
//   PUT /api/v1/app_data/{hash}
//     hash = validated.info.app_data_hex      (0x-prefixed keccak256 digest)
//     body = validated.info.app_data_content  (canonical JSON)
assert_eq!(validated.info.app_data_hex.len(), 66); // "0x" + 32-byte digest
# Ok(())
# }
```

## CID conversion

Convert between the 32-byte app-data hash and its CID form. The transform is
pure and offline — no network — and round-trips losslessly:

```rust
use cow_sdk_app_data::{app_data_hex_to_cid, cid_to_app_data_hex};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let hex = "0x0000000000000000000000000000000000000000000000000000000000000000";
let cid = app_data_hex_to_cid(hex)?;
assert_eq!(cid_to_app_data_hex(&cid)?, hex);
# Ok(())
# }
```

## Reading a document from IPFS

The primary way to read a document you registered is the orderbook
`GET /api/v1/app_data/{hash}` request, served from the orderbook database with
no gateway involved. The IPFS read seam is the secondary, not-in-database
path: it derives the keccak-256 CIDv1 from an app-data hash and reads it
through a fetch transport you supply, so the SDK stays decoupled from any
specific HTTP stack.

The seam is `async`, so native and browser runtimes can plug in their own HTTP
client; `cow-sdk-wasm` implements it over JavaScript's `CowFetchCallback`, and
browser and non-browser wasm runtimes share the same app-data contract.
Because documents are addressed by a keccak-256 CID, the gateway must be able
to resolve keccak-CID documents — a generic public gateway cannot.

```rust,no_run
use cow_sdk_app_data::{AppDataError, IpfsFetchTransport, fetch_doc_from_app_data_hex};

struct IpfsClient;

#[async_trait::async_trait]
impl IpfsFetchTransport for IpfsClient {
    async fn get(&self, uri: &str) -> Result<String, AppDataError> {
        // Issue the GET with your HTTP client of choice and return the body.
        let _ = uri;
        Ok(r#"{"version":"1.4.0","metadata":{}}"#.to_owned())
    }
}

# async fn example(client: &IpfsClient) -> Result<(), AppDataError> {
let app_data_hex =
    "0x337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df";
let doc = fetch_doc_from_app_data_hex(app_data_hex, client, None).await?;
assert_eq!(doc["version"], "1.4.0");
# Ok(())
# }
```

Pass `None` to read from the default CoW gateway, or `Some(uri)` to target a
specific keccak-CID-capable gateway. When you already hold a CID rather than a
hash, `fetch_doc_from_cid` takes the same transport.

## Canonical JSON

App-data document canonicalisation routes through `serde_jcs::to_string` per
RFC 8785 (JSON Canonicalization Scheme). The serializer sorts object keys
by UTF-16 code unit value and emits a deterministic byte sequence for any
equivalent document shape, so the resulting CID is byte-identical to the
canonical form the upstream `@cowprotocol/cow-sdk` TypeScript helper would
produce for the same input. Documents whose object keys carry code points
whose UTF-16 ordering and UTF-8 byte ordering disagree are pinned by
`parity/fixtures/app_data/canonical_json_utf16.json`; ASCII-only documents
are byte-identical to any earlier bytewise canonicalisation.

The cow `AppDataHash` is a cow-owned `#[repr(transparent)]` newtype over
`alloy_primitives::B256` per
[ADR 0052](https://github.com/cowdao-grants/cow-rs/blob/main/docs/adr/0052-alloy-primitives-canonical-primitive-layer.md);
the canonical CID conversion lives on the inherent method
`AppDataHash::to_cid`. The digest input fed to
`alloy_primitives::keccak256` is the canonical-JSON byte stream produced
by `serde_jcs`.

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [CID Dependency Audit](https://github.com/cowdao-grants/cow-rs/blob/main/docs/audit/cid-dependency-audit.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
