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
        .join(name);
    fs::read_to_string(path).expect("snapshot must be readable")
}
