#![cfg(not(target_arch = "wasm32"))]

use std::{fs, path::PathBuf};

const SNAPSHOTS: &[&str] = &[
    "cow_sdk_wasm_web.d.ts",
    "cow_sdk_wasm_bundler.d.ts",
    "cow_sdk_wasm_nodejs.d.ts",
];

#[test]
fn generated_type_declarations_hide_callback_registry() {
    let forbidden = [
        concat!("FetchCallback", "Handle"),
        concat!("register", "FetchCallback"),
        concat!("from", "Handle"),
        concat!("With", "Fetch"),
        concat!("HttpTo", "IpfsAdapter"),
    ];

    for snapshot in SNAPSHOTS {
        let content = read_snapshot(snapshot);
        for token in forbidden {
            assert!(
                !content.contains(token),
                "{snapshot} must not expose `{token}`"
            );
        }
    }
}

#[test]
fn generated_type_declarations_use_camel_case_public_params() {
    let forbidden = [
        "app_data_hex:",
        "chain_id:",
        "custom_callback:",
        "digest_signer:",
        "ecdsa_signature:",
        "order_uid:",
        "order_uids:",
        "request_callback:",
        "signer_callback:",
        "typed_data_signer:",
    ];

    for snapshot in SNAPSHOTS {
        let content = read_snapshot(snapshot);
        for token in forbidden {
            assert!(
                !content.contains(token),
                "{snapshot} must not expose `{token}`"
            );
        }
    }
}

#[test]
fn generated_type_declarations_name_callback_params() {
    let expected = [
        "digestSigner: DigestSignerCallback",
        "requestCallback: Eip1193RequestCallback",
        "signerCallback: TypedDataSignerCallback",
        "typedDataSigner: TypedDataSignerCallback",
        "customCallback: CustomEip1271Callback",
    ];

    for snapshot in SNAPSHOTS {
        let content = read_snapshot(snapshot);
        for token in expected {
            assert!(content.contains(token), "{snapshot} must expose `{token}`");
        }
    }
}

#[test]
fn generated_type_declarations_version_errors_and_outputs() {
    let expected = [
        "export type SchemaVersion = \"v1\" | \"__unknown\";",
        "export interface WasmEnvelope<T>",
        "schemaVersion: SchemaVersion;",
        "kind: \"forbiddenInteraction\"",
        "kind: \"__unknown\"",
    ];
    let forbidden = ["Promise<Promise<"];

    for snapshot in SNAPSHOTS {
        let content = read_snapshot(snapshot);
        for token in expected {
            assert!(content.contains(token), "{snapshot} must expose `{token}`");
        }
        for token in forbidden {
            assert!(
                !content.contains(token),
                "{snapshot} must not expose `{token}`"
            );
        }
    }
}

#[test]
fn generated_type_declarations_keep_unknown_escape_hatch_scoped() {
    let expected = [
        "message: Value;",
        "params?: Value[];",
        "variables?: Value;",
        "raw: Value",
    ];
    let forbidden = [
        "input: Value",
        "request: Value",
        "params: Value",
        "signed: Value",
    ];

    for snapshot in SNAPSHOTS {
        let content = read_snapshot(snapshot);
        for token in expected {
            assert!(content.contains(token), "{snapshot} must expose `{token}`");
        }
        for token in forbidden {
            assert!(
                !content.contains(token),
                "{snapshot} must not expose primary input as `{token}`"
            );
        }
    }
}

#[test]
fn generated_type_declarations_keep_single_client_classes() {
    let expected = [
        "export class IpfsClient",
        "export class OrderBookClient",
        "export class SubgraphClient",
        "export class TradingClient",
    ];

    for snapshot in SNAPSHOTS {
        let content = read_snapshot(snapshot);
        for token in expected {
            assert!(content.contains(token), "{snapshot} must expose `{token}`");
        }
    }
}

fn read_snapshot(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("snapshots")
        .join("raw")
        .join(name);
    fs::read_to_string(path).expect("snapshot must be readable")
}
