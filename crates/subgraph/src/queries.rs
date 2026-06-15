//! Saved GraphQL documents for the canonical subgraph helpers.

/// Reviewed GraphQL document exposed as a stable `&'static str` constant. The
/// public surface stays at curated documents and DTOs rather than a generated
/// schema.
pub const TOTALS_QUERY: &str = include_str!("query_documents/totals.graphql");

/// Reviewed GraphQL document exposed as a stable `&'static str` constant.
pub const LAST_DAYS_VOLUME_QUERY: &str = include_str!("query_documents/last_days_volume.graphql");

/// Reviewed GraphQL document exposed as a stable `&'static str` constant.
pub const LAST_HOURS_VOLUME_QUERY: &str = include_str!("query_documents/last_hours_volume.graphql");
