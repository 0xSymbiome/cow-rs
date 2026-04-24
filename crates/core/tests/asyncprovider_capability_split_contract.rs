use std::fmt;

use cow_sdk_core::{
    Address, Amount, AsyncProvider, AsyncSigner, AsyncSigningProvider, BlockInfo, ContractCall,
    ContractHandle, Hash32, HexData, Provider, Signer, TransactionHash, TransactionReceipt,
    TransactionRequest, TypedDataDomain, TypedDataField,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestError(&'static str);

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Clone)]
struct ReadOnlyProvider;

impl AsyncProvider for ReadOnlyProvider {
    type Error = TestError;

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(1)
    }

    async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        Ok(Some(HexData::new("0x6000").unwrap()))
    }

    async fn get_transaction_receipt(
        &self,
        transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(Some(TransactionReceipt {
            transaction_hash: transaction_hash.clone(),
        }))
    }

    async fn get_storage_at(
        &self,
        _address: &Address,
        _slot: &str,
    ) -> Result<HexData, Self::Error> {
        Ok(HexData::new("0x12").unwrap())
    }

    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Ok(HexData::new("0x34").unwrap())
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        Ok(format!("read:{}", request.method))
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo {
            number: 42,
            hash: None,
        })
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle {
            address: address.clone(),
            abi_json: abi_json.to_owned(),
        })
    }
}

#[derive(Clone)]
struct DirectAsyncSigner {
    address: Address,
}

impl AsyncSigner for DirectAsyncSigner {
    type Error = TestError;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(self.address.clone())
    }

    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        Ok(format!("message:{}", message.len()))
    }

    async fn sign_transaction(&self, tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok(format!("tx:{}", tx.to.is_some()))
    }

    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error> {
        Ok(format!(
            "{}:{}:{}",
            domain.name,
            fields.len(),
            value_json.len()
        ))
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionReceipt, Self::Error> {
        Ok(TransactionReceipt {
            transaction_hash: Hash32::new(format!("0x{}", "aa".repeat(32))).unwrap(),
        })
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Ok(Amount::from(21_000u32))
    }
}

#[derive(Clone)]
struct SigningAsyncProvider {
    read: ReadOnlyProvider,
    signer: DirectAsyncSigner,
}

impl AsyncProvider for SigningAsyncProvider {
    type Error = TestError;

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        self.read.get_chain_id().await
    }

    async fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error> {
        self.read.get_code(address).await
    }

    async fn get_transaction_receipt(
        &self,
        transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        self.read.get_transaction_receipt(transaction_hash).await
    }

    async fn get_storage_at(&self, address: &Address, slot: &str) -> Result<HexData, Self::Error> {
        self.read.get_storage_at(address, slot).await
    }

    async fn call(&self, tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        self.read.call(tx).await
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        self.read.read_contract(request).await
    }

    async fn get_block(&self, block_tag: &str) -> Result<BlockInfo, Self::Error> {
        self.read.get_block(block_tag).await
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        self.read.get_contract(address, abi_json).await
    }
}

impl AsyncSigningProvider for SigningAsyncProvider {
    type Signer = DirectAsyncSigner;

    async fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Ok(self.signer.clone())
    }
}

#[derive(Clone)]
struct SyncSigner {
    address: Address,
    provider_hint: Option<String>,
}

impl Signer for SyncSigner {
    type Provider = String;
    type Error = String;

    fn connect(&mut self, provider: Self::Provider) {
        self.provider_hint = Some(provider);
    }

    fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(self.address.clone())
    }

    fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        Ok(format!("sync-message:{}", message.len()))
    }

    fn sign_transaction(&self, tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok(format!("sync-tx:{}", tx.to.is_some()))
    }

    fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error> {
        Ok(format!(
            "sync:{}:{}:{}",
            domain.name,
            fields.len(),
            value_json.len()
        ))
    }

    fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionReceipt, Self::Error> {
        Ok(TransactionReceipt {
            transaction_hash: Hash32::new(format!("0x{}", "bb".repeat(32))).unwrap(),
        })
    }

    fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Ok(Amount::from(21_000u32))
    }
}

struct SyncProvider {
    signer: SyncSigner,
}

impl Provider for SyncProvider {
    type Signer = SyncSigner;
    type Error = String;

    fn signer_or_null(&self) -> Option<&Self::Signer> {
        Some(&self.signer)
    }

    fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(1)
    }

    fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        Ok(Some(HexData::new("0x6000").unwrap()))
    }

    fn get_transaction_receipt(
        &self,
        transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(Some(TransactionReceipt {
            transaction_hash: transaction_hash.clone(),
        }))
    }

    fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Ok(self.signer.clone())
    }

    fn get_storage_at(&self, _address: &Address, _slot: &str) -> Result<HexData, Self::Error> {
        Ok(HexData::new("0x12").unwrap())
    }

    fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Ok(HexData::new("0x34").unwrap())
    }

    fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        Ok(format!("read:{}", request.method))
    }

    fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo {
            number: 42,
            hash: None,
        })
    }

    fn set_signer(&mut self, signer: Self::Signer) {
        self.signer = signer;
    }

    fn set_provider(&mut self, provider_hint: String) {
        self.signer.provider_hint = Some(provider_hint);
    }

    fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle {
            address: address.clone(),
            abi_json: abi_json.to_owned(),
        })
    }
}

fn sample_address() -> Address {
    Address::new("0x1111111111111111111111111111111111111111").unwrap()
}

fn sample_hash() -> TransactionHash {
    Hash32::new(format!("0x{}", "cc".repeat(32))).unwrap()
}

fn sample_transaction() -> TransactionRequest {
    TransactionRequest {
        to: Some(sample_address()),
        data: Some(HexData::new("0x1234").unwrap()),
        value: Some(Amount::zero()),
        gas_limit: Some(Amount::from(21_000u32)),
    }
}

fn sample_contract_call() -> ContractCall {
    ContractCall {
        address: sample_address(),
        method: "balanceOf".to_owned(),
        abi_json: "[]".to_owned(),
        args_json: "[]".to_owned(),
    }
}

async fn assert_read_methods<P>(provider: &P)
where
    P: AsyncProvider,
    P::Error: fmt::Debug,
{
    let address = sample_address();
    let tx_hash = sample_hash();
    let tx = sample_transaction();
    let call = sample_contract_call();

    assert_eq!(provider.get_chain_id().await.unwrap(), 1);
    assert_eq!(
        provider.get_code(&address).await.unwrap().unwrap(),
        HexData::new("0x6000").unwrap()
    );
    assert_eq!(
        provider
            .get_transaction_receipt(&tx_hash)
            .await
            .unwrap()
            .unwrap()
            .transaction_hash,
        tx_hash
    );
    assert_eq!(
        provider.get_storage_at(&address, "0x0").await.unwrap(),
        HexData::new("0x12").unwrap()
    );
    assert_eq!(
        provider.call(&tx).await.unwrap(),
        HexData::new("0x34").unwrap()
    );
    assert_eq!(
        provider.read_contract(&call).await.unwrap(),
        "read:balanceOf"
    );
    assert_eq!(provider.get_block("latest").await.unwrap().number, 42);
    assert_eq!(
        provider
            .get_contract(&address, "[{\"type\":\"function\"}]")
            .await
            .unwrap()
            .abi_json,
        "[{\"type\":\"function\"}]"
    );
}

#[tokio::test]
async fn read_only_async_provider_dispatches_all_read_methods_without_signer_wiring() {
    let provider = ReadOnlyProvider;

    assert_read_methods(&provider).await;
}

#[tokio::test]
async fn signing_extension_preserves_signer_creation() {
    let provider = SigningAsyncProvider {
        read: ReadOnlyProvider,
        signer: DirectAsyncSigner {
            address: sample_address(),
        },
    };

    assert_read_methods(&provider).await;
    let signer = AsyncSigningProvider::create_signer(&provider, "primary")
        .await
        .unwrap();
    assert_eq!(signer.get_address().await.unwrap(), sample_address());
    assert_eq!(
        signer.estimate_gas(&sample_transaction()).await.unwrap(),
        Amount::from(21_000u32)
    );
}

#[tokio::test]
async fn sync_provider_blankets_split_read_and_signing_capabilities() {
    let provider = SyncProvider {
        signer: SyncSigner {
            address: sample_address(),
            provider_hint: None,
        },
    };

    assert_read_methods(&provider).await;
    let signer = AsyncSigningProvider::create_signer(&provider, "sync")
        .await
        .unwrap();
    assert_eq!(
        AsyncSigner::get_address(&signer).await.unwrap(),
        sample_address()
    );
    assert_eq!(
        AsyncSigner::sign_message(&signer, b"cow").await.unwrap(),
        "sync-message:3"
    );
}
