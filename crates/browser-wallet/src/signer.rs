//! Typed EIP-1193 signer implementation for browser wallets.

use cow_sdk_core::{
    Address, Amount, AsyncSigner, SupportedChainId, TransactionReceipt, TransactionRequest,
    TypedDataDomain, TypedDataField, TypedDataPayload, TypedDataTypes,
};
use serde_json::{Value, json};

use crate::{
    BrowserWalletError,
    provider::{Eip1193Provider, parse_quantity_to_decimal, transaction_to_rpc},
};

/// Browser-wallet signer that implements [`cow_sdk_core::AsyncSigner`].
#[derive(Debug, Clone)]
pub struct Eip1193Signer {
    provider: Eip1193Provider,
    account_hint: Option<Address>,
    expected_chain_id: Option<SupportedChainId>,
}

impl Eip1193Signer {
    pub(crate) fn new(provider: Eip1193Provider, account_hint: Option<Address>) -> Self {
        Self {
            provider,
            account_hint,
            expected_chain_id: None,
        }
    }

    /// Returns the provider associated with this signer.
    #[must_use]
    pub fn provider(&self) -> &Eip1193Provider {
        &self.provider
    }

    /// Returns a copy of this signer bound to one expected chain id.
    ///
    /// Chain-bound signers revalidate the wallet session chain before address,
    /// signature, gas, and transaction operations.
    #[must_use]
    pub fn with_expected_chain(mut self, chain_id: SupportedChainId) -> Self {
        self.expected_chain_id = Some(chain_id);
        self
    }

    /// Returns the expected chain id fixed on this signer, when one is set.
    #[must_use]
    pub fn expected_chain_id(&self) -> Option<SupportedChainId> {
        self.expected_chain_id
    }

    async fn ensure_expected_chain(&self) -> Result<(), BrowserWalletError> {
        let Some(expected_chain_id) = self.expected_chain_id else {
            return Ok(());
        };
        let session_chain_id = self.provider.query_chain_id().await?;
        let expected_chain_id = u64::from(expected_chain_id);
        if session_chain_id != expected_chain_id {
            return Err(BrowserWalletError::SessionChainMismatch {
                expected_chain_id,
                session_chain_id,
            });
        }
        Ok(())
    }

    fn validate_typed_data_chain(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<(), BrowserWalletError> {
        let Some(expected_chain_id) = self.expected_chain_id else {
            return Ok(());
        };
        let expected_chain_id = u64::from(expected_chain_id);
        if payload.domain.chain_id != expected_chain_id {
            return Err(BrowserWalletError::TypedDataChainMismatch {
                expected_chain_id,
                typed_data_chain_id: payload.domain.chain_id,
            });
        }
        Ok(())
    }

    async fn account(&self) -> Result<Address, BrowserWalletError> {
        self.ensure_expected_chain().await?;
        if let Some(address) = &self.account_hint {
            return Ok(address.clone());
        }
        if let Some(address) = self.provider.selected_account() {
            return Ok(address);
        }
        let accounts = self.provider.query_accounts(false).await?;
        accounts.first().cloned().ok_or_else(|| {
            BrowserWalletError::malformed_response(
                "eth_accounts",
                "wallet does not currently expose any account",
            )
        })
    }

    /// Signs typed data through the legacy compatibility bridge.
    ///
    /// This helper is intentionally narrow. It supports only the `CoW` order and order-cancellation
    /// field layouts that legacy browser-wallet integrations expect. For other primary types, use
    /// [`cow_sdk_core::AsyncSigner::sign_typed_data_payload`] with an explicit
    /// [`TypedDataPayload`].
    ///
    /// # Errors
    ///
    /// Returns an error when the field layout does not match a supported compatibility payload,
    /// when account resolution fails, when request serialization fails, or when the wallet rejects
    /// the signing request.
    pub async fn sign_typed_data_compatibility(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, BrowserWalletError> {
        let payload = compatibility_typed_data_payload(domain, fields, value_json)?;
        self.sign_typed_data_payload(&payload).await
    }
}

#[allow(async_fn_in_trait)]
impl AsyncSigner for Eip1193Signer {
    type Error = BrowserWalletError;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        self.account().await
    }

    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        let account = self.account().await?;
        let value = self
            .provider
            .request(
                "personal_sign",
                Some(json!([
                    format!("0x{}", hex::encode(message)),
                    account.as_str()
                ])),
            )
            .await?;
        value.as_str().map(str::to_owned).ok_or_else(|| {
            BrowserWalletError::malformed_response(
                "personal_sign",
                "wallet must return a signature string",
            )
        })
    }

