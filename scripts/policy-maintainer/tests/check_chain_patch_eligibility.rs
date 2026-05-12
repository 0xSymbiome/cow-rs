use policy_maintainer::check_chain_patch_eligibility::{analyze_diff, validate_diff};

const CHAIN_DIFF: &str = "\
diff --git a/crates/core/src/config/chains.rs b/crates/core/src/config/chains.rs
+++ b/crates/core/src/config/chains.rs
@@
+    Testnet = 12345,
";

#[test]
fn chain_patch_eligibility_ignores_diffs_without_chain_additions() {
    let diff = "\
diff --git a/README.md b/README.md
+++ b/README.md
@@
+docs
";

    let report = analyze_diff(diff);
    assert!(report.added_chain_ids.is_empty());
    assert!(validate_diff(diff, "").is_empty());
}

#[test]
fn chain_patch_eligibility_accepts_chain_already_in_source_lock() {
    let source_lock = "repositories:\n- producer_paths: []\n  authority: chain_id: 12345\n";

    assert!(validate_diff(CHAIN_DIFF, source_lock).is_empty());
}

#[test]
fn chain_patch_eligibility_rejects_missing_authority_or_source_lock_refresh() {
    assert!(
        validate_diff(CHAIN_DIFF, "")
            .iter()
            .any(|error| { error.contains("not visible in the source-lock authority text") })
    );

    let refreshed = format!("{CHAIN_DIFF}+++ b/parity/source-lock.yaml\n+commit: abc\n");
    assert!(
        validate_diff(&refreshed, "chain_id: 12345")
            .iter()
            .any(|error| error.contains("source-lock refresh"))
    );
}
