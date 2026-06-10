//! Const-backed registry of canonical `CoW` Protocol contract deployments.
//!
//! This module owns the typed key space — [`ContractId`], [`DeploymentChainId`],
//! and [`DeploymentEnv`] — and the [`Registry`] that resolves a deployed
//! contract address from the `(ContractId, DeploymentChainId, DeploymentEnv)`
//! key triple. The `GPv2Settlement`, `GPv2VaultRelayer`, and `CoWSwapEthFlow`
//! contracts are CREATE2 singletons that deploy to the same address on every
//! supported chain; each contract family carries one production and one
//! staging deployment, so the registry is a small const table rather than a
//! per-chain manifest. Each address is pinned to its upstream source
//! repository in `parity/source-lock.yaml` and confirmed on-chain by a
//! read-only `eth_getCode` presence probe.
//!
//! [`DeploymentChainId`] is kept distinct from [`cow_sdk_core::SupportedChainId`]
//! so deployment evidence for chains such as Lens can be represented without
//! broadening runtime API support: Lens is deployment-only for the composable /
//! COW-Shed contract families and carries none of the `GPv2` contracts, so it
//! resolves to [`None`].

use cow_sdk_core::{Address, CowEnv, SupportedChainId};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

// ----- Chain identifiers -----

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
    /// Returns the numeric EVM chain id.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self as u64
    }
}

impl From<SupportedChainId> for DeploymentChainId {
    /// Bridges a [`SupportedChainId`] runtime tag onto the deployment-evidence
    /// chain enum.
    ///
    /// # Panics
    ///
    /// Panics only if a future [`SupportedChainId`] variant is added upstream
    /// without a corresponding deployment-evidence chain landing in this
    /// match. The non-exhaustive wildcard arm exists solely to satisfy the
    /// compiler across crate boundaries; any new chain must land in the same
    /// patch as this match arm and is gated by reviewer policy.
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
            // SAFETY: SupportedChainId is the sole producer for this bridge.
            // Every currently supported chain has an explicit match arm above.
            // Reaching the wildcard would require a new SupportedChainId variant
            // landing without a matching deployment-evidence chain in the same
            // patch, which the reviewer policy prevents.
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

// ----- Contract identifiers -----

/// Canonical contract identifiers resolved by [`Registry`].
///
/// Every variant is a CREATE2 singleton that deploys to the same address on
/// every supported chain; the eth-flow periphery additionally carries a
/// distinct production and staging deployment. Variants follow the
/// **Pascal-case** convention.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ContractId {
    /// `GPv2Settlement` — the `CoW` Protocol settlement entry point.
    Settlement,
    /// `GPv2VaultRelayer` — relays balance operations into the Balancer vault.
    VaultRelayer,
    /// `CoWSwapEthFlow` — wraps the native asset into orders on behalf of traders.
    EthFlow,
}

impl ContractId {
    /// Returns the canonical Pascal-case name for this identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Settlement => "Settlement",
            Self::VaultRelayer => "VaultRelayer",
            Self::EthFlow => "EthFlow",
        }
    }
}

impl std::fmt::Display for ContractId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ----- Environment identifiers -----

/// Deployment environment carried by registry keys.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentEnv {
    /// Production deployment row.
    Prod,
    /// Staging deployment row.
    Staging,
}

impl DeploymentEnv {
    /// Returns the manifest spelling for this environment.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prod => "prod",
            Self::Staging => "staging",
        }
    }
}

