//! Deployment registry environment identifiers.

use cow_sdk_core::CowEnv;
use serde::{Deserialize, Serialize};

/// Deployment environment carried by registry keys.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentEnv {
    /// Production deployment row.
    Prod,
    /// Staging deployment row.
    Staging,
    /// Deployment row shared by every environment.
    EnvironmentAgnostic,
}

impl DeploymentEnv {
    /// Returns the manifest spelling for this environment.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prod => "prod",
            Self::Staging => "staging",
            Self::EnvironmentAgnostic => "environment_agnostic",
        }
    }
}

impl From<CowEnv> for DeploymentEnv {
    fn from(value: CowEnv) -> Self {
        match value {
            CowEnv::Prod => Self::Prod,
            CowEnv::Staging => Self::Staging,
            #[allow(
                unreachable_patterns,
                reason = "CowEnv is non_exhaustive across crate boundaries"
            )]
            _ => unreachable!("unsupported future environment cannot be converted without review"),
        }
    }
}

impl std::fmt::Display for DeploymentEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
