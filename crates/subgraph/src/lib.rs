#![cfg_attr(any(doctest, docsrs), doc = include_str!("../README.md"))]

//! Typed `CoW` Protocol subgraph queries.
//!
//! `cow-sdk-subgraph` keeps saved query documents, explicit raw-query inputs,
//! and typed error boundaries in a dedicated crate instead of widening the root
//! facade with GraphQL transport behavior.

#![warn(missing_docs)]

/// Typed subgraph client configuration and query execution.
pub mod api;
/// Typestate-checked construction surface for [`SubgraphApi`].
pub mod builder;
/// Typed subgraph transport, GraphQL, and decoding errors.
pub mod error;
/// Saved query documents exposed as stable helper constants.
pub mod queries;
/// Public request and response DTOs for the subgraph surface.
pub mod types;

pub use api::{API_NAME, SubgraphApi, SubgraphApiBaseUrls, SubgraphConfig, SubgraphConfigOverride};
pub use builder::{
    ApiKeySet, ApiKeyUnset, ChainIdSet, ChainIdUnset, SubgraphApiBuilder, TransportSet,
    TransportUnset,
};
pub use error::{
    SubgraphError, SubgraphGraphQlError, SubgraphGraphQlErrorLocation, SubgraphRequestErrorContext,
};
pub use queries::{LAST_DAYS_VOLUME_QUERY, LAST_HOURS_VOLUME_QUERY, TOTALS_QUERY};
pub use types::{
    DailyTotal, HourlyTotal, LastDaysVolumeResponse, LastHoursVolumeResponse, SubgraphQueryRequest,
    Total, TotalsResponse,
};

pub use cow_sdk_core::{ExternalHostPolicy, HostPolicyError};
