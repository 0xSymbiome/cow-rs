//! Typed hooks metadata consumed by the app-data metadata shape.
//!
//! The reviewed hook envelope is carried on the wire as
//! `metadata.hooks.{pre,post}`. Each hook call names a target contract, the
//! calldata to execute, a decimal-string gas limit, and an optional dApp id.
//! [`HookList`] narrows that schema to Rust types while preserving the wire
//! field names and decimal-string `gasLimit` representation.

use cow_sdk_core::{Address, HexData};
use serde::{Deserialize, Serialize};

/// Typed `metadata.hooks` value with pre- and post-interaction hook lists.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HookList {
    /// Optional hooks metadata schema version carried by some wire documents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Hooks executed before an order interaction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre: Vec<Hook>,
    /// Hooks executed after an order interaction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post: Vec<Hook>,
}

impl HookList {
    /// Creates a typed hooks envelope from pre- and post-hook lists.
    #[must_use]
    pub const fn new(pre: Vec<Hook>, post: Vec<Hook>) -> Self {
        Self {
            version: None,
            pre,
            post,
        }
    }

    /// Returns a copy with an explicit hooks metadata schema version.
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

/// One typed pre- or post-interaction hook call.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Hook {
    /// Contract address called by the hook.
    pub target: Address,
    /// Calldata supplied to the hook target.
    pub call_data: HexData,
    /// Gas limit for the hook, serialized as the schema's decimal string.
    #[serde(with = "alloy_serde::displayfromstr")]
    pub gas_limit: u64,
    /// Optional dApp identifier attached to the hook.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dapp_id: Option<String>,
}

impl Hook {
    /// Creates a typed hook call.
    #[must_use]
    pub const fn new(target: Address, call_data: HexData, gas_limit: u64) -> Self {
        Self {
            target,
            call_data,
            gas_limit,
            dapp_id: None,
        }
    }

    /// Returns a copy with an explicit dApp identifier.
    #[must_use]
    pub fn with_dapp_id(mut self, dapp_id: impl Into<String>) -> Self {
        self.dapp_id = Some(dapp_id.into());
        self
    }
}
