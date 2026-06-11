//! Composing an SDK Alloy leaf with a consumer-supplied trait implementation,
//! in both directions: an SDK provider leaf paired with your own `Signer`, and
//! an SDK signer leaf paired with your own `Provider`. The `cow_sdk::core`
//! `Provider` and `Signer` seams accept any async implementation, so a consumer
//! can keep one half from the SDK and supply the other — for example an
//! HSM-backed signer in front of the SDK provider, or a bespoke RPC provider
//! behind the SDK signer. The mixed pair is proven end-to-end by driving it
//! through `cow_sdk::trading::submit_and_wait_for_receipt` against a wiremock
//! JSON-RPC server: the consumer signer broadcasts, the SDK provider polls the
//! mined receipt.

use std::{convert::Infallible, error::Error};

use cow_sdk::alloy_provider::RpcAlloyProvider;
use cow_sdk::alloy_signer::LocalAlloySigner;
use cow_sdk::core::{
    Address, Amount, BlockHash, BlockInfo, ChainId, ContractCall, ContractHandle, HexData,
    Provider, Signer, SupportedChainId, TransactionBroadcast, TransactionHash, TransactionReceipt,
    TransactionRequest, TransactionStatus, TypedDataPayload, address,
};
use cow_sdk::trading::{WaitOptions, submit_and_wait_for_receipt};
use cow_sdk_examples_native::support::{TEST_KEY, TX_HASH, mount_rpc};
use serde_json::json;
use wiremock::MockServer;

const ADDRESS: Address = address!("0x1111111111111111111111111111111111111111");

/// A consumer-supplied signer behind the public `Signer` trait — stand-in for
/// an HSM, remote KMS, or hardware wallet exposing the same async interface.
/// Its broadcast acknowledgement carries the canned RPC fixture hash, the way
/// a real backend returns the hash of the transaction it just submitted.
struct StaticSigner;

impl Signer for StaticSigner {
    type Error = Infallible;

    async fn address(&self) -> Result<Address, Self::Error> {
        Ok(ADDRESS)
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Ok(format!("0x{}1b", "11".repeat(64)))
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok("0x01".to_owned())
    }

    async fn sign_typed_data_payload(
        &self,
        _payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        Ok(format!("0x{}1b", "22".repeat(64)))
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Ok(TransactionBroadcast::new(
            TransactionHash::new(TX_HASH).expect("example transaction hash must remain valid"),
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
            Some(BlockHash::new(TX_HASH).unwrap()),
            Some(Amount::from(21_000u64)),
            Some(ADDRESS),
            Some(ADDRESS),
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
    // `mount_rpc` serves the JSON-RPC fixtures (chain id, mined receipt) the
    // provider leaf reads over real HTTP.
    let server = MockServer::start().await;
    mount_rpc(&server).await;
    let sdk_provider = RpcAlloyProvider::builder().http(server.uri())?.build()?;
    let consumer_signer = StaticSigner;

    // The mixed pair driven through one real SDK seam: the consumer signer
    // broadcasts (an HSM would do this out of process), then the SDK provider
    // polls the wiremock RPC until the mined receipt arrives.
    let tx = TransactionRequest::new(
        Some(ADDRESS),
        None,
        Some(Amount::ZERO),
        Some(Amount::from(21_000u32)),
    );
    let receipt = submit_and_wait_for_receipt(
        &consumer_signer,
        &sdk_provider,
        &tx,
        WaitOptions::approve_default(),
    )
    .await?;

    // Direction 2: SDK signer leaf (`LocalAlloySigner`) + consumer `Provider`.
    let sdk_signer = LocalAlloySigner::builder()
        .private_key(TEST_KEY)?
        .chain_id(SupportedChainId::Mainnet)
        .build()?;
    let consumer_provider = StaticProvider;

    let code = consumer_provider.get_code(&ADDRESS).await?;

    let report = json!({
        "surface": "cow_sdk::core::{Provider, Signer} + cow_sdk::trading::submit_and_wait_for_receipt",
        "sdkProviderWithConsumerSigner": {
            "chainId": sdk_provider.get_chain_id().await?,
            "signer": consumer_signer.address().await?.to_hex_string(),
            "submitAndWaitReceipt": {
                "transactionHash": receipt.transaction_hash.to_hex_string(),
                "status": receipt.status,
                "blockNumber": receipt.block_number
            }
        },
        "sdkSignerWithConsumerProvider": {
            "chainId": consumer_provider.get_chain_id().await?,
            "signer": sdk_signer.address().await?.to_hex_string(),
            "code": code.map(|data| data.to_hex_string())
        }
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