impl From<CowEnv> for DeploymentEnv {
    /// Bridges a [`CowEnv`] runtime tag onto the deployment-evidence
    /// environment enum.
    ///
    /// # Panics
    ///
    /// Panics only if a future [`CowEnv`] variant is added upstream without a
    /// corresponding deployment-evidence environment landing in this match.
    /// The non-exhaustive wildcard arm exists solely to satisfy the compiler
    /// across crate boundaries; any new environment must land in the same
    /// patch as this match arm and is gated by reviewer policy.
    fn from(value: CowEnv) -> Self {
        match value {
            CowEnv::Prod => Self::Prod,
            CowEnv::Staging => Self::Staging,
            #[allow(
                unreachable_patterns,
                reason = "CowEnv is non_exhaustive across crate boundaries"
            )]
            // SAFETY: CowEnv is the sole producer for this bridge. Every
            // currently supported environment has an explicit match arm above.
            // Reaching the wildcard would require a new CowEnv variant landing
            // without a matching deployment-evidence environment in the same
            // patch, which the reviewer policy prevents.
            _ => unreachable!("unsupported future environment cannot be converted without review"),
        }
    }
}

impl std::fmt::Display for DeploymentEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ----- Address registry -----

/// `GPv2Settlement` production deployment — identical on every supported chain.
const GPV2_SETTLEMENT: &str = "0x9008D19f58AAbD9eD0D60971565AA8510560ab41";
/// `GPv2Settlement` staging deployment — identical on every supported chain.
const GPV2_SETTLEMENT_STAGING: &str = "0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13";
/// `GPv2VaultRelayer` production deployment — identical on every supported chain.
const GPV2_VAULT_RELAYER: &str = "0xC92E8bdf79f0507f65a392b0ab4667716BFE0110";
/// `GPv2VaultRelayer` staging deployment — identical on every supported chain.
const GPV2_VAULT_RELAYER_STAGING: &str = "0xC7242d167563352E2BCA4d71C043fbe542DB8FB2";
/// `CoWSwapEthFlow` production deployment — identical on every supported chain.
const ETH_FLOW_PROD: &str = "0xba3cb449bd2b4adddbc894d8697f5170800eadec";
/// `CoWSwapEthFlow` staging deployment — identical on every supported chain.
const ETH_FLOW_STAGING: &str = "0xb37aDD6AC288BD3825a901Cba6ec65A89f31B8CC";

/// Resolver for canonical `CoW` Protocol contract deployment addresses.
///
/// [`Registry::default`] is the only constructor; every shipped leaf crate
/// resolves through [`Registry::address`] rather than reading chain-scoped
/// address constants directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Registry {
    settlement: Address,
    settlement_staging: Address,
    vault_relayer: Address,
    vault_relayer_staging: Address,
    eth_flow_prod: Address,
    eth_flow_staging: Address,
}

impl Default for Registry {
    /// Builds the canonical registry from the committed address constants.
    ///
    /// # Panics
    ///
    /// Panics only if a committed address literal stops being a valid 20-byte
    /// address — impossible without a source edit, and pinned by the
    /// `deployment_addresses_resolve_to_canonical_singletons` regression.
    fn default() -> Self {
        // SAFETY: every literal above is a canonical 20-byte deployment address.
        let parse =
            |hex: &str| Address::new(hex).expect("canonical deployment address literal is valid");
        Self {
            settlement: parse(GPV2_SETTLEMENT),
            settlement_staging: parse(GPV2_SETTLEMENT_STAGING),
            vault_relayer: parse(GPV2_VAULT_RELAYER),
            vault_relayer_staging: parse(GPV2_VAULT_RELAYER_STAGING),
            eth_flow_prod: parse(ETH_FLOW_PROD),
            eth_flow_staging: parse(ETH_FLOW_STAGING),
        }
    }
}

