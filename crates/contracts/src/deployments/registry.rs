//! Const-backed registry of canonical `CoW` Protocol contract deployments.
//!
//! [`Registry::address`] resolves a deployed contract address from the
//! `(ContractId, DeploymentChainId, DeploymentEnv)` key triple. The
//! `GPv2Settlement`, `GPv2VaultRelayer`, and `CoWSwapEthFlow` contracts are
//! CREATE2 singletons that deploy to the same address on every supported
//! chain (eth-flow carries one production and one staging deployment), so the
//! registry is a small const table rather than a per-chain manifest. The
//! upstream provenance for each address is pinned per source repository in
//! `parity/source-lock.yaml`; a read-only `eth_getCode` presence probe
//! confirms each address on-chain.
//!
//! Lens is deployment-only for the composable / COW-Shed contract families
//! and carries none of the GPv2 contracts, so it resolves to [`None`].

use cow_sdk_core::Address;

use super::{ContractId, DeploymentChainId, DeploymentEnv};

/// `GPv2Settlement` singleton — identical on every supported chain.
const GPV2_SETTLEMENT: &str = "0x9008D19f58AAbD9eD0D60971565AA8510560ab41";
/// `GPv2VaultRelayer` singleton — identical on every supported chain.
const GPV2_VAULT_RELAYER: &str = "0xC92E8bdf79f0507f65a392b0ab4667716BFE0110";
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
    vault_relayer: Address,
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
            vault_relayer: parse(GPV2_VAULT_RELAYER),
            eth_flow_prod: parse(ETH_FLOW_PROD),
            eth_flow_staging: parse(ETH_FLOW_STAGING),
        }
    }
}

impl Registry {
    /// Returns the deployed address registered for the supplied identifier
    /// triple, or [`None`] when the contract is not deployed on that chain.
    ///
    /// The GPv2 settlement, vault-relayer, and eth-flow contracts deploy on
    /// every runtime-supported chain; settlement and vault-relayer share one
    /// address across both environments, while eth-flow resolves the
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
            (ContractId::Settlement, _) => self.settlement,
            (ContractId::VaultRelayer, _) => self.vault_relayer,
            (ContractId::EthFlow, DeploymentEnv::Staging) => self.eth_flow_staging,
            (ContractId::EthFlow, _) => self.eth_flow_prod,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ContractId, DeploymentChainId, DeploymentEnv, Registry};

    #[test]
    fn deployment_addresses_resolve_to_canonical_singletons() {
        let registry = Registry::default();

        // Settlement and vault-relayer are chain- and env-invariant singletons.
        let settlement = registry
            .address(ContractId::Settlement, DeploymentChainId::Mainnet, DeploymentEnv::Prod)
            .expect("settlement is deployed on mainnet");
        assert_eq!(
            settlement.to_hex_string(),
            "0x9008d19f58aabd9ed0d60971565aa8510560ab41"
        );
        assert_eq!(
            registry.address(ContractId::Settlement, DeploymentChainId::Base, DeploymentEnv::Staging),
            Some(settlement),
            "settlement is identical across chains and environments"
        );

        // Eth-flow resolves a distinct production and staging deployment.
        let prod = registry
            .address(ContractId::EthFlow, DeploymentChainId::GnosisChain, DeploymentEnv::Prod)
            .expect("eth-flow production deployment exists");
        let staging = registry
            .address(ContractId::EthFlow, DeploymentChainId::GnosisChain, DeploymentEnv::Staging)
            .expect("eth-flow staging deployment exists");
        assert_ne!(prod, staging, "eth-flow prod and staging are distinct deployments");

        // The GPv2 contracts are not deployed on the deployment-only Lens chain.
        assert_eq!(
            registry.address(ContractId::Settlement, DeploymentChainId::Lens, DeploymentEnv::Prod),
            None,
        );
    }
}
