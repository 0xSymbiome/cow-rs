//! Shared app-data types, constants, and configuration structs.

pub use self::{doc::*, ipfs::*, params::*, partner_fee::*, validation::*};

mod doc;
mod ipfs;
mod params;
mod partner_fee;
mod validation;

/// Default `appCode` value inserted by [`crate::generate_app_data_doc`].
pub const DEFAULT_APP_CODE: &str = "CoW Swap";
/// Default IPFS read gateway for app-data documents.
///
/// The gateway must resolve keccak-256 `CIDv1` values — the app-data CID shape —
/// which generic public IPFS gateways do not serve.
pub const DEFAULT_IPFS_READ_URI: &str = "https://gnosis.mypinata.cloud/ipfs";
/// Latest bundled app-data schema version.
pub const LATEST_APP_DATA_VERSION: &str = "1.14.0";
/// Alias for the latest bundled schema version.
pub const LATEST_SCHEMA_VERSION: &str = LATEST_APP_DATA_VERSION;
