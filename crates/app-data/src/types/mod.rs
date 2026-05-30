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
/// The gateway must resolve keccak-256 CIDv1 values — the app-data CID shape —
/// which generic public IPFS gateways do not serve.
pub const DEFAULT_IPFS_READ_URI: &str = "https://gnosis.mypinata.cloud/ipfs";
/// Default Pinata base URI used for write operations.
pub const DEFAULT_IPFS_WRITE_URI: &str = "https://api.pinata.cloud";
/// Latest bundled app-data schema version.
pub const LATEST_APP_DATA_VERSION: &str = "1.14.0";
/// Alias for the latest bundled schema version.
pub const LATEST_SCHEMA_VERSION: &str = LATEST_APP_DATA_VERSION;
/// Latest supported quote metadata schema version.
pub const LATEST_QUOTE_METADATA_VERSION: &str = "1.1.0";
/// Latest supported referrer metadata schema version.
pub const LATEST_REFERRER_METADATA_VERSION: &str = "1.0.0";
/// Latest supported order-class metadata schema version.
pub const LATEST_ORDER_CLASS_METADATA_VERSION: &str = "0.3.0";
/// Latest supported UTM metadata schema version.
pub const LATEST_UTM_METADATA_VERSION: &str = "0.3.0";
/// Latest supported hooks metadata schema version.
pub const LATEST_HOOKS_METADATA_VERSION: &str = "0.2.0";
/// Latest supported signer metadata schema version.
pub const LATEST_SIGNER_METADATA_VERSION: &str = "0.1.0";
/// Latest supported widget metadata schema version.
pub const LATEST_WIDGET_METADATA_VERSION: &str = "0.1.0";
/// Latest supported partner-fee metadata schema version.
pub const LATEST_PARTNER_FEE_METADATA_VERSION: &str = "1.0.0";
/// Latest supported replaced-order metadata schema version.
pub const LATEST_REPLACED_ORDER_METADATA_VERSION: &str = "0.1.0";
/// Latest supported wrappers metadata schema version.
pub const LATEST_WRAPPERS_METADATA_VERSION: &str = "0.2.0";
/// Latest supported user-consents metadata schema version.
pub const LATEST_USER_CONSENTS_METADATA_VERSION: &str = "0.1.0";
