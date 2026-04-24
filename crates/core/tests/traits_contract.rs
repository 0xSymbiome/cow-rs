use cow_sdk_core::{
    Address, Amount, AsyncProvider, AsyncSigner, AsyncSigningProvider, BlockInfo, ContractCall,
    ContractHandle, GraphTransport, Hash32, HexData, PinningTransport, Provider, Signer,
    TransactionReceipt, TransactionRequest, TypedDataDomain, TypedDataField, TypedDataPayload,
    TypedDataTypes,
};

#[derive(Clone)]
struct MockSigner {
    address: Address,
    provider_hint: Option<String>,
}

impl Signer for MockSigner {
    type Provider = String;
    type Error = String;

    fn connect(&mut self, provider: Self::Provider) {
        self.provider_hint = Some(provider);
    }

    fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(self.address.clone())
    }

    fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        Ok(format!("signed-message:{}", message.len()))
    }

    fn sign_transaction(&self, tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok(format!("signed-transaction:{}", tx.to.is_some()))
    }

    fn sign_typed_data(
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

    fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionReceipt, Self::Error> {
        Ok(TransactionReceipt::new(
            Hash32::new(format!("0x{}", "fa".repeat(32))).unwrap(),
        ))
    }

    fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Ok(Amount::from(21_000u32))
    }
}

struct MockProvider {
    signer: Option<MockSigner>,
    provider_hint: String,
}

impl Provider for MockProvider {
    type Signer = MockSigner;
    type Error = String;

    fn signer_or_null(&self) -> Option<&Self::Signer> {
        self.signer.as_ref()
    }

    fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(1)
    }

    fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error> {
        Ok(Some(
            HexData::new(format!("0x{}", address.as_str().trim_start_matches("0x"))).unwrap(),
        ))
    }

    fn get_transaction_receipt(
        &self,
        transaction_hash: &cow_sdk_core::TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(Some(TransactionReceipt::new(transaction_hash.clone())))
    }

    fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Ok(self.signer.clone().unwrap())
    }

    fn get_storage_at(&self, _address: &Address, slot: &str) -> Result<HexData, Self::Error> {
        Ok(HexData::new(format!("0x{slot:0>4}")).unwrap())
    }

    fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Ok(HexData::new("0x63616c6c").unwrap())
    }

    fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        Ok(format!("read:{}", request.method))
    }

    fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(
            1,
            Some(Hash32::new(format!("0x{}", "ab".repeat(32))).unwrap()),
        ))
    }

    fn set_signer(&mut self, signer: Self::Signer) {
        self.signer = Some(signer);
    }

    fn set_provider(&mut self, provider_hint: String) {
        self.provider_hint = provider_hint;
    }

    fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(address.clone(), abi_json.to_owned()))
    }
}

struct MockGraph;

impl GraphTransport for MockGraph {
    type Error = String;

    fn execute(
        &self,
        endpoint: &str,
        query: &str,
        variables_json: Option<&str>,
    ) -> Result<String, Self::Error> {
        Ok(format!(
            "{endpoint}|{query}|{}",
            variables_json.unwrap_or("{}")
        ))
    }
}

struct MockPinning;

impl PinningTransport for MockPinning {
    type Error = String;

    fn pin_json(&self, payload: &str) -> Result<String, Self::Error> {
        Ok(format!("cid:{payload}"))
    }
}

fn sample_signer() -> MockSigner {
    MockSigner {
        address: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        provider_hint: None,
    }
}

fn sample_provider(signer: MockSigner) -> MockProvider {
    MockProvider {
        signer: Some(signer),
        provider_hint: "initial".to_owned(),
    }
}

fn sample_transaction() -> TransactionRequest {
    TransactionRequest::new(
        Some(Address::new("0x2222222222222222222222222222222222222222").unwrap()),
        Some(HexData::new("0x01020304").unwrap()),
        Some(Amount::zero()),
        Some(Amount::from(21_000u32)),
    )
}

