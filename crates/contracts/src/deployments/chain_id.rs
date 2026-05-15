//! Deployment registry chain identifiers.
//!
//! The deployment registry tracks contract availability beyond the core
//! orderbook-supported chain list. Keep this enum separate from
//! [`cow_sdk_core::SupportedChainId`] so deployment evidence for chains such
//! as Lens can be represented without broadening runtime API support.

use cow_sdk_core::SupportedChainId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// Chain ids accepted by the deployment registry.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u64)]
pub enum DeploymentChainId {
    /// Ethereum mainnet.
    Mainnet = 1,
    /// BNB Smart Chain.
    Bnb = 56,
    /// Gnosis Chain.
    GnosisChain = 100,
    /// Polygon `PoS`.
    Polygon = 137,
    /// Base.
    Base = 8453,
    /// Plasma.
    Plasma = 9745,
    /// Arbitrum One.
    ArbitrumOne = 42_161,
    /// Avalanche C-Chain.
    Avalanche = 43_114,
    /// Ink.
    Ink = 57_073,
    /// Linea.
    Linea = 59_144,
    /// Ethereum Sepolia.
    Sepolia = 11_155_111,
    /// Lens.
    Lens = 232,
}

impl DeploymentChainId {
    /// Complete list of addressable deployment chain ids.
    pub const ALL: [Self; 12] = [
        Self::Mainnet,
        Self::Bnb,
        Self::GnosisChain,
        Self::Polygon,
        Self::Base,
        Self::Plasma,
        Self::ArbitrumOne,
        Self::Avalanche,
        Self::Ink,
        Self::Linea,
        Self::Sepolia,
        Self::Lens,
    ];

    /// Returns the numeric EVM chain id.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self as u64
    }
}

impl From<SupportedChainId> for DeploymentChainId {
    fn from(value: SupportedChainId) -> Self {
        match value {
            SupportedChainId::Mainnet => Self::Mainnet,
            SupportedChainId::Bnb => Self::Bnb,
            SupportedChainId::GnosisChain => Self::GnosisChain,
            SupportedChainId::Polygon => Self::Polygon,
            SupportedChainId::Base => Self::Base,
            SupportedChainId::Plasma => Self::Plasma,
            SupportedChainId::ArbitrumOne => Self::ArbitrumOne,
            SupportedChainId::Avalanche => Self::Avalanche,
            SupportedChainId::Ink => Self::Ink,
            SupportedChainId::Linea => Self::Linea,
            SupportedChainId::Sepolia => Self::Sepolia,
            #[allow(
                unreachable_patterns,
                reason = "SupportedChainId is non_exhaustive across crate boundaries"
            )]
            _ => unreachable!("unsupported future chain id cannot be converted without review"),
        }
    }
}

impl TryFrom<u64> for DeploymentChainId {
    type Error = DeploymentChainIdError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Mainnet),
            56 => Ok(Self::Bnb),
            100 => Ok(Self::GnosisChain),
            137 => Ok(Self::Polygon),
            8453 => Ok(Self::Base),
            9745 => Ok(Self::Plasma),
            42_161 => Ok(Self::ArbitrumOne),
            43_114 => Ok(Self::Avalanche),
            57_073 => Ok(Self::Ink),
            59_144 => Ok(Self::Linea),
            11_155_111 => Ok(Self::Sepolia),
            232 => Ok(Self::Lens),
            chain_id => Err(DeploymentChainIdError { chain_id }),
        }
    }
}

impl From<DeploymentChainId> for u64 {
    fn from(value: DeploymentChainId) -> Self {
        value.as_u64()
    }
}

impl Serialize for DeploymentChainId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64((*self).into())
    }
}

impl<'de> Deserialize<'de> for DeploymentChainId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

/// Error returned when a numeric chain id is outside the deployment taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("unsupported deployment chain id {chain_id}")]
pub struct DeploymentChainIdError {
    /// Unsupported numeric chain id.
    pub chain_id: u64,
}
