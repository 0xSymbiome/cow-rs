//! Deployment verification status carried by registry rows.

use serde::{Deserialize, Serialize};

/// Verification status for a deployed registry row.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentVerificationStatus {
    /// Verified by comparing the deployed bytecode hash against a canonical source.
    CodeHashVerified,
    /// Verified through an external deployment authority.
    ExternalVerified,
    /// Present in upstream README deployment tables but not code-hash verified.
    ReadmeTableUnverified,
    /// Canonical upstream row without a dedicated verification artifact.
    CanonicalUnverified,
}

impl DeploymentVerificationStatus {
    /// Returns the manifest spelling for this status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CodeHashVerified => "code_hash_verified",
            Self::ExternalVerified => "external_verified",
            Self::ReadmeTableUnverified => "readme_table_unverified",
            Self::CanonicalUnverified => "canonical_unverified",
        }
    }
}

impl std::fmt::Display for DeploymentVerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
