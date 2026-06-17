use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::types::{Address, ChainId};
/// Typed-data domain metadata used for EIP-712 signing.
// Not `alloy_sol_types::Eip712Domain`: this is the EIP-1193
// `eth_signTypedData_v4` wire shape (four required fields, numeric camelCase
// `chainId`, no `salt`), whereas alloy's is the hashing-side type with `Option`
// fields, `U256` `chainId` (hex when serialized), and a `salt`. Emitting the
// alloy shape would break JS wallet integrations. Bridge to the hashing-side
// type with `to_alloy_domain()`.
//
// ADR 0052, ADR 0040. Enforced by cargo check-source-fences.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedDataDomain {
    /// Human-readable protocol name.
    pub name: String,
    /// Domain version string.
    pub version: String,
    /// Numeric chain id for the typed-data domain.
    pub chain_id: ChainId,
    /// Contract address used as the domain verifier.
    pub verifying_contract: Address,
}

impl TypedDataDomain {
    /// Creates typed-data domain metadata for EIP-712 signing.
    #[inline]
    #[must_use]
    pub const fn new(
        name: String,
        version: String,
        chain_id: ChainId,
        verifying_contract: Address,
    ) -> Self {
        Self {
            name,
            version,
            chain_id,
            verifying_contract,
        }
    }

    /// Returns the canonical [`alloy_sol_types::Eip712Domain`] view of this
    /// domain.
    ///
    /// The returned value carries the four required EIP-712 domain fields —
    /// `name`, `version`, `chainId` (encoded as `uint256`), and
    /// `verifyingContract` — and leaves `salt` as `None`. This matches the
    /// `GPv2` settlement-contract domain shape, the
    /// `EIP712Domain(string name,string version,uint256 chainId,address
    /// verifyingContract)` type string used by every shipped digest path,
    /// and the EIP-1193 `eth_signTypedData_v4` wire shape expected by JS
    /// wallets (numeric `chainId`, lowercase 20-byte `verifyingContract`,
    /// no `salt`).
    #[must_use]
    pub fn to_alloy_domain(&self) -> alloy_sol_types::Eip712Domain {
        alloy_sol_types::Eip712Domain {
            name: Some(self.name.clone().into()),
            version: Some(self.version.clone().into()),
            chain_id: Some(alloy_primitives::U256::from(self.chain_id)),
            verifying_contract: Some(*self.verifying_contract.as_alloy()),
            salt: None,
        }
    }
}

/// A single EIP-712 typed-data field descriptor.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedDataField {
    /// Field name as it appears in the typed-data schema.
    pub name: String,
    #[serde(rename = "type")]
    /// Solidity type name for the field.
    pub kind: String,
}

impl TypedDataField {
    /// Creates an EIP-712 typed-data field descriptor.
    #[inline]
    #[must_use]
    pub const fn new(name: String, kind: String) -> Self {
        Self { name, kind }
    }
}

/// EIP-712 type map keyed by type name.
pub type TypedDataTypes = BTreeMap<String, Vec<TypedDataField>>;

/// Generic EIP-712 envelope shape used by typed helpers and signer payloads.
///
/// The signer-facing alias uses a canonical JSON string for `message` so the
/// payload travels as one self-contained, digest-complete value: domain,
/// full type map, primary-type name, and message together.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedDataEnvelope<M> {
    /// Domain metadata used to compute the typed-data digest.
    pub domain: TypedDataDomain,
    /// Primary type name for the payload.
    pub primary_type: String,
    /// Full type map including the primary type and `EIP712Domain`.
    pub types: TypedDataTypes,
    /// Payload message body.
    pub message: M,
}

/// Typed-data envelope that stores the message body as canonical JSON.
pub type TypedDataPayload = TypedDataEnvelope<String>;

impl<M> TypedDataEnvelope<M> {
    /// Creates an EIP-712 typed-data envelope.
    #[inline]
    #[must_use]
    pub const fn new(
        domain: TypedDataDomain,
        primary_type: String,
        types: TypedDataTypes,
        message: M,
    ) -> Self {
        Self {
            domain,
            primary_type,
            types,
            message,
        }
    }

    /// Returns the field list for the current primary type, if present.
    #[must_use]
    pub fn primary_type_fields(&self) -> Option<&[TypedDataField]> {
        self.types.get(&self.primary_type).map(Vec::as_slice)
    }

    /// Replaces the message body while preserving domain and type metadata.
    #[must_use]
    pub fn with_message<N>(self, message: N) -> TypedDataEnvelope<N> {
        TypedDataEnvelope {
            domain: self.domain,
            primary_type: self.primary_type,
            types: self.types,
            message,
        }
    }
}

impl TypedDataPayload {
    /// Returns the canonical JSON message body.
    #[must_use]
    pub const fn message_json(&self) -> &str {
        self.message.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_alloy_domain_emits_the_canonical_five_field_shape() {
        let domain = TypedDataDomain::new(
            "Gnosis Protocol".to_owned(),
            "v2".to_owned(),
            1,
            crate::types::Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap(),
        );

        let alloy = domain.to_alloy_domain();

        assert_eq!(alloy.name.as_deref(), Some("Gnosis Protocol"));
        assert_eq!(alloy.version.as_deref(), Some("v2"));
        assert_eq!(alloy.chain_id, Some(alloy_primitives::U256::from(1u64)));
        assert_eq!(
            alloy.verifying_contract,
            Some(*domain.verifying_contract.as_alloy())
        );
        assert!(alloy.salt.is_none());
    }
}
