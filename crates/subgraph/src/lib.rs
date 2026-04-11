//! Typed CoW Protocol subgraph query helpers and transport-level error
//! boundaries.

pub mod api;
pub mod error;
pub mod queries;
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
