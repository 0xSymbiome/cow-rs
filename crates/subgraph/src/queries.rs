/// Canonical subgraph operation sources remain private saved GraphQL documents.
/// The public API stays at reviewed query constants and DTOs rather than a
/// generated schema surface.
pub const TOTALS_QUERY: &str = include_str!("query_documents/totals.graphql");

/// Canonical subgraph operation sources remain private saved GraphQL documents.
pub const LAST_DAYS_VOLUME_QUERY: &str = include_str!("query_documents/last_days_volume.graphql");

/// Canonical subgraph operation sources remain private saved GraphQL documents.
pub const LAST_HOURS_VOLUME_QUERY: &str = include_str!("query_documents/last_hours_volume.graphql");
