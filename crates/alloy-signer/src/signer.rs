//! `AsyncSigner` implementation backed by Alloy's local private-key signer.

use std::{fmt, sync::Arc};

use alloy_dyn_abi::{
    DynSolType,
    eip712::{PropertyDef, Resolver, TypeDef, TypedData},
};
use alloy_primitives::Signature;
use alloy_signer::Signer as AlloySigner;
use alloy_signer_local::PrivateKeySigner;
use cow_sdk_core::{
    Address, Amount, AsyncSigner, ChainId, TransactionBroadcast, TransactionRequest,
    TypedDataDomain, TypedDataField, TypedDataPayload, TypedDataTypes,
};

use crate::{builder::LocalAlloyKeystoreSignerBuilder, error::AsyncSignerError};

/// Alloy-backed local-keystore signer for native SDK consumers.
#[derive(Clone)]
pub struct LocalAlloyKeystoreSigner {
    inner: Arc<PrivateKeySigner>,
    chain_id: ChainId,
}

impl LocalAlloyKeystoreSigner {
    /// Returns a fresh typestate builder.
    pub const fn builder() -> LocalAlloyKeystoreSignerBuilder {
        LocalAlloyKeystoreSignerBuilder::new()
    }

    pub(crate) fn from_parts(inner: PrivateKeySigner, chain_id: ChainId) -> Self {
        let bound = inner.with_chain_id(Some(chain_id));
        Self {
            inner: Arc::new(bound),
            chain_id,
        }
    }

    /// Returns the chain id bound to this signer at construction time.
    #[must_use]
    pub const fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub(crate) fn upstream_signer(&self) -> &PrivateKeySigner {
        &self.inner
    }
}

impl fmt::Debug for LocalAlloyKeystoreSigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalAlloyKeystoreSigner")
            .field("inner", &"[redacted]")
            .field("chain_id", &self.chain_id)
            .finish()
    }
}

impl AsyncSigner for LocalAlloyKeystoreSigner {
    type Error = AsyncSignerError;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        let alloy_address = AlloySigner::address(self.upstream_signer());
        Ok(Address::from_bytes(alloy_address.into_array()))
    }

    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        let signature = AlloySigner::sign_message(self.upstream_signer(), message)
            .await
            .map_err(|error| AsyncSignerError::from_alloy_signer(&error))?;
        Ok(alloy_signature_to_hex(&signature)?)
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Err(AsyncSignerError::ProviderRequired {
            method: "sign_transaction",
        })
    }

    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        #[cfg(feature = "eip712")]
        {
            let typed = typed_data_from_payload(payload).map_err(AsyncSignerError::Validation)?;
            let signature = AlloySigner::sign_dynamic_typed_data(self.upstream_signer(), &typed)
                .await
                .map_err(|error| AsyncSignerError::from_alloy_signer(&error))?;
            Ok(alloy_signature_to_hex(&signature)?)
        }

        #[cfg(not(feature = "eip712"))]
        {
            let _ = payload;
            Err(AsyncSignerError::Unsupported(
                "sign_typed_data_payload requires the eip712 feature",
            ))
        }
    }

    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error> {
        #[cfg(feature = "eip712")]
        {
            let typed = typed_data_from_flat_fields(domain, fields, value_json)
                .map_err(AsyncSignerError::Validation)?;
            let signature = AlloySigner::sign_dynamic_typed_data(self.upstream_signer(), &typed)
                .await
                .map_err(|error| AsyncSignerError::from_alloy_signer(&error))?;
            Ok(alloy_signature_to_hex(&signature)?)
        }

        #[cfg(not(feature = "eip712"))]
        {
            let _ = (domain, fields, value_json);
            Err(AsyncSignerError::Unsupported(
                "sign_typed_data requires the eip712 feature",
            ))
        }
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Err(AsyncSignerError::ProviderRequired {
            method: "send_transaction",
        })
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Err(AsyncSignerError::ProviderRequired {
            method: "estimate_gas",
        })
    }
}

/// Builds an [`alloy_dyn_abi::eip712::TypedData`] value from the cow
/// typed-data payload.
///
/// The cow [`TypedDataDomain`] is aliased onto
/// [`alloy_sol_types::Eip712Domain`] so the domain field is consumed
/// directly without a conversion step. The cow [`TypedDataTypes`] map
/// is translated into the alloy [`Resolver`] shape one field at a
/// time. The canonical-JSON message body is parsed into a
/// `serde_json::Value` so alloy's dynamic ABI encoder can drive the
/// signing-hash computation.
fn typed_data_from_payload(payload: &TypedDataPayload) -> Result<TypedData, String> {
    let resolver = build_resolver(&payload.types, &payload.primary_type)?;
    let message = serde_json::from_str(payload.message_json())
        .map_err(|error| format!("typed-data message JSON parse error: {error}"))?;

    let typed = TypedData {
        domain: payload.domain.clone(),
        resolver,
        primary_type: payload.primary_type.clone(),
        message,
    };
    typed
        .eip712_signing_hash()
        .map_err(|error| format!("alloy TypedData rejected by eip712_signing_hash: {error}"))?;
    Ok(typed)
}

/// Builds an [`alloy_dyn_abi::eip712::TypedData`] value from a legacy
/// flat typed-data field set.
///
/// The flat shape has no primary type name, so this compatibility path
/// uses the placeholder primary type `"Message"`. Canonical `CoW` order
/// signing must use [`typed_data_from_payload`] so the original primary
/// type is preserved.
fn typed_data_from_flat_fields(
    domain: &TypedDataDomain,
    fields: &[TypedDataField],
    value_json: &str,
) -> Result<TypedData, String> {
    let mut types = TypedDataTypes::new();
    types.insert("Message".to_owned(), fields.to_vec());
    let payload = TypedDataPayload::new(
        domain.clone(),
        "Message".to_owned(),
        types,
        value_json.to_owned(),
    );
    typed_data_from_payload(&payload)
}