impl Registry {
    /// Returns the deployed address registered for the supplied identifier
    /// triple, or [`None`] when the contract is not deployed on that chain.
    ///
    /// The `GPv2` settlement, vault-relayer, and eth-flow contracts deploy on
    /// every runtime-supported chain; each contract family resolves a distinct
    /// production or staging deployment from `env`.
    #[must_use]
    pub fn address(
        &self,
        contract_id: ContractId,
        chain_id: impl Into<DeploymentChainId>,
        env: impl Into<DeploymentEnv>,
    ) -> Option<Address> {
        // Lens is deployment-only for the composable / COW-Shed families and
        // carries none of the GPv2 contracts resolved here.
        if matches!(chain_id.into(), DeploymentChainId::Lens) {
            return None;
        }
        Some(match (contract_id, env.into()) {
            (ContractId::Settlement, DeploymentEnv::Prod) => self.settlement,
            (ContractId::Settlement, DeploymentEnv::Staging) => self.settlement_staging,
            (ContractId::VaultRelayer, DeploymentEnv::Prod) => self.vault_relayer,
            (ContractId::VaultRelayer, DeploymentEnv::Staging) => self.vault_relayer_staging,
            (ContractId::EthFlow, DeploymentEnv::Prod) => self.eth_flow_prod,
            (ContractId::EthFlow, DeploymentEnv::Staging) => self.eth_flow_staging,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ContractId, DeploymentChainId, DeploymentEnv, Registry};

    #[test]
    fn deployment_addresses_resolve_to_canonical_singletons() {
        let registry = Registry::default();

        // Each environment resolves a chain-invariant CREATE2 singleton.
        let settlement = registry
            .address(
                ContractId::Settlement,
                DeploymentChainId::Mainnet,
                DeploymentEnv::Prod,
            )
            .expect("settlement is deployed on mainnet");
        assert_eq!(
            settlement.to_hex_string(),
            "0x9008d19f58aabd9ed0d60971565aa8510560ab41"
        );
        assert_eq!(
            registry.address(
                ContractId::Settlement,
                DeploymentChainId::Base,
                DeploymentEnv::Prod
            ),
            Some(settlement),
            "production settlement is identical across chains"
        );

        // Settlement and vault-relayer resolve distinct staging deployments,
        // each likewise identical across chains.
        let settlement_staging = registry
            .address(
                ContractId::Settlement,
                DeploymentChainId::Mainnet,
                DeploymentEnv::Staging,
            )
            .expect("staging settlement is deployed on mainnet");
        assert_eq!(
            settlement_staging.to_hex_string(),
            "0xf553d092b50bdcbdded1a99af2ca29fbe5e2cb13"
        );
        assert_eq!(
            registry.address(
                ContractId::Settlement,
                DeploymentChainId::Base,
                DeploymentEnv::Staging
            ),
            Some(settlement_staging),
            "staging settlement is identical across chains"
        );
        assert_ne!(
            settlement, settlement_staging,
            "settlement prod and staging are distinct deployments"
        );
        let vault_relayer_staging = registry
            .address(
                ContractId::VaultRelayer,
                DeploymentChainId::Mainnet,
                DeploymentEnv::Staging,
            )
            .expect("staging vault-relayer is deployed on mainnet");
        assert_eq!(
            vault_relayer_staging.to_hex_string(),
            "0xc7242d167563352e2bca4d71c043fbe542db8fb2"
        );
        assert_ne!(
            registry.address(
                ContractId::VaultRelayer,
                DeploymentChainId::Mainnet,
                DeploymentEnv::Prod,
            ),
            Some(vault_relayer_staging),
            "vault-relayer prod and staging are distinct deployments"
        );

        // Eth-flow resolves a distinct production and staging deployment.
        let prod = registry
            .address(
                ContractId::EthFlow,
                DeploymentChainId::GnosisChain,
                DeploymentEnv::Prod,
            )
            .expect("eth-flow production deployment exists");
        let staging = registry
            .address(
                ContractId::EthFlow,
                DeploymentChainId::GnosisChain,
                DeploymentEnv::Staging,
            )
            .expect("eth-flow staging deployment exists");
        assert_ne!(
            prod, staging,
            "eth-flow prod and staging are distinct deployments"
        );

        // The GPv2 contracts are not deployed on the deployment-only Lens chain.
        assert_eq!(
            registry.address(
                ContractId::Settlement,
                DeploymentChainId::Lens,
                DeploymentEnv::Prod
            ),
            None,
        );
    }
}
