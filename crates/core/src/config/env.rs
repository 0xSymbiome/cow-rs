use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    redaction::RedactedUrlMap,
    types::{Address, ChainId},
};

use super::chains::SupportedChainId;

const PROD_BASE_URL: &str = "https://api.cow.fi";
const STAGING_BASE_URL: &str = "https://barn.api.cow.fi";
const PARTNER_PROD_BASE_URL: &str = "https://partners.cow.fi";
const PARTNER_STAGING_BASE_URL: &str = "https://partners.barn.cow.fi";

/// Supported `CoW` deployment environments.
///
/// Downstream crates should include a wildcard arm when matching so future
/// deployment environments remain semver-compatible.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CowEnv {
    /// Production endpoints and deployments.
    Prod,
    /// Staging endpoints and deployments.
    Staging,
}

impl CowEnv {
    /// Returns the stable lowercase environment identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prod => "prod",
            Self::Staging => "staging",
        }
    }
}

impl fmt::Display for CowEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Redacting mapping from numeric chain id to API base URL.
pub type ApiBaseUrls = RedactedUrlMap<ChainId>;
/// Mapping from numeric chain id to deployment address override.
pub type AddressPerChain = BTreeMap<ChainId, Address>;

/// Returns the default `CoW` API base URLs for every supported chain.
#[must_use]
pub fn default_api_base_urls(env: CowEnv, partner_api: bool) -> ApiBaseUrls {
    SupportedChainId::ALL
        .into_iter()
        .map(|chain_id| {
            let base = match (env, partner_api) {
                (CowEnv::Prod, false) => PROD_BASE_URL,
                (CowEnv::Staging, false) => STAGING_BASE_URL,
                (CowEnv::Prod, true) => PARTNER_PROD_BASE_URL,
                (CowEnv::Staging, true) => PARTNER_STAGING_BASE_URL,
            };
            (chain_id.into(), format!("{base}/{}", chain_id.api_path()))
        })
        .collect()
}
