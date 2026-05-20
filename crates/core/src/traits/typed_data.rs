use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Typed-data domain metadata used for EIP-712 signing.
///
/// Aliased onto [`alloy_sol_types::Eip712Domain`] so the cow primitive
/// layer routes through one canonical EIP-712 domain shape across the
/// SDK and the `alloy-sol-types` reference implementation. Construct
/// values with the [`alloy_sol_types::eip712_domain!`] macro or a
/// direct struct-literal expression:
///
/// ```rust
/// use alloy_primitives::{Address, U256, address};
/// use alloy_sol_types::Eip712Domain;
/// use cow_sdk_core::TypedDataDomain;
///
/// let domain: TypedDataDomain = Eip712Domain {
///     name: Some("Gnosis Protocol".into()),
///     version: Some("v2".into()),
///     chain_id: Some(U256::from(1u64)),
///     verifying_contract: Some(address!("9008D19f58AAbD9eD0D60971565AA8510560ab41")),
///     salt: None,
/// };
/// # let _ = domain;
/// ```
pub type TypedDataDomain = alloy_sol_types::Eip712Domain;

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
