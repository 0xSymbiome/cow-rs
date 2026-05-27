use std::{convert::Infallible, error::Error};

use cow_sdk::alloy_provider::RpcAlloyProvider;
use cow_sdk::core::{
    Address, Amount, Provider, Signer, TransactionBroadcast, TransactionRequest, TypedDataDomain,
    TypedDataField,
};
use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const ADDRESS: &str = "0x1111111111111111111111111111111111111111";

struct StaticSigner;

impl Signer for StaticSigner {
    type Error = Infallible;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(Address::new(ADDRESS).unwrap())
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Ok(format!("0x{}1b", "11".repeat(64)))
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok("0x01".to_owned())
    }

    async fn sign_typed_data(
        &self,
        _domain: &TypedDataDomain,
        _fields: &[TypedDataField],
        _value_json: &str,
    ) -> Result<String, Self::Error> {
        Ok(format!("0x{}1b", "22".repeat(64)))
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Ok(TransactionBroadcast::new(
            cow_sdk::core::TransactionHash::new(format!("0x{}", "33".repeat(32))).unwrap(),
        ))
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Ok(Amount::from(21_000u32))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x1",
        })))
        .mount(&server)
        .await;

    let provider = RpcAlloyProvider::builder()
        .http(server.uri())?
        .build()
        .await?;
    let signer = StaticSigner;
    let report = json!({
        "surface": "cow-sdk::alloy_provider with consumer async signer",
        "chainId": provider.get_chain_id().await?,
        "signer": signer.get_address().await?.to_hex_string(),
        "messageSignatureBytes": (signer.sign_message(b"hello").await?.len() - 2) / 2
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
