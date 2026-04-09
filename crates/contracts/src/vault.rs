use serde::{Deserialize, Serialize};

use cow_sdk_core::Address;

use crate::{
    ContractsError,
    primitives::{encode_address, encode_fixed_bytes, function_selector, keccak256_hex},
};

pub const VAULT_INTERFACE: [&str; 2] = [
    "function manageUserBalance((uint8, address, uint256, address, address)[])",
    "function batchSwap(uint8, (bytes32, uint256, uint256, uint256, bytes)[], address[], (address, bool, address, bool), int256[], uint256)",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequiredVaultRole {
    pub method: String,
    pub selector: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantRoleCall {
    pub authorizer_address: Address,
    pub authorizer_abi_json: String,
    pub method: String,
    pub args_json: String,
}

pub fn required_vault_roles(
    vault_address: &Address,
) -> Result<Vec<RequiredVaultRole>, ContractsError> {
    VAULT_INTERFACE
        .iter()
        .map(|entry| {
            let method = entry
                .trim_start_matches("function ")
                .split('(')
                .next()
                .unwrap_or(entry)
                .to_owned();
            let selector = function_selector(entry.trim_start_matches("function "));
            let role = role_hash(vault_address, selector)?;
            Ok(RequiredVaultRole {
                method,
                selector: format!("0x{}", hex::encode(selector)),
                role,
            })
        })
        .collect()
}

pub fn required_vault_role_calls(
    authorizer_address: &Address,
    authorizer_abi_json: &str,
    vault_address: &Address,
    vault_relayer_address: &Address,
) -> Result<Vec<GrantRoleCall>, ContractsError> {
    required_vault_roles(vault_address)?
        .into_iter()
        .map(|role| {
            Ok(GrantRoleCall {
                authorizer_address: authorizer_address.clone(),
                authorizer_abi_json: authorizer_abi_json.to_owned(),
                method: "grantRole".to_owned(),
                args_json: serde_json::to_string(&(role.role, vault_relayer_address.clone()))
                    .map_err(|error| ContractsError::Serialization(error.to_string()))?,
            })
        })
        .collect()
}

pub fn grant_required_roles<F, E>(
    authorizer_address: &Address,
    authorizer_abi_json: &str,
    vault_address: &Address,
    vault_relayer_address: &Address,
    mut contract_call: F,
) -> Result<(), ContractsError>
where
    F: FnMut(&GrantRoleCall) -> Result<(), E>,
    E: std::fmt::Display,
{
    for call in required_vault_role_calls(
        authorizer_address,
        authorizer_abi_json,
        vault_address,
        vault_relayer_address,
    )? {
        contract_call(&call).map_err(|error| ContractsError::Provider(error.to_string()))?;
    }
    Ok(())
}

fn role_hash(vault_address: &Address, selector: [u8; 4]) -> Result<String, ContractsError> {
    let mut payload = Vec::with_capacity(64);
    payload.extend_from_slice(&encode_address(vault_address)?);
    payload.extend_from_slice(&encode_fixed_bytes(selector));
    Ok(keccak256_hex(payload))
}
