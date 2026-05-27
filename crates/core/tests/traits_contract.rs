use cow_sdk_core::{
    Address, Amount, BlockInfo, ContractCall, ContractHandle, DigestSigner, GraphTransport, Hash32,
    HexData, Owner, PinningTransport, Provider, Signer, SigningProvider, TransactionBroadcast,
    TransactionReceipt, TransactionRequest, TransactionStatus, TypedDataDomain, TypedDataField,
    TypedDataPayload, TypedDataSigner, TypedDataTypes,
};

const HASH_1: &str = "0x1111111111111111111111111111111111111111111111111111111111111111";
const BLOCK_HASH_1: &str = "0x2222222222222222222222222222222222222222222222222222222222222222";
const FROM_ADDR: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TO_ADDR: &str = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[derive(Clone)]
struct MockSigner {
    address: Address,
}

impl Signer for MockSigner {
    type Error = String;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(self.address)
    }

    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        Ok(format!("signed-message:{}", message.len()))
    }

    async fn sign_transaction(&self, tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok(format!("signed-transaction:{}", tx.to.is_some()))
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
    ) -> Result<TransactionBroadcast, Self::Error> {
        Ok(TransactionBroadcast::new(
            Hash32::new(format!("0x{}", "fa".repeat(32))).unwrap(),
        ))
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Ok(Amount::from(21_000u32))
    }
}

struct MockProvider {
    signer: Option<MockSigner>,
}

impl Provider for MockProvider {
    type Error = String;

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(1)
    }

    async fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error> {
        Ok(Some(HexData::new(address.to_hex_string()).unwrap()))
    }

    async fn get_transaction_receipt(
        &self,
        transaction_hash: &cow_sdk_core::TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(Some(
            TransactionReceipt::new(*transaction_hash)
                .with_status(TransactionStatus::Success)
                .with_block_number(42)
                .with_block_hash(Hash32::new(format!("0x{}", "ba".repeat(32))).unwrap())
                .with_gas_used(Amount::from(21_000u32))
                .with_from(Address::new(FROM_ADDR).unwrap())
                .with_to(Address::new(TO_ADDR).unwrap()),
        ))
    }

    async fn get_storage_at(&self, _address: &Address, slot: &str) -> Result<HexData, Self::Error> {
        Ok(HexData::new(format!("0x{slot:0>4}")).unwrap())
    }

    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Ok(HexData::new("0x63616c6c").unwrap())
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        Ok(format!("read:{}", request.method))
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(
            1,
            Some(Hash32::new(format!("0x{}", "ab".repeat(32))).unwrap()),
        ))
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(*address, abi_json.to_owned()))
    }
}

impl SigningProvider for MockProvider {
    type Signer = MockSigner;

    async fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Ok(self.signer.clone().unwrap())
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
    }
}

const fn sample_provider(signer: MockSigner) -> MockProvider {
    MockProvider {
        signer: Some(signer),
    }
}

fn sample_transaction() -> TransactionRequest {
    TransactionRequest::new(
        Some(Address::new("0x2222222222222222222222222222222222222222").unwrap()),
        Some(HexData::new("0x01020304").unwrap()),
        Some(Amount::ZERO),
        Some(Amount::from(21_000u32)),
    )
}

fn transaction_hash(value: &str) -> cow_sdk_core::TransactionHash {
    Hash32::new(value).unwrap()
}

fn sample_from_address() -> Address {
    Address::new(FROM_ADDR).unwrap()
}