fn sample_typed_data_domain() -> TypedDataDomain {
    TypedDataDomain::new(
        "Gnosis Protocol".to_owned(),
        "v2".to_owned(),
        1,
        Address::new("0x3333333333333333333333333333333333333333").unwrap(),
    )
}

fn sample_typed_data_payload(domain: TypedDataDomain) -> TypedDataPayload {
    let mut types = TypedDataTypes::new();
    types.insert(
        "Order".to_owned(),
        vec![TypedDataField::new(
            "sellToken".to_owned(),
            "address".to_owned(),
        )],
    );
    types.insert(
        "EIP712Domain".to_owned(),
        vec![TypedDataField::new("name".to_owned(), "string".to_owned())],
    );

    TypedDataPayload::new(
        domain,
        "Order".to_owned(),
        types,
        "{\"kind\":\"sell\"}".to_owned(),
    )
}

fn assert_signer_contracts(
    active_signer: &mut MockSigner,
    tx: &TransactionRequest,
    domain: &TypedDataDomain,
) {
    active_signer.connect("rpc://local".to_owned());
    assert_eq!(
        Signer::get_address(active_signer).unwrap().as_str(),
        "0x1111111111111111111111111111111111111111"
    );
    assert_eq!(
        Signer::sign_message(active_signer, b"cow").unwrap(),
        "signed-message:3"
    );
    assert_eq!(
        Signer::sign_transaction(active_signer, tx).unwrap(),
        "signed-transaction:true"
    );
    assert_eq!(
        Signer::sign_typed_data(
            active_signer,
            domain,
            &[TypedDataField::new(
                "sellToken".to_owned(),
                "address".to_owned(),
            )],
            "{\"kind\":\"sell\"}"
        )
        .unwrap(),
        "Gnosis Protocol:1:15"
    );
    assert_eq!(
        Signer::sign_typed_data_payload(active_signer, &sample_typed_data_payload(domain.clone()))
            .unwrap(),
        "Gnosis Protocol:1:15"
    );
    assert_eq!(
        Signer::estimate_gas(active_signer, tx).unwrap(),
        Amount::from(21_000u32)
    );
    assert_eq!(
        Signer::send_transaction(active_signer, tx)
            .unwrap()
            .transaction_hash,
        Hash32::new(format!("0x{}", "fa".repeat(32))).unwrap()
    );
}

fn assert_provider_contracts(
    provider: &mut MockProvider,
    signer: MockSigner,
    tx: &TransactionRequest,
) {
    assert_eq!(Provider::get_chain_id(provider).unwrap(), 1);
    assert_eq!(
        Provider::get_code(
            provider,
            &Address::new("0x4444444444444444444444444444444444444444").unwrap(),
        )
        .unwrap()
        .unwrap(),
        HexData::new("0x4444444444444444444444444444444444444444").unwrap()
    );
    let receipt_hash = Hash32::new(format!("0x{}", "be".repeat(32))).unwrap();
    assert_eq!(
        Provider::get_transaction_receipt(provider, &receipt_hash)
            .unwrap()
            .unwrap()
            .transaction_hash,
        receipt_hash
    );
    assert_eq!(
        Provider::get_storage_at(
            provider,
            &Address::new("0x5555555555555555555555555555555555555555").unwrap(),
            "0",
        )
        .unwrap(),
        HexData::new("0x0000").unwrap()
    );
    assert_eq!(
        Provider::call(provider, tx).unwrap(),
        HexData::new("0x63616c6c").unwrap()
    );
    assert_eq!(
        Provider::read_contract(
            provider,
            &ContractCall::new(
                Address::new("0x6666666666666666666666666666666666666666").unwrap(),
                "balanceOf".to_owned(),
                "[]".to_owned(),
                "[\"0xabc\"]".to_owned(),
            ),
        )
        .unwrap(),
        "read:balanceOf"
    );
    assert_eq!(Provider::get_block(provider, "latest").unwrap().number, 1);
    provider.set_provider("rpc://updated".to_owned());
    provider.set_signer(signer);
    assert_eq!(provider.provider_hint, "rpc://updated");
    assert_eq!(
        Provider::get_contract(
            provider,
            &Address::new("0x7777777777777777777777777777777777777777").unwrap(),
            "[{\"type\":\"function\"}]",
        )
        .unwrap()
        .abi_json,
        "[{\"type\":\"function\"}]"
    );
}

