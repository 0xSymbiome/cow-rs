use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::types::{Address, ChainId};
/// Typed-data domain metadata used for EIP-712 signing.
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
/// The signer-facing alias uses a canonical JSON string for `message` so
/// existing `Signer` implementors can keep the legacy `sign_typed_data`
/// method and still gain additive compatibility through the default
/// `sign_typed_data_payload` implementation.
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