fn sample_to_address() -> Address {
    Address::new(TO_ADDR).unwrap()
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

fn sample_custom_action_payload() -> TypedDataPayload {
    let mut types = TypedDataTypes::new();
    types.insert(
        "CustomAction".to_owned(),
        vec![TypedDataField::new(
            "actor".to_owned(),
            "address".to_owned(),
        )],
    );
    TypedDataPayload::new(
        TypedDataDomain::new(
            "Gnosis Protocol".to_owned(),
            "v2".to_owned(),
            1,
            Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        ),
        "CustomAction".to_owned(),
        types,
        "{\"actor\":\"0x9999999999999999999999999999999999999999\"}".to_owned(),
    )
}

async fn assert_signer_contracts(
    active_signer: &MockSigner,
    tx: &TransactionRequest,
    domain: &TypedDataDomain,
) {
    assert_eq!(
        Signer::get_address(active_signer)
            .await
            .unwrap()
            .to_hex_string(),
        "0x1111111111111111111111111111111111111111"
    );
    assert_eq!(
        Signer::sign_message(active_signer, b"cow").await.unwrap(),
        "signed-message:3"
    );
    assert_eq!(
        Signer::sign_transaction(active_signer, tx).await.unwrap(),
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
        .await
        .unwrap(),
        "Gnosis Protocol:1:15"
    );
    assert_eq!(
        Signer::sign_typed_data_payload(active_signer, &sample_typed_data_payload(domain.clone()))
            .await
            .unwrap(),
        "Gnosis Protocol:1:15"
    );
    assert_eq!(
        Signer::estimate_gas(active_signer, tx).await.unwrap(),
        Amount::from(21_000u32)
    );
    assert_eq!(
        Signer::send_transaction(active_signer, tx)
            .await
            .unwrap()
            .transaction_hash,
        Hash32::new(format!("0x{}", "fa".repeat(32))).unwrap()
    );
}

async fn assert_provider_contracts(provider: &MockProvider, tx: &TransactionRequest) {
    assert_eq!(Provider::get_chain_id(provider).await.unwrap(), 1);
    assert_eq!(
        Provider::get_code(
            provider,
            &Address::new("0x4444444444444444444444444444444444444444").unwrap(),
        )
        .await
        .unwrap()
        .unwrap(),
        HexData::new("0x4444444444444444444444444444444444444444").unwrap()
    );
    let receipt_hash = Hash32::new(format!("0x{}", "be".repeat(32))).unwrap();
    assert_eq!(
        Provider::get_transaction_receipt(provider, &receipt_hash)
            .await
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
        .await
        .unwrap(),
        HexData::new("0x0000").unwrap()
    );
    assert_eq!(
        Provider::call(provider, tx).await.unwrap(),
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
        .await
        .unwrap(),
        "read:balanceOf"
    );
    assert_eq!(
        Provider::get_block(provider, "latest")
            .await
            .unwrap()
            .number,
        1
    );
    assert_eq!(
        Provider::get_contract(
            provider,
            &Address::new("0x7777777777777777777777777777777777777777").unwrap(),
            "[{\"type\":\"function\"}]",
        )
        .await
        .unwrap()
        .abi_json,
        "[{\"type\":\"function\"}]"
    );
}

#[tokio::test]
async fn signer_returns_transaction_broadcast() {
    let signer = MockSigner {
        address: Address::new("0x9999999999999999999999999999999999999999").unwrap(),
    };
    let tx = sample_transaction();

    let broadcast = Signer::send_transaction(&signer, &tx).await.unwrap();

    assert_eq!(
        broadcast,
        TransactionBroadcast::new(Hash32::new(format!("0x{}", "fa".repeat(32))).unwrap())
    );
}

#[tokio::test]
async fn provider_returns_rich_transaction_receipt() {
    let provider = sample_provider(sample_signer());
    let tx_hash = transaction_hash(HASH_1);

    let receipt = Provider::get_transaction_receipt(&provider, &tx_hash)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(receipt.transaction_hash, tx_hash);
    assert_eq!(receipt.status, Some(TransactionStatus::Success));
    assert_eq!(receipt.block_number, Some(42));
    assert_eq!(
        receipt.block_hash,
        Some(Hash32::new(format!("0x{}", "ba".repeat(32))).unwrap())
    );
    assert_eq!(receipt.gas_used, Some(Amount::from(21_000u32)));
    assert_eq!(receipt.from, Some(sample_from_address()));
    assert_eq!(receipt.to, Some(sample_to_address()));
}

#[test]
fn transaction_status_serde_roundtrip_camel_case() {
    let serialized = serde_json::to_string(&TransactionStatus::Success).unwrap();

    assert_eq!(serialized, "\"success\"");
    assert_eq!(
        serde_json::from_str::<TransactionStatus>(&serialized).unwrap(),
        TransactionStatus::Success
    );
    assert_eq!(
        serde_json::from_str::<TransactionStatus>("\"reverted\"").unwrap(),
        TransactionStatus::Reverted
    );
}

#[test]
fn transaction_receipt_skips_serializing_none_fields() {
    let receipt = TransactionReceipt::new(transaction_hash(HASH_1));

    assert_eq!(
        serde_json::to_value(receipt).unwrap(),
        serde_json::json!({
            "transactionHash": HASH_1,
        })
    );
}

#[test]
fn transaction_receipt_serializes_populated_fields() {
    let receipt = TransactionReceipt::from_parts(
        transaction_hash(HASH_1),
        Some(TransactionStatus::Success),
        Some(12_345),
        Some(transaction_hash(BLOCK_HASH_1)),
        Some(Amount::from(21_000u64)),
        Some(sample_from_address()),
        Some(sample_to_address()),
    );

    assert_eq!(
        serde_json::to_value(receipt).unwrap(),
        serde_json::json!({
            "transactionHash": HASH_1,
            "status": "success",
            "blockNumber": 12345,
            "blockHash": BLOCK_HASH_1,
            "gasUsed": "21000",
            "from": FROM_ADDR,
            "to": TO_ADDR,
        })
    );
}

#[test]
fn transaction_receipt_with_builders_round_trips() {
    let receipt = TransactionReceipt::new(transaction_hash(HASH_1))
        .with_status(TransactionStatus::Reverted)
        .with_block_number(98_765)
        .with_block_hash(transaction_hash(BLOCK_HASH_1))
        .with_gas_used(Amount::from(30_000u64))
        .with_from(sample_from_address())
        .with_to(sample_to_address());

    let serialized = serde_json::to_string(&receipt).unwrap();
    let deserialized = serde_json::from_str::<TransactionReceipt>(&serialized).unwrap();

    assert_eq!(deserialized, receipt);
}

#[test]
fn transaction_broadcast_minimal_serde_shape() {
    let broadcast = TransactionBroadcast::new(transaction_hash(HASH_1));

    assert_eq!(
        serde_json::to_value(broadcast).unwrap(),
        serde_json::json!({
            "transactionHash": HASH_1,
        })
    );
}

#[tokio::test]
async fn signer_and_provider_contracts_are_runtime_agnostic_and_callable() {
    let signer = sample_signer();
    let provider = sample_provider(signer.clone());
    let tx = sample_transaction();
    let domain = sample_typed_data_domain();
    let active_signer = SigningProvider::create_signer(&provider, "local")
        .await
        .unwrap();

    assert_signer_contracts(&active_signer, &tx, &domain).await;
    assert_provider_contracts(&provider, &tx).await;
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
async fn signer_satisfies_owner_typed_data_and_digest_capabilities() {
    let signer = MockSigner {
        address: Address::new("0x9999999999999999999999999999999999999999").unwrap(),
    };
    let provider = MockProvider {
        signer: Some(signer.clone()),
    };

    let tx = TransactionRequest::new(
        Some(Address::new("0x8888888888888888888888888888888888888888").unwrap()),
        Some(HexData::new("0x1234").unwrap()),
        Some(Amount::ZERO),
        Some(Amount::from(21_000u32)),
    );

    let active_signer = SigningProvider::create_signer(&provider, "active")
        .await
        .unwrap();
    assert_eq!(
        Signer::get_address(&active_signer)
            .await
            .unwrap()
            .to_hex_string(),
        "0x9999999999999999999999999999999999999999"
    );
    assert_eq!(
        Owner::get_address(&active_signer)
            .await
            .unwrap()
            .to_hex_string(),
        "0x9999999999999999999999999999999999999999"
    );
    assert_eq!(
        Signer::estimate_gas(&active_signer, &tx).await.unwrap(),
        Amount::from(21_000u32)
    );
    let payload = sample_custom_action_payload();
    assert_eq!(
        Signer::sign_typed_data_payload(&active_signer, &payload)
            .await
            .unwrap(),
        "Gnosis Protocol:1:54"
    );
    assert_eq!(
        TypedDataSigner::sign_typed_data_payload(&active_signer, &payload)
            .await
            .unwrap(),
        "Gnosis Protocol:1:54"
    );
    assert_eq!(
        DigestSigner::sign_digest(&active_signer, b"cow")
            .await
            .unwrap(),
        "signed-message:3"
    );
    assert_eq!(
        Signer::send_transaction(&active_signer, &tx)
            .await
            .unwrap()
            .transaction_hash,
        Hash32::new(format!("0x{}", "fa".repeat(32))).unwrap()
    );
    assert_eq!(Provider::get_chain_id(&provider).await.unwrap(), 1);
    assert_eq!(
        Provider::call(&provider, &tx).await.unwrap(),
        HexData::new("0x63616c6c").unwrap()
    );
}
