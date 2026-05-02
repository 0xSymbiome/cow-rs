#[test]
fn orderbook_schema_sources_are_pinned_and_test_only() {
    let source_lock = include_str!("../../../parity/source-lock.yaml");
    let parity_scope = include_str!("../../../docs/parity-scope.md");

    for source in [
        "remote: https://github.com/cowprotocol/services.git",
        "commit: 0720b9bc15138ecc362078f505d0e3ba1c7b9883",
        "crates/orderbook/openapi.yml",
        "crates/shared/src/order_validation.rs",
        "crates/orderbook/src/app_data.rs",
        "packages/order-book/src/types.ts",
        "packages/order-book/src/api.ts",
        "packages/order-book/src/request.ts",
    ] {
        assert!(source_lock.contains(source), "missing source {source}");
    }

    assert!(parity_scope.contains("Orderbook"));
    assert!(parity_scope.contains("crates/orderbook/openapi.yml"));
    assert!(parity_scope.contains("non-public"));
    assert!(parity_scope.contains("test-only"));
}
