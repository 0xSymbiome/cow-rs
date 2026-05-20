use std::{convert::Infallible, error::Error};

use cow_sdk::alloy_signer::LocalAlloyKeystoreSigner;
use cow_sdk::core::{
    Address, Amount, AsyncProvider, AsyncSigner, BlockHash, BlockInfo, ChainId, ContractCall,
    ContractHandle, HexData, SupportedChainId, TransactionHash, TransactionReceipt,
    TransactionRequest, TransactionStatus,
};
use serde_json::json;

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const ADDRESS: &str = "0x1111111111111111111111111111111111111111";
const HASH: &str = "0x13579bdf2468ace013579bdf2468ace013579bdf2468ace013579bdf2468ace0";

struct StaticAsyncProvider;

impl AsyncProvider for StaticAsyncProvider {
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
            transaction_hash.clone(),
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
        Ok(ContractHandle::new(address.clone(), abi_json.to_owned()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let signer = LocalAlloyKeystoreSigner::builder()
        .private_key(TEST_KEY)?
        .chain_id(SupportedChainId::Mainnet)
        .build()?;
    let provider = StaticAsyncProvider;
    let address = Address::new(ADDRESS)?;
    let code = provider.get_code(&address).await?;
    let signature = signer.sign_message(b"hello cow").await?;
    let receipt = provider
        .get_transaction_receipt(&TransactionHash::new(HASH)?)
        .await?
        .expect("static provider returns a receipt");

    let report = json!({
        "surface": "cow-sdk::alloy_signer with consumer async provider",
        "chainId": provider.get_chain_id().await?,
        "signer": signer.get_address().await?.to_hex_string(),
        "code": code.map(|data| data.to_hex_string()),
        "messageSignatureBytes": (signature.len() - 2) / 2,
        "receipt": {
            "status": receipt.status.map(|status| format!("{status:?}")),
            "blockNumber": receipt.block_number,
            "gasUsed": receipt.gas_used.map(|gas| gas.to_string())
        }
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
