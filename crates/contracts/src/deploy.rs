use serde::{Deserialize, Serialize};

use cow_sdk_core::{
    Address, CowEnv, SupportedChainId, eth_flow_contract_address, settlement_contract_address,
    vault_relayer_address,
};

use crate::{
    ContractsError,
    primitives::{encode_address, keccak256},
};

/// Deterministic deployment salt used by CoW deployments.
pub const SALT: &str = "0x4d61747472657373657320696e204265726c696e210000000000000000000000";
/// Deployer contract address used for deterministic deployment derivation.
pub const DEPLOYER_CONTRACT: &str = "0x4e59b44847b379578588920ca78fbf26c0b4956c";

/// Supported named CoW deployment artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContractName {
    /// Authenticator contract.
    Authenticator,
    /// Settlement contract.
    Settlement,
    /// Trade-simulation helper contract.
    TradeSimulator,
}

/// Core CoW deployment addresses for a supported chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractAddresses {
    /// Settlement contract address.
    pub settlement: Address,
    /// Vault relayer address.
    pub vault_relayer: Address,
    /// EthFlow contract address.
    pub eth_flow: Address,
}

/// Computes a deterministic deployment address from bytecode and constructor arguments.
///
/// # Errors
///
/// Returns [`ContractsError`] when bytecode or constructor arguments are not
/// valid hex, or when address validation fails during `CREATE2` derivation.
pub fn deterministic_deployment_address(
    bytecode: &str,
    deployment_arguments: &[String],
) -> Result<Address, ContractsError> {
    let mut init_code = crate::primitives::parse_hex(bytecode, "bytecode")?;
    for arg in deployment_arguments {
        init_code.extend_from_slice(&crate::primitives::parse_hex(arg, "deploymentArgument")?);
    }

    let deployer = Address::new(DEPLOYER_CONTRACT)?;
    let salt = crate::primitives::parse_hex_exact(SALT, "salt", 32)?;
    let mut create2_payload = Vec::with_capacity(85);
    create2_payload.push(0xff);
    create2_payload.extend_from_slice(&crate::primitives::parse_hex_exact(
        deployer.as_str(),
        "deployer",
        20,
    )?);
    create2_payload.extend_from_slice(&salt);
    create2_payload.extend_from_slice(&keccak256(init_code));
    let hash = keccak256(create2_payload);
    Address::new(format!("0x{}", hex::encode(&hash[12..]))).map_err(Into::into)
}

/// Returns the canonical production deployment addresses for a supported chain.
///
/// # Errors
///
/// Returns [`ContractsError::UnsupportedChain`] when `chain_id` is not part of
/// the supported CoW deployment set.
pub fn deployment_for_chain(chain_id: u64) -> Result<ContractAddresses, ContractsError> {
    let chain = SupportedChainId::try_from(chain_id)
        .map_err(|_| ContractsError::UnsupportedChain(chain_id))?;
    Ok(ContractAddresses {
        settlement: settlement_contract_address(chain, CowEnv::Prod),
        vault_relayer: vault_relayer_address(chain, CowEnv::Prod),
        eth_flow: eth_flow_contract_address(chain, CowEnv::Prod),
    })
}

/// Returns the keccak256 hash of the deployment init code.
///
/// # Errors
///
/// Returns [`ContractsError`] when bytecode or constructor arguments are not
/// valid hex, or when deployer address validation fails.
pub fn deployment_address_hash_input(
    bytecode: &str,
    deployment_arguments: &[String],
) -> Result<[u8; 32], ContractsError> {
    let _ = encode_address(&Address::new(DEPLOYER_CONTRACT)?)?;
    let mut init_code = crate::primitives::parse_hex(bytecode, "bytecode")?;
    for arg in deployment_arguments {
        init_code.extend_from_slice(&crate::primitives::parse_hex(arg, "deploymentArgument")?);
    }
    Ok(keccak256(init_code))
}
