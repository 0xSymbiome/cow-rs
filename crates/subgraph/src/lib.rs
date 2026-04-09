pub mod api;
pub mod error;
pub mod queries;
pub mod types;

pub use api::{API_NAME, SubgraphApi, SubgraphApiBaseUrls, SubgraphConfig, SubgraphConfigOverride};
pub use error::SubgraphError;
pub use queries::{LAST_DAYS_VOLUME_QUERY, LAST_HOURS_VOLUME_QUERY, TOTALS_QUERY};
pub use types::{
    DailyTotal, HourlyTotal, LastDaysVolumeResponse, LastHoursVolumeResponse, Total, TotalsResponse,
};
