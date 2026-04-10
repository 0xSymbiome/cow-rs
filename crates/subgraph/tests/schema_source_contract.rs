#[test]
fn subgraph_schema_sources_are_pinned_and_test_only() {
    let source_lock = include_str!("../../../parity/source-lock.yaml");
    let parity_scope = include_str!("../../../docs/parity-scope.md");

    for source in [
        "remote: https://github.com/cowprotocol/cow-sdk.git",
        "commit: 17fcfc590be8529dc4fe05b1c472fef1b07b47f4",
        "packages/subgraph/src/api.ts",
        "packages/subgraph/src/api.spec.ts",
        "packages/subgraph/src/graphql.ts",
        "packages/subgraph/src/queries.ts",
    ] {
        assert!(source_lock.contains(source), "missing source {source}");
    }

    assert!(parity_scope.contains("Subgraph"));
    assert!(parity_scope.contains("packages/subgraph/src/queries.ts"));
    assert!(parity_scope.contains("internal or test-only"));
}
