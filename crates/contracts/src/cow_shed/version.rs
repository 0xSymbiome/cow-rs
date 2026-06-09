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
    /// Every supported COW Shed version, current generation first.
    ///
    /// Ordered latest-first (`V1_0_1` before `V1_0_0`) so the default, current
    /// generation leads and legacy generations follow. Use it to enumerate
    /// every candidate proxy a user may own across versions — the building
    /// block for an "account proxies" discovery flow — without matching on the
    /// `#[non_exhaustive]` variants directly:
    ///
    /// ```
    /// use cow_sdk_contracts::cow_shed::CowShedVersion;
    ///
    /// assert_eq!(
    ///     CowShedVersion::ALL,
    ///     [CowShedVersion::V1_0_1, CowShedVersion::V1_0_0],
    /// );
    /// // The current generation leads and is the default.
    /// assert_eq!(CowShedVersion::ALL[0], CowShedVersion::default());
    /// ```
    pub const ALL: [Self; 2] = [Self::V1_0_1, Self::V1_0_0];

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
