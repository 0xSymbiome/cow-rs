use alloy_sol_types::{SolCall, sol};
use serde::{Deserialize, Serialize};

use cow_sdk_core::Address;

use crate::ContractsError;

sol! {
    // Canonical GPv2VaultRelayer ABI surface plus the partial Balancer V2 Vault
    // interface whose selectors drive the role-grant flow. Signatures are
    // reproduced verbatim from the upstream cowprotocol/contracts repository
    // (`src/contracts/GPv2VaultRelayer.sol` and the Balancer V2 Vault ABI used
    // by the GPv2 toolchain). The Solidity excerpt used to author these
    // bindings is committed under `crates/contracts/abi/vault-relayer/` for
    // provenance.
    #[sol(rename_all = "camelcase")]
    interface IGPv2VaultRelayer {
        struct Transfer {
            address account;
            address token;
            uint256 amount;
            uint8 balance;
        }

        struct BatchSwapStep {
            bytes32 poolId;
            uint256 assetInIndex;
            uint256 assetOutIndex;
            uint256 amount;
            bytes userData;
        }

        struct FundManagement {
            address sender;
            bool fromInternalBalance;
            address recipient;
            bool toInternalBalance;
        }

        function transferFromAccounts(Transfer[] calldata transfers) external;

        function batchSwapWithFee(
            uint8 kind,
            BatchSwapStep[] calldata swaps,
            address[] memory tokens,
            FundManagement memory funds,
            int256[] memory limits,
            uint256 deadline,
            Transfer calldata feeTransfer
        ) external returns (int256[] memory tokenDeltas);
    }

    #[sol(rename_all = "camelcase")]
    interface IVault {
        struct UserBalanceOp {
            uint8 kind;
            address asset;
            uint256 amount;
            address sender;
            address recipient;
        }

        struct BatchSwapStep {
            bytes32 poolId;
            uint256 assetInIndex;
            uint256 assetOutIndex;
            uint256 amount;
            bytes userData;
        }

        struct FundManagement {
            address sender;
            bool fromInternalBalance;
            address recipient;
            bool toInternalBalance;
        }

        function manageUserBalance(UserBalanceOp[] calldata ops) external payable;

        function batchSwap(
            uint8 kind,
            BatchSwapStep[] calldata swaps,
            address[] memory assets,
            FundManagement memory funds,
            int256[] memory limits,
            uint256 deadline
        ) external payable returns (int256[] memory assetDeltas);
    }
}

/// Vault methods that require explicit relayer authorization.
///
/// The list mirrors the Balancer V2 Vault entrypoints the `GPv2` Vault Relayer
/// invokes on behalf of the settlement contract. Signature strings are
/// preserved for legacy reader helpers that parse the method name off the
/// head of each entry; selectors used to derive role hashes are sourced from
/// the `alloy::sol!`-generated `IVault` interface above.
pub const VAULT_INTERFACE: [&str; 2] = [
    "function manageUserBalance((uint8, address, uint256, address, address)[])",
    "function batchSwap(uint8, (bytes32, uint256, uint256, uint256, bytes)[], address[], (address, bool, address, bool), int256[], uint256)",
];

const VAULT_ROLE_SOURCES: [(&str, [u8; 4]); 2] = [
    ("manageUserBalance", IVault::manageUserBalanceCall::SELECTOR),
    ("batchSwap", IVault::batchSwapCall::SELECTOR),
];

/// Derived vault role metadata for a specific method selector.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequiredVaultRole {
    /// Human-readable method name.
    pub method: String,
    /// Method selector as a hex string.
    pub selector: String,
    /// Derived role hash.
    pub role: String,
}

/// Prepared `grantRole` call for a vault relayer authorization flow.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantRoleCall {
    /// Authorizer contract address.
    pub authorizer_address: Address,
    /// JSON ABI for the authorizer contract.
    pub authorizer_abi_json: String,
    /// Method name to invoke.
    pub method: String,
    /// JSON-encoded arguments for the method call.
    pub args_json: String,
}

impl RequiredVaultRole {
    /// Creates derived vault role metadata.
    #[must_use]
    pub const fn new(method: String, selector: String, role: String) -> Self {
        Self {
            method,
            selector,
            role,
        }
    }
}

impl GrantRoleCall {
    /// Creates a prepared `grantRole` call.
    #[must_use]
    pub const fn new(
        authorizer_address: Address,
        authorizer_abi_json: String,
        method: String,
        args_json: String,
    ) -> Self {
        Self {
            authorizer_address,
            authorizer_abi_json,
            method,
            args_json,
        }
    }
}

/// Returns the required vault role hashes for the supported vault methods.
///
/// # Errors
///
/// Returns [`ContractsError`] if address encoding fails while deriving the role hash.
pub fn required_vault_roles(
    vault_address: &Address,
) -> Result<Vec<RequiredVaultRole>, ContractsError> {
    VAULT_ROLE_SOURCES
        .iter()
        .map(|(method, selector)| {
            let role = role_hash(vault_address, *selector);
            Ok(RequiredVaultRole::new(
                (*method).to_owned(),
                format!("0x{}", hex::encode(selector)),
                role,
            ))
        })
        .collect()
}

/// Builds `grantRole` calls for every required vault role.
///
/// # Errors
///
/// Returns [`ContractsError`] if role derivation or JSON argument serialization fails.
pub fn required_vault_role_calls(
    authorizer_address: &Address,
    authorizer_abi_json: &str,
    vault_address: &Address,
    vault_relayer: &Address,
) -> Result<Vec<GrantRoleCall>, ContractsError> {
    required_vault_roles(vault_address)?
        .into_iter()
        .map(|role| {
            Ok(GrantRoleCall::new(
                *authorizer_address,
                authorizer_abi_json.to_owned(),
                "grantRole".to_owned(),
                serde_json::to_string(&(role.role, *vault_relayer))?,
            ))
        })
        .collect()
}

/// Executes all required vault role grants through the supplied callback.
///
/// # Errors
///
/// Returns [`ContractsError`] if role-call construction fails or if `contract_call`
/// returns an error for any required role.
pub fn grant_required_roles<F, E>(
    authorizer_address: &Address,
    authorizer_abi_json: &str,
    vault_address: &Address,
    vault_relayer: &Address,
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
        vault_relayer,
    )? {
        contract_call(&call).map_err(|error| ContractsError::Provider {
            operation: "grantRole",
            message: error.to_string().into(),
        })?;
    }
    Ok(())
}

fn role_hash(vault_address: &Address, selector: [u8; 4]) -> String {
    let address_bytes = vault_address.into_alloy().0.0;
    let mut packed = [0u8; 36];
    packed[12..32].copy_from_slice(&address_bytes);
    packed[32..36].copy_from_slice(&selector);
    format!(
        "0x{}",
        hex::encode(alloy_primitives::keccak256(packed).as_slice())
    )
}