fn build_resolver(types: &TypedDataTypes, primary_type: &str) -> Result<Resolver, String> {
    if !types.contains_key(primary_type) {
        return Err(format!(
            "primary type `{primary_type}` is not defined in the typed-data type map"
        ));
    }

    let mut resolver = Resolver::default();
    for (type_name, fields) in types {
        if type_name == "EIP712Domain" {
            continue;
        }

        let props = fields
            .iter()
            .map(|field| property_def(type_name, field))
            .collect::<Result<Vec<_>, _>>()?;
        let type_def = TypeDef::new(type_name.clone(), props)
            .map_err(|error| format!("TypeDef::new for `{type_name}` failed: {error}"))?;
        resolver.ingest(type_def);
    }

    Ok(resolver)
}

fn property_def(type_name: &str, field: &TypedDataField) -> Result<PropertyDef, String> {
    DynSolType::parse(&field.kind).map_err(|error| {
        format!(
            "type `{}` field `{}` has unsupported EIP-712 kind `{}`: {error}",
            type_name, field.name, field.kind
        )
    })?;
    PropertyDef::new(field.kind.clone(), field.name.clone()).map_err(|error| {
        format!(
            "type `{}` field `{}` has invalid EIP-712 kind `{}`: {error}",
            type_name, field.name, field.kind
        )
    })
}

/// Hex-encodes an Alloy ECDSA signature through the canonical `CoW`
/// recovery-byte normalizer so the signer surface emits the same
/// `0x`-prefixed lowercase form as every other `CoW` signer.
fn alloy_signature_to_hex(
    signature: &Signature,
) -> Result<String, cow_sdk_contracts::ContractsError> {
    let raw = format!("0x{}", hex::encode(signature.as_bytes()));
    cow_sdk_contracts::normalized_ecdsa_signature(&raw)
}

#[cfg(test)]
mod tests {
    use cow_sdk_core::{AsyncSigner as _, SupportedChainId};

    use super::*;

    const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

    #[tokio::test]
    async fn get_address_returns_signer_address() {
        let signer = test_signer();

        assert_eq!(
            signer.get_address().await.unwrap().to_hex_string(),
            "0x70997970c51812dc3a010c7d01b50e0d17dc79c8"
        );
    }

    #[tokio::test]
    async fn transaction_methods_return_provider_required() {
        let signer = test_signer();
        let tx = TransactionRequest::default();

        assert!(matches!(
            signer.sign_transaction(&tx).await,
            Err(AsyncSignerError::ProviderRequired {
                method: "sign_transaction"
            })
        ));
        assert!(matches!(
            signer.send_transaction(&tx).await,
            Err(AsyncSignerError::ProviderRequired {
                method: "send_transaction"
            })
        ));
        assert!(matches!(
            signer.estimate_gas(&tx).await,
            Err(AsyncSignerError::ProviderRequired {
                method: "estimate_gas"
            })
        ));
    }

    #[test]
    fn typed_data_from_payload_round_trip() {
        let payload = simple_payload("Greeting");
        let typed = typed_data_from_payload(&payload).unwrap();

        assert_eq!(typed.primary_type, "Greeting");
        assert_eq!(
            typed.message,
            serde_json::json!({
                "message": "hello"
            })
        );
        typed.eip712_signing_hash().unwrap();
    }

    #[test]
    fn flat_conversion_uses_message_placeholder_primary_type() {
        let payload = simple_payload("Greeting");
        let typed = typed_data_from_flat_fields(
            &payload.domain,
            payload.primary_type_fields().unwrap(),
            payload.message_json(),
        )
        .unwrap();

        assert_eq!(typed.primary_type, "Message");
        assert_ne!(
            typed.eip712_signing_hash().unwrap(),
            typed_data_from_payload(&payload)
                .unwrap()
                .eip712_signing_hash()
                .unwrap()
        );
    }

    #[test]
    fn alloy_signature_to_hex_format() {
        let mut bytes = [1u8; 65];
        bytes[64] = 27;
        let signature = Signature::from_raw(&bytes).unwrap();
        let encoded = alloy_signature_to_hex(&signature).unwrap();

        assert_eq!(encoded.len(), 132);
        assert!(encoded.starts_with("0x"));
        assert!(encoded.ends_with("1b"));
    }

    fn simple_payload(primary_type: &str) -> TypedDataPayload {
        let domain = TypedDataDomain {
            name: Some("CoW".into()),
            version: Some("1".into()),
            chain_id: Some(alloy_primitives::U256::from(1u64)),
            verifying_contract: Some(
                *Address::new("0x1111111111111111111111111111111111111111")
                    .unwrap()
                    .as_alloy(),
            ),
            salt: None,
        };
        let mut types = TypedDataTypes::new();
        types.insert(
            primary_type.to_owned(),
            vec![TypedDataField::new(
                "message".to_owned(),
                "string".to_owned(),
            )],
        );
        TypedDataPayload::new(
            domain,
            primary_type.to_owned(),
            types,
            serde_json::json!({ "message": "hello" }).to_string(),
        )
    }

    fn test_signer() -> LocalAlloyKeystoreSigner {
        LocalAlloyKeystoreSigner::builder()
            .private_key(TEST_KEY)
            .unwrap()
            .chain_id(SupportedChainId::Sepolia)
            .build()
            .unwrap()
    }
}
