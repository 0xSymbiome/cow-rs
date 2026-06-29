#![cfg(not(target_arch = "wasm32"))]

use std::{fs, path::PathBuf};

use serde_json::Value;

#[derive(Debug, Clone)]
struct FacadeSnapshot {
    name: String,
    features: Vec<String>,
}

impl FacadeSnapshot {
    fn has_feature(&self, feature: &str) -> bool {
        self.features.iter().any(|candidate| candidate == feature)
    }
}

#[test]
fn facade_declarations_match_flavour_matrix() {
    for snapshot in snapshots() {
        let path = snapshot_path(&snapshot.name);
        assert!(
            path.exists(),
            "{} facade snapshot must exist",
            snapshot.name
        );
    }
}

#[test]
fn facade_declarations_hide_raw_wasm_bindgen_surface() {
    let forbidden = [
        "app_data_hex",
        "chain_id",
        "custom_callback",
        "digest_signer",
        "fetch_callback",
        "order_uid",
        "order_uids",
        "request_callback",
        "timeout_ms",
        "typed_data_signer",
        concat!("With", "Fetch"),
        concat!("register", "FetchCallback"),
        concat!("FetchCallback", "Handle"),
        "Function",
        // The raw wasm-bindgen `free()` stays hidden behind the facade's
        // `dispose()`. `[Symbol.dispose]` (and its `esnext.disposable` lib
        // reference) are part of the facade contract now — clients implement it
        // so `using` works — so they are intentionally allowed.
        "free(): void",
    ];

    for snapshot in snapshots() {
        let content = read_snapshot(&snapshot.name);
        for token in forbidden {
            assert!(
                !content.contains(token),
                "{} facade snapshot must not expose `{token}`",
                snapshot.name
            );
        }
    }
}

#[test]
fn facade_declarations_expose_dispose_and_named_callback_types() {
    for snapshot in snapshots() {
        let content = read_snapshot(&snapshot.name);
        if snapshot.has_feature("orderbook")
            || snapshot.has_feature("ipfs")
            || snapshot.has_feature("subgraph")
            || snapshot.has_feature("trading")
        {
            assert!(
                content.contains("dispose(): void"),
                "{} must expose dispose() for client resources",
                snapshot.name
            );
            assert!(
                content.contains("[Symbol.dispose]"),
                "{} must expose [Symbol.dispose] so `using` releases client resources",
                snapshot.name
            );
        }
        assert!(
            content.contains("TypedDataSignerCallback"),
            "{} must expose named signing callbacks",
            snapshot.name
        );
        assert!(
            content.contains("CowError"),
            "{} must expose normalized SDK errors",
            snapshot.name
        );
    }
}

#[test]
fn facade_declarations_keep_feature_scoped_client_classes() {
    for snapshot in snapshots() {
        let content = read_snapshot(&snapshot.name);
        assert_feature_class(&snapshot, &content, "ipfs", "IpfsClient");
        assert_feature_class(&snapshot, &content, "orderbook", "OrderBookClient");
        assert_feature_class(&snapshot, &content, "subgraph", "SubgraphClient");
        assert_feature_class(&snapshot, &content, "trading", "TradingClient");
    }
}

fn assert_feature_class(snapshot: &FacadeSnapshot, content: &str, feature: &str, class_name: &str) {
    let token = format!("declare class {class_name}");
    if snapshot.has_feature(feature) {
        assert!(
            content.contains(&token),
            "{} must expose `{token}`",
            snapshot.name
        );
    } else {
        assert!(
            !content.contains(&token),
            "{} must not expose `{token}`",
            snapshot.name
        );
    }
}

fn snapshots() -> Vec<FacadeSnapshot> {
    let descriptor_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("npm")
        .join("flavours.json");
    let descriptor: Value = serde_json::from_str(
        &fs::read_to_string(descriptor_path).expect("flavours.json must be readable"),
    )
    .expect("flavours.json must be valid JSON");
    descriptor["flavours"]
        .as_array()
        .expect("flavours must be an array")
        .iter()
        .map(|flavour| FacadeSnapshot {
            name: format!("{}.d.ts", flavour["name"].as_str().expect("flavour name")),
            features: flavour["features"]
                .as_array()
                .expect("flavour features")
                .iter()
                .map(|feature| feature.as_str().expect("feature name").to_owned())
                .collect(),
        })
        .collect()
}

fn read_snapshot(name: &str) -> String {
    fs::read_to_string(snapshot_path(name)).expect("facade snapshot must be readable")
}

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("snapshots")
        .join("facade")
        .join(name)
}
