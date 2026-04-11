//! Typed CoW Protocol subgraph queries.
//!
//! `cow-sdk-subgraph` keeps saved query documents, explicit raw-query inputs,
//! and typed error boundaries in a dedicated crate instead of widening the root
//! facade with GraphQL transport behavior.

/// Typed subgraph client configuration and query execution.
pub mod api;
/// Typed subgraph transport, GraphQL, and decoding errors.
pub mod error;
/// Saved query documents exposed as stable helper constants.
pub mod queries;
/// Public request and response DTOs for the subgraph surface.
pub mod types;

pub use api::{
    API_NAME, DEFAULT_SUBGRAPH_USER_AGENT, SubgraphApi, SubgraphApiBaseUrls, SubgraphConfig,
    SubgraphConfigOverride, SubgraphTransportPolicy,
};
pub use error::{
    SubgraphError, SubgraphGraphQlError, SubgraphGraphQlErrorLocation, SubgraphRequestErrorContext,
};
pub use queries::{LAST_DAYS_VOLUME_QUERY, LAST_HOURS_VOLUME_QUERY, TOTALS_QUERY};
pub use types::{
    DailyTotal, HourlyTotal, LastDaysVolumeResponse, LastHoursVolumeResponse, SubgraphQueryRequest,
    Total, TotalsResponse,
};