#[test]
fn signer_and_provider_contracts_are_runtime_agnostic_and_callable() {
    let signer = sample_signer();
    let mut provider = sample_provider(signer.clone());
    let tx = sample_transaction();
    let domain = sample_typed_data_domain();
    let mut active_signer = Provider::create_signer(&provider, "local").unwrap();

    assert_signer_contracts(&mut active_signer, &tx, &domain);
    assert_provider_contracts(&mut provider, signer, &tx);
}

#[test]
fn graph_and_pinning_transports_cover_shared_io_boundaries() {
    let graph = MockGraph;
    let pinning = MockPinning;

    assert_eq!(
        graph
            .execute(
                "https://api.thegraph.com",
                "query Totals {}",
                Some("{\"days\":7}")
            )
            .unwrap(),
        "https://api.thegraph.com|query Totals {}|{\"days\":7}"
    );
    assert_eq!(
        pinning.pin_json("{\"appCode\":\"CoW Swap\"}").unwrap(),
        "cid:{\"appCode\":\"CoW Swap\"}"
    );
}

#[tokio::test]
async fn sync_runtime_contracts_gain_async_compatibility_through_blanket_impls() {
    let signer = MockSigner {
        address: Address::new("0x9999999999999999999999999999999999999999").unwrap(),
        provider_hint: None,
    };
    let provider = MockProvider {
        signer: Some(signer.clone()),
        provider_hint: "rpc://test".to_owned(),
    };

    let tx = TransactionRequest::new(
        Some(Address::new("0x8888888888888888888888888888888888888888").unwrap()),
        Some(HexData::new("0x1234").unwrap()),
        Some(Amount::zero()),
        Some(Amount::from(21_000u32)),
    );

    let async_signer = AsyncSigningProvider::create_signer(&provider, "blanket")
        .await
        .unwrap();
    assert_eq!(
        AsyncSigner::get_address(&async_signer)
            .await
            .unwrap()
            .as_str(),
        "0x9999999999999999999999999999999999999999"
    );
    assert_eq!(
        AsyncSigner::estimate_gas(&async_signer, &tx).await.unwrap(),
        Amount::from(21_000u32)
    );
    let mut types = TypedDataTypes::new();
    types.insert(
        "CustomAction".to_owned(),
        vec![TypedDataField::new(
            "actor".to_owned(),
            "address".to_owned(),
        )],
    );
    assert_eq!(
        AsyncSigner::sign_typed_data_payload(
            &async_signer,
            &TypedDataPayload::new(
                TypedDataDomain::new(
                    "Gnosis Protocol".to_owned(),
                    "v2".to_owned(),
                    1,
                    Address::new("0x3333333333333333333333333333333333333333").unwrap(),
                ),
                "CustomAction".to_owned(),
                types,
                "{\"actor\":\"0x9999999999999999999999999999999999999999\"}".to_owned(),
            ),
        )
        .await
        .unwrap(),
        "Gnosis Protocol:1:54"
    );
    assert_eq!(
        AsyncSigner::send_transaction(&async_signer, &tx)
            .await
            .unwrap()
            .transaction_hash,
        Hash32::new(format!("0x{}", "fa".repeat(32))).unwrap()
    );
    assert_eq!(AsyncProvider::get_chain_id(&provider).await.unwrap(), 1);
    assert_eq!(
        AsyncProvider::call(&provider, &tx).await.unwrap(),
        HexData::new("0x63616c6c").unwrap()
    );
}
