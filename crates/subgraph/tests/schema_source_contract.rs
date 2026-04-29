#[test]
fn subgraph_schema_sources_are_pinned_and_test_only() {
    let source_lock = include_str!("../../../parity/source-lock.yaml");
    let parity_scope = include_str!("../../../docs/parity-scope.md");
    let lib_rs = include_str!("../src/lib.rs");
    let totals_document = include_str!("../src/query_documents/totals.graphql");
    let last_days_document = include_str!("../src/query_documents/last_days_volume.graphql");
    let last_hours_document = include_str!("../src/query_documents/last_hours_volume.graphql");
    let schema_snapshot = include_str!("schema_evidence/schema.graphql");

    for source in [
        "remote: https://github.com/cowprotocol/cow-sdk.git",
        "commit: 00c3dbd41c086ff9a51d5e5a30648615d4c66d0d",
        "packages/subgraph/src/api.ts",
        "packages/subgraph/src/api.spec.ts",
        "packages/subgraph/src/graphql.ts",
        "packages/subgraph/src/queries.ts",
    ] {
        assert!(source_lock.contains(source), "missing source {source}");
    }

    assert!(parity_scope.contains("Subgraph"));
    assert!(parity_scope.contains("packages/subgraph/src/queries.ts"));
    assert!(parity_scope.contains("non-public"));
    assert!(parity_scope.contains("test-only"));
    assert!(parity_scope.contains("crates/subgraph/src/query_documents"));
    assert!(parity_scope.contains("crates/subgraph/tests/schema_evidence"));

    assert!(lib_rs.contains("pub mod queries;"));
    assert!(!lib_rs.contains("query_documents"));
    assert!(!lib_rs.contains("schema_evidence"));

    assert!(totals_document.starts_with("query Totals"));
    assert!(last_days_document.starts_with("query LastDaysVolume"));
    assert!(last_hours_document.starts_with("query LastHoursVolume"));
    assert!(schema_snapshot.contains("type Query"));
    assert!(schema_snapshot.contains("enum OrderDirection"));
}
