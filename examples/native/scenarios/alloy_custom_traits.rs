//! Composing an SDK Alloy leaf with a consumer-supplied trait implementation,
//! in both directions: an SDK provider leaf paired with your own `Signer`, and
//! an SDK signer leaf paired with your own `Provider`. The `cow_sdk::core`
//! `Provider` and `Signer` seams accept any async implementation, so a consumer
//! can keep one half from the SDK and supply the other — for example an
//! HSM-backed signer in front of the SDK provider, or a bespoke RPC provider
//! behind the SDK signer.

use std::{convert::Infallible, error::Error};

use cow_sdk::alloy_provider::RpcAlloyProvider;
use cow_sdk::alloy_signer::LocalAlloyKeystoreSigner;
use cow_sdk::core::{
    BlockHash, BlockInfo, ChainId, ContractCall, ContractHandle, HexData, Provider, Signer,
    TransactionBroadcast, TransactionHash, TransactionReceipt, TransactionRequest,
    TransactionStatus, TypedDataDomain, TypedDataField,
};
use cow_sdk::prelude::{Address, Amount, SupportedChainId};
use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const ADDRESS: &str = "0x1111111111111111111111111111111111111111";
const HASH: &str = "0x13579bdf2468ace013579bdf2468ace013579bdf2468ace013579bdf2468ace0";
const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

/// A consumer-supplied signer behind the public `Signer` trait — stand-in for
/// an HSM, remote KMS, or hardware wallet exposing the same async interface.
struct StaticSigner;

impl Signer for StaticSigner {
    type Error = Infallible;

    async fn address(&self) -> Result<Address, Self::Error> {
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
            TransactionHash::new(format!("0x{}", "33".repeat(32))).unwrap(),
        ))
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Ok(Amount::from(21_000u32))
    }
}

/// A consumer-supplied provider behind the public `Provider` trait — stand-in
/// for a private RPC, an indexer, or a cache exposing the same async interface.
struct StaticProvider;

impl Provider for StaticProvider {
    type Error = Infallible;

    async fn get_chain_id(&self) -> Result<ChainId, Self::Error> {
        Ok(u64::from(SupportedChainId::Mainnet))
    }

    async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        Ok(Some(HexData::new("0x60016002").unwrap()))
    }

    async fn get_transaction_receipt(
        &self,
        transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(Some(TransactionReceipt::from_parts(
            *transaction_hash,
            Some(TransactionStatus::Success),
            Some(1234),
            Some(BlockHash::new(HASH).unwrap()),
            Some(Amount::from(21_000u64)),
            Some(Address::new(ADDRESS).unwrap()),
            Some(Address::new(ADDRESS).unwrap()),
        )))
    }

    async fn get_storage_at(
        &self,
        _address: &Address,
        _slot: &str,
    ) -> Result<HexData, Self::Error> {
        Ok(HexData::new(format!("0x{:0>64}", "0")).unwrap())
    }

    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Ok(HexData::new("0x").unwrap())
    }

    async fn read_contract(&self, _request: &ContractCall) -> Result<String, Self::Error> {
        Ok(r#""42""#.to_owned())
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(1, None))
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(*address, abi_json.to_owned()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Direction 1: SDK provider leaf (`RpcAlloyProvider`) + consumer `Signer`.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x1",
        })))
        .mount(&server)
        .await;
    let sdk_provider = RpcAlloyProvider::builder().http(server.uri())?.build().await?;
    let consumer_signer = StaticSigner;

    // Direction 2: SDK signer leaf (`LocalAlloyKeystoreSigner`) + consumer `Provider`.
    let sdk_signer = LocalAlloyKeystoreSigner::builder()
        .private_key(TEST_KEY)?
        .chain_id(SupportedChainId::Mainnet)
        .build()?;
    let consumer_provider = StaticProvider;

    let address = Address::new(ADDRESS)?;
    let code = consumer_provider.get_code(&address).await?;
    let receipt = consumer_provider
        .get_transaction_receipt(&TransactionHash::new(HASH)?)
        .await?
        .expect("static provider returns a receipt");

    let report = json!({
        "surface": "cow-sdk::core::{Provider, Signer} seams accept consumer trait impls",
        "sdkProviderWithConsumerSigner": {
            "chainId": sdk_provider.get_chain_id().await?,
            "signer": consumer_signer.address().await?.to_hex_string(),
            "messageSignatureBytes": (consumer_signer.sign_message(b"hello").await?.len() - 2) / 2
        },
        "sdkSignerWithConsumerProvider": {
            "chainId": consumer_provider.get_chain_id().await?,
            "signer": sdk_signer.address().await?.to_hex_string(),
            "code": code.map(|data| data.to_hex_string()),
            "receipt": {
                "status": receipt.status.map(|status| format!("{status:?}")),
                "blockNumber": receipt.block_number,
                "gasUsed": receipt.gas_used.map(|gas| gas.to_string())
            }
        }
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
