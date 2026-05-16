use std::fmt;

/// Deployed COW Shed implementation versions supported by this crate.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CowShedVersion {
    /// COW Shed `1.0.0`.
    V1_0_0,
    /// COW Shed `1.0.1`.
    #[default]
    V1_0_1,
}

impl CowShedVersion {
    /// Returns the version string used in the COW Shed EIP-712 domain.
    #[must_use]
    pub const fn version_str(self) -> &'static str {
        match self {
            Self::V1_0_0 => "1.0.0",
            Self::V1_0_1 => "1.0.1",
        }
    }
}

impl fmt::Display for CowShedVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.version_str())
    }
}
