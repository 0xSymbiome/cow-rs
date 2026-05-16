//! Composable chain-coverage contract test: assert that every
//! `SupportedChainId` chain plus `DeploymentChainId::Lens` has
//! `Some(addr)` for `ContractId::ComposableCow`. This is the
//! per-chain coverage invariant from the composable capability landing risk register
//! (R4): every `SupportedChainId` chain must have a composable
//! deployment row.

use cow_sdk_contracts::{ContractId, DeploymentChainId, DeploymentEnv, Registry};
use cow_sdk_core::SupportedChainId;

const SUPPORTED_CHAINS: &[SupportedChainId] = &[
    SupportedChainId::Mainnet,
    SupportedChainId::Bnb,
    SupportedChainId::GnosisChain,
    SupportedChainId::Polygon,
    SupportedChainId::Base,
    SupportedChainId::Plasma,
    SupportedChainId::ArbitrumOne,
    SupportedChainId::Avalanche,
    SupportedChainId::Linea,
    SupportedChainId::Sepolia,
];

#[test]
fn every_supported_chain_has_composable_cow_address() {
    let registry = Registry::default();
    for chain in SUPPORTED_CHAINS {
        let address = registry.address(
            ContractId::ComposableCow,
            DeploymentChainId::from(*chain),
            DeploymentEnv::EnvironmentAgnostic,
        );
        assert!(
            address.is_some(),
            "ContractId::ComposableCow must have an address on chain {chain:?}; got None"
        );
    }
}

#[test]
fn lens_has_composable_cow_address() {
    let registry = Registry::default();
    let address = registry.address(
        ContractId::ComposableCow,
        DeploymentChainId::Lens,
        DeploymentEnv::EnvironmentAgnostic,
    );
    assert!(
        address.is_some(),
        "ContractId::ComposableCow must have an address on Lens (DeploymentChainId::Lens = 232); got None"
    );
}

#[test]
fn ink_does_not_have_composable_cow_address() {
    let registry = Registry::default();
    let address = registry.address(
        ContractId::ComposableCow,
        DeploymentChainId::Ink,
        DeploymentEnv::EnvironmentAgnostic,
    );
    assert!(
        address.is_none(),
        "Ink (chain_id 57073) must be a not_deployed coverage record, not an addressable registry row; got Some({address:?})"
    );
}

#[test]
fn cow_shed_for_composable_cow_is_only_on_gnosis_chain() {
    let registry = Registry::default();
    let gnosis = registry.address(
        ContractId::CowShedForComposableCow,
        DeploymentChainId::GnosisChain,
        DeploymentEnv::EnvironmentAgnostic,
    );
    assert!(
        gnosis.is_some(),
        "CowShedForComposableCow must have an address on Gnosis Chain (chain_id 100); got None"
    );
    for chain in SUPPORTED_CHAINS {
        if *chain == SupportedChainId::GnosisChain {
            continue;
        }
        let address = registry.address(
            ContractId::CowShedForComposableCow,
            DeploymentChainId::from(*chain),
            DeploymentEnv::EnvironmentAgnostic,
        );
        assert!(
            address.is_none(),
            "CowShedForComposableCow must NOT have an address on {chain:?}; got Some"
        );
    }
}
