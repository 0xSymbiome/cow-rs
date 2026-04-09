use cow_sdk_core::{
    Address, AsyncSigner, TransactionReceipt, TransactionRequest, TypedDataDomain, TypedDataField,
};
use serde_json::{Value, json};

use crate::{
    BrowserWalletError,
    provider::{Eip1193Provider, parse_quantity_to_decimal, transaction_to_rpc},
};

#[derive(Clone)]
pub struct Eip1193Signer {
    provider: Eip1193Provider,
    account_hint: Option<Address>,
}

impl Eip1193Signer {
    pub(crate) fn new(provider: Eip1193Provider, account_hint: Option<Address>) -> Self {
        Self {
            provider,
            account_hint,
        }
    }

    pub fn provider(&self) -> &Eip1193Provider {
        &self.provider
    }

    async fn account(&self) -> Result<Address, BrowserWalletError> {
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

    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error> {
        let account = self.account().await?;
        let typed_data = serde_json::to_string(&typed_data_payload(domain, fields, value_json)?)
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
            transaction_hash: hash.to_owned(),
        })
    }

    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<String, Self::Error> {
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

fn typed_data_payload(
    domain: &TypedDataDomain,
    fields: &[TypedDataField],
    value_json: &str,
) -> Result<Value, BrowserWalletError> {
    let primary_type = infer_primary_type(fields);
    let domain = serde_json::to_value(domain)
        .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;
    let fields = serde_json::to_value(fields)
        .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;
    let message = serde_json::from_str::<Value>(value_json)
        .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;

    Ok(json!({
        "types": {
            "EIP712Domain": [
                { "name": "name", "type": "string" },
                { "name": "version", "type": "string" },
                { "name": "chainId", "type": "uint256" },
                { "name": "verifyingContract", "type": "address" }
            ],
            primary_type: fields
        },
        "primaryType": primary_type,
        "domain": domain,
        "message": message,
    }))
}

fn infer_primary_type(fields: &[TypedDataField]) -> &'static str {
    if fields.iter().any(|field| field.name == "orderUids") {
        "OrderCancellations"
    } else if fields.iter().any(|field| field.name == "sellToken") {
        "Order"
    } else {
        "Message"
    }
}