    async fn sign_transaction(&self, tx: &TransactionRequest) -> Result<String, Self::Error> {
        let from = self.account().await?;
        let value = self
            .provider
            .request(
                "eth_signTransaction",
                Some(json!([transaction_to_rpc(tx, Some(&from))?])),
            )
            .await?;
        value.as_str().map(str::to_owned).ok_or_else(|| {
            BrowserWalletError::malformed_response(
                "eth_signTransaction",
                "wallet must return a signed transaction string",
            )
        })
    }

    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        self.validate_typed_data_chain(payload)?;
        let account = self.account().await?;
        let typed_data = serde_json::to_string(&typed_data_request(payload)?)
            .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;
        let value = self
            .provider
            .request(
                "eth_signTypedData_v4",
                Some(json!([account.as_str(), typed_data])),
            )
            .await?;
        value.as_str().map(str::to_owned).ok_or_else(|| {
            BrowserWalletError::malformed_response(
                "eth_signTypedData_v4",
                "wallet must return a signature string",
            )
        })
    }

    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error> {
        self.sign_typed_data_compatibility(domain, fields, value_json)
            .await
    }

    async fn send_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionReceipt, Self::Error> {
        let from = self.account().await?;
        let value = self
            .provider
            .request(
                "eth_sendTransaction",
                Some(json!([transaction_to_rpc(tx, Some(&from))?])),
            )
            .await?;
        let hash = value.as_str().ok_or_else(|| {
            BrowserWalletError::malformed_response(
                "eth_sendTransaction",
                "wallet must return a transaction hash",
            )
        })?;
        Ok(TransactionReceipt {
            transaction_hash: cow_sdk_core::TransactionHash::new(hash)?,
        })
    }

    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        let from = self.account().await?;
        let value = self
            .provider
            .request(
                "eth_estimateGas",
                Some(json!([transaction_to_rpc(tx, Some(&from))?])),
            )
            .await?;
        parse_quantity_to_decimal(&value, "eth_estimateGas")
    }
}

fn typed_data_request(payload: &TypedDataPayload) -> Result<Value, BrowserWalletError> {
    if payload.primary_type_fields().is_none() {
        return Err(BrowserWalletError::serialization(format!(
            "typed-data payload must include a definition for primary type `{}`",
            payload.primary_type
        )));
    }

    let domain = serde_json::to_value(&payload.domain)
        .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;
    let types = serde_json::to_value(&payload.types)
        .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;
    let message = serde_json::from_str::<Value>(payload.message_json())
        .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;

    Ok(json!({
        "types": types,
        "primaryType": payload.primary_type,
        "domain": domain,
        "message": message,
    }))
}

fn compatibility_typed_data_payload(
    domain: &TypedDataDomain,
    fields: &[TypedDataField],
    value_json: &str,
) -> Result<TypedDataPayload, BrowserWalletError> {
    let primary_type = compatibility_primary_type(fields)?;
    let mut types = TypedDataTypes::new();
    types.insert(primary_type.to_owned(), fields.to_vec());
    types.insert("EIP712Domain".to_owned(), domain_type_fields());

    Ok(TypedDataPayload {
        domain: domain.clone(),
        primary_type: primary_type.to_owned(),
        types,
        message: value_json.to_owned(),
    })
}

fn compatibility_primary_type(
    fields: &[TypedDataField],
) -> Result<&'static str, BrowserWalletError> {
    if matches_fields(fields, ORDER_COMPATIBILITY_FIELDS) {
        Ok("Order")
    } else if matches_fields(fields, ORDER_CANCELLATIONS_COMPATIBILITY_FIELDS) {
        Ok("OrderCancellations")
    } else {
        Err(BrowserWalletError::serialization(
            "legacy sign_typed_data compatibility supports only CoW order and order cancellation payloads; use sign_typed_data_payload for explicit primary types",
        ))
    }
}

const ORDER_COMPATIBILITY_FIELDS: &[(&str, &str)] = &[
    ("sellToken", "address"),
    ("buyToken", "address"),
    ("receiver", "address"),
    ("sellAmount", "uint256"),
    ("buyAmount", "uint256"),
    ("validTo", "uint32"),
    ("appData", "bytes32"),
    ("feeAmount", "uint256"),
    ("kind", "string"),
    ("partiallyFillable", "bool"),
    ("sellTokenBalance", "string"),
    ("buyTokenBalance", "string"),
];

const ORDER_CANCELLATIONS_COMPATIBILITY_FIELDS: &[(&str, &str)] = &[("orderUids", "bytes[]")];

fn matches_fields(fields: &[TypedDataField], expected: &[(&str, &str)]) -> bool {
    fields.len() == expected.len()
        && fields
            .iter()
            .zip(expected.iter())
            .all(|(field, (name, kind))| field.name == *name && field.kind == *kind)
}

fn domain_type_fields() -> Vec<TypedDataField> {
    [
        ("name", "string"),
        ("version", "string"),
        ("chainId", "uint256"),
        ("verifyingContract", "address"),
    ]
    .into_iter()
    .map(|(name, kind)| TypedDataField {
        name: name.to_owned(),
        kind: kind.to_owned(),
    })
    .collect()
}
