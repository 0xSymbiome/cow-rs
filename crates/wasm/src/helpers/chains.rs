//! Chain and deployment helpers.

use cow_sdk_contracts::{ContractId, Registry};
use cow_sdk_core::{CowEnv, SupportedChainId};

use crate::helpers::{dto::DeploymentAddresses, errors::PureError};

/// Parses a numeric chain id into the SDK-supported chain set.
///
/// # Errors
///
/// Returns [`PureError::UnsupportedChain`] when the chain is not configured.
pub fn supported_chain(chain_id: u32) -> Result<SupportedChainId, PureError> {
    SupportedChainId::try_from(u64::from(chain_id))
        .map_err(|_| PureError::UnsupportedChain { chain_id })
}

/// Parses an optional environment string. Omitted environments default to production.
///
/// # Errors
///
/// Returns [`PureError`] when the environment is not `prod` or `staging`.
pub fn env_from_str(env: Option<&str>) -> Result<CowEnv, PureError> {
    match env.unwrap_or("prod") {
        "prod" | "production" => Ok(CowEnv::Prod),
        "staging" | "barn" => Ok(CowEnv::Staging),
        value => Err(PureError::unknown_enum("env", value)),
    }
}

/// Computes the EIP-712 domain separator for a supported chain.
///
/// # Errors
///
/// Returns [`PureError`] for unsupported chains or signing-domain failures.
pub fn domain_separator(chain_id: u32) -> Result<String, PureError> {
    let chain = supported_chain(chain_id)?;
    cow_sdk_signing::domain_separator(chain, None)
        .map_err(|error| PureError::invalid("chainId", error.to_string()))
}

/// Returns supported chain ids in deterministic order.
#[must_use]
pub fn supported_chain_ids() -> Vec<u32> {
    SupportedChainId::ALL
        .into_iter()
        .filter_map(|chain| u32::try_from(u64::from(chain)).ok())
        .collect()
}

/// Looks up canonical deployment addresses for a chain and environment.
///
/// # Errors
///
/// Returns [`PureError`] for unsupported chains, environments, or missing registry rows.
pub fn deployment_addresses(
    chain_id: u32,
    env: Option<&str>,
) -> Result<DeploymentAddresses, PureError> {
    let chain = supported_chain(chain_id)?;
    let env = env_from_str(env)?;
    let registry = Registry::default();

    let address = |contract_id| {
        registry
            .address(contract_id, chain, env)
            .ok_or_else(|| PureError::invalid("chainId", "deployment is not configured"))
            .map(|addr| addr.to_hex_string())
    };

    Ok(DeploymentAddresses {
        settlement: address(ContractId::Settlement)?,
        vault_relayer: address(ContractId::VaultRelayer)?,
        eth_flow: address(ContractId::EthFlow)?,
    })
}

/// Returns the crate version embedded at compile time.
#[must_use]
pub fn wasm_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}
