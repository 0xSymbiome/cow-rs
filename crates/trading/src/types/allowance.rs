use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, Amount, CowEnv, SupportedChainId};

/// Parameters for allowance-check helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AllowanceParameters {
    /// ERC-20 token address.
    pub token_address: Address,
    /// Owner whose allowance should be inspected.
    pub owner: Address,
    /// Optional chain-id override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    /// Optional environment override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Optional explicit vault-relayer deployment override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_relayer_override: Option<Address>,
}

impl AllowanceParameters {
    /// Creates allowance parameters with the required token and owner fields.
    #[must_use]
    pub const fn new(token_address: Address, owner: Address) -> Self {
        Self {
            token_address,
            owner,
            chain_id: None,
            env: None,
            vault_relayer_override: None,
        }
    }

    /// Returns a copy with an explicit chain-id override.
    #[must_use]
    pub const fn with_chain_id(mut self, chain_id: SupportedChainId) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Returns a copy with an explicit environment override.
    #[must_use]
    pub const fn with_env(mut self, env: CowEnv) -> Self {
        self.env = Some(env);
        self
    }

    /// Returns a copy with an explicit vault-relayer deployment override.
    #[must_use]
    pub const fn with_vault_relayer_override(mut self, address: Address) -> Self {
        self.vault_relayer_override = Some(address);
        self
    }
}

/// Parameters for approval-transaction helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ApprovalParameters {
    /// ERC-20 token address.
    pub token_address: Address,
    /// Approval amount.
    pub amount: Amount,
    /// Optional chain-id override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    /// Optional environment override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Optional explicit vault-relayer deployment override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_relayer_override: Option<Address>,
}

impl ApprovalParameters {
    /// Creates approval parameters with the required token and amount fields.
    #[must_use]
    pub const fn new(token_address: Address, amount: Amount) -> Self {
        Self {
            token_address,
            amount,
            chain_id: None,
            env: None,
            vault_relayer_override: None,
        }
    }

    /// Returns a copy with an explicit chain-id override.
    #[must_use]
    pub const fn with_chain_id(mut self, chain_id: SupportedChainId) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Returns a copy with an explicit environment override.
    #[must_use]
    pub const fn with_env(mut self, env: CowEnv) -> Self {
        self.env = Some(env);
        self
    }

    /// Returns a copy with an explicit vault-relayer deployment override.
    #[must_use]
    pub const fn with_vault_relayer_override(mut self, address: Address) -> Self {
        self.vault_relayer_override = Some(address);
        self
    }
}
