#[test]
fn orderbook_schema_sources_are_pinned_and_test_only() {
    let source_lock = include_str!("../../../parity/source-lock.yaml");
    let parity_scope = include_str!("../../../docs/parity-scope.md");

    for source in [
        "remote: https://github.com/cowprotocol/cow-sdk.git",
        "remote: https://github.com/cowprotocol/composable-cow.git",
        "remote: https://github.com/cowdao-grants/cow-shed.git",
        "remote: https://github.com/cowprotocol/watch-tower.git",
        "packages/cow-shed/src/const.ts",
        "src/ComposableCoW.sol",
        "src/COWShedFactory.sol",
    ] {
        assert!(source_lock.contains(source), "missing source {source}");
    }

    assert!(parity_scope.contains("Orderbook"));
    assert!(parity_scope.contains("crates/orderbook/openapi.yml"));
    assert!(parity_scope.contains("non-public"));
    assert!(parity_scope.contains("test-only"));
}
