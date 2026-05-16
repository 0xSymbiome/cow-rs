//! Proxy creation-code SHA-256 contract test: assert the committed
//! `.bin` files at
//! `crates/contracts/abi/cow-shed/proxy-creation-code/` hash to
//! exactly the values in their adjacent `.sha256` neighbors.
//!
//! The same invariant runs inside `crates/contracts/build.rs::
//! validate_cow_shed_proxy_artifacts` at build time; this test
//! catches the same regression as a runtime test as well so a
//! manual `cargo test` exercise of the contracts crate surfaces
//! the drift even when build-script caching is bypassed.

use sha2::{Digest, Sha256};
use std::path::PathBuf;

const VERSIONS: &[&str] = &["v1.0.0", "v1.0.1"];

fn proxy_code_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("abi")
        .join("cow-shed")
        .join("proxy-creation-code")
}

#[test]
fn every_proxy_creation_code_bin_matches_committed_sha256() {
    let dir = proxy_code_dir();
    for version in VERSIONS {
        let bin = dir.join(format!("{version}.bin"));
        let digest_path = dir.join(format!("{version}.bin.sha256"));
        let bytes = std::fs::read(&bin)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", bin.display()));
        let committed = std::fs::read_to_string(&digest_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", digest_path.display()));
        let committed = committed.trim();
        let computed = format!("{:x}", Sha256::digest(&bytes));
        assert_eq!(
            committed, computed,
            "committed sha256 for {version}.bin must match the runtime digest of the .bin file; committed={committed}, computed={computed}"
        );
    }
}

#[test]
fn proxy_code_artifacts_are_non_empty() {
    let dir = proxy_code_dir();
    for version in VERSIONS {
        let bin = dir.join(format!("{version}.bin"));
        let bytes = std::fs::read(&bin)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", bin.display()));
        assert!(
            !bytes.is_empty(),
            "{version}.bin must be non-empty; pre-deploy signing depends on the byte content"
        );
    }
}
