mod common;

use std::{cell::RefCell, fmt, rc::Rc, sync::Mutex};

use alloy_primitives::Address as AlloyAddress;
use alloy_sol_types::SolCall;
use cow_sdk_contracts::{
    ContractsError, EIP1271_MAGICVALUE, Eip1271SignatureData, Eip1271VerificationCache,
    Eip1271VerificationRequest, IERC1271, Signature, SigningScheme, decode_eip1271_signature_data,
    decode_signing_scheme, encode_eip1271_signature_data, encode_signing_scheme,
    function_magic_value, normalized_ecdsa_signature, verify_eip1271_signature,
    verify_eip1271_signature_async,
};
use cow_sdk_core::{
    Address, Amount, AsyncProvider, AsyncSigner, AsyncSigningProvider, BlockInfo, ContractCall,
    ContractHandle, Hash32, HexData, TransactionBroadcast, TransactionReceipt, TransactionRequest,
};
use k256::ecdsa::SigningKey;
use sha3::{Digest, Keccak256};

#[derive(Default)]
struct NoCache;

impl Eip1271VerificationCache for NoCache {
    fn get(&self, _verifier: Address, _digest: [u8; 32]) -> Option<bool> {
        None
    }
    fn put(&self, _verifier: Address, _digest: [u8; 32], _result: bool) {}
}

#[derive(Default)]
struct RecordingCache {
    hit: Mutex<Option<bool>>,
    writes: Mutex<Vec<(Address, [u8; 32], bool)>>,
}

impl RecordingCache {
    const fn with_hit(hit: Option<bool>) -> Self {
        Self {
            hit: Mutex::new(hit),
            writes: Mutex::new(Vec::new()),
        }
    }

    fn writes(&self) -> Vec<(Address, [u8; 32], bool)> {
        self.writes.lock().unwrap().clone()
    }
}

impl Eip1271VerificationCache for RecordingCache {
    fn get(&self, _verifier: Address, _digest: [u8; 32]) -> Option<bool> {
        *self.hit.lock().unwrap()
    }

    fn put(&self, verifier: Address, digest: [u8; 32], result: bool) {
        self.writes.lock().unwrap().push((verifier, digest, result));
    }
}

use common::{MockProvider, fixture_case};

fn expected_u8(value: &serde_json::Value) -> u8 {
    u8::try_from(value.as_u64().unwrap()).expect("fixture discriminant must fit in u8")
}

fn synthetic_signature_with_v(v: u8) -> String {
    format!("0x{}{:02x}", "a".repeat(128), v)
}

fn deterministic_signing_key() -> SigningKey {
    SigningKey::from_slice(
        &hex::decode("4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318").unwrap(),
    )
    .unwrap()
}

fn expected_address_for_key(signing_key: &SigningKey) -> Address {
    Address::new(AlloyAddress::from_private_key(signing_key).to_string()).unwrap()
}

fn ecdsa_signature_for_prehash(signing_key: &SigningKey, prehash: &[u8; 32]) -> String {
    let (signature, recovery_id) = signing_key.sign_prehash_recoverable(prehash).unwrap();
    let mut bytes = [0_u8; 65];
    bytes[..64].copy_from_slice(signature.to_bytes().as_slice());
    bytes[64] = recovery_id.to_byte() + 27;
    format!("0x{}", hex::encode(bytes))
}

fn cow_eth_sign_prehash(digest: &Hash32) -> [u8; 32] {
    let digest_bytes = hex::decode(digest.to_hex_string().trim_start_matches("0x")).unwrap();
    let mut payload = Vec::with_capacity(60);
    payload.extend_from_slice(b"\x19Ethereum Signed Message:\n32");
    payload.extend_from_slice(&digest_bytes);
    Keccak256::digest(payload).into()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AsyncMockProviderError(String);

impl fmt::Display for AsyncMockProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Default)]
struct DummyAsyncSigner;

impl AsyncSigner for DummyAsyncSigner {
    type Error = AsyncMockProviderError;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(Address::new("0x1111111111111111111111111111111111111111").unwrap())
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Err(AsyncMockProviderError("not used".to_owned()))
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Err(AsyncMockProviderError("not used".to_owned()))
    }

    async fn sign_typed_data(
        &self,
        _domain: &cow_sdk_core::TypedDataDomain,
        _fields: &[cow_sdk_core::TypedDataField],
        _value_json: &str,
    ) -> Result<String, Self::Error> {
        Err(AsyncMockProviderError("not used".to_owned()))
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Ok(TransactionBroadcast::new(
            Hash32::new(format!("0x{}", "aa".repeat(32))).unwrap(),
        ))
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Ok(Amount::from(21_000u32))
    }
}

#[derive(Clone)]
struct AsyncMockProvider {
    calls: Rc<RefCell<Vec<ContractCall>>>,
    response: Rc<RefCell<String>>,
    response_error: Rc<RefCell<Option<String>>>,
    code: Rc<RefCell<Option<HexData>>>,
    code_error: Rc<RefCell<Option<String>>>,
}

impl AsyncMockProvider {
    fn new() -> Self {
        Self {
            calls: Rc::new(RefCell::new(Vec::new())),
            response: Rc::new(RefCell::new("null".to_owned())),
            response_error: Rc::new(RefCell::new(None)),
            code: Rc::new(RefCell::new(None)),
            code_error: Rc::new(RefCell::new(None)),
        }
    }

    fn set_code(&self, value: Option<&str>) {
        *self.code.borrow_mut() = value.map(|value| HexData::new(value).unwrap());
    }

    fn set_code_error(&self, value: Option<&str>) {
        *self.code_error.borrow_mut() = value.map(str::to_owned);
    }

    fn set_response(&self, value: &str) {
        let mut response = self.response.borrow_mut();
        value.clone_into(&mut response);
    }

    fn set_response_error(&self, value: Option<&str>) {
        *self.response_error.borrow_mut() = value.map(str::to_owned);
    }
}

impl AsyncProvider for AsyncMockProvider {
    type Error = AsyncMockProviderError;

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(1)
    }

    async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        if let Some(message) = self.code_error.borrow().clone() {
            return Err(AsyncMockProviderError(message));
        }
        Ok(self.code.borrow().clone())
    }

    async fn get_transaction_receipt(
        &self,
        _transaction_hash: &cow_sdk_core::TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(None)
    }

    async fn get_storage_at(
        &self,
        _address: &Address,
        _slot: &str,
    ) -> Result<HexData, Self::Error> {
        Ok(HexData::new("0x").unwrap())
    }

    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Ok(HexData::new("0x").unwrap())
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        self.calls.borrow_mut().push(request.clone());
        if let Some(message) = self.response_error.borrow().clone() {
            return Err(AsyncMockProviderError(message));
        }
        Ok(self.response.borrow().clone())
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(0, None))
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(*address, abi_json.to_owned()))
    }
}

impl AsyncSigningProvider for AsyncMockProvider {
    type Signer = DummyAsyncSigner;

    async fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Ok(DummyAsyncSigner)
    }
}

#[test]
fn signing_scheme_and_magic_value_match_fixture_contract() {
    let schemes = fixture_case("contracts-signing-scheme-discriminants");
    let expected = &schemes["expected"];
    assert_eq!(
        encode_signing_scheme(SigningScheme::Eip712),
        expected_u8(&expected["EIP712"])
    );
    assert_eq!(
        encode_signing_scheme(SigningScheme::EthSign),
        expected_u8(&expected["ETHSIGN"])
    );
    assert_eq!(
        encode_signing_scheme(SigningScheme::Eip1271),
        expected_u8(&expected["EIP1271"])
    );
    assert_eq!(
        encode_signing_scheme(SigningScheme::PreSign),
        expected_u8(&expected["PRESIGN"])
    );

    assert_eq!(decode_signing_scheme(0).unwrap(), SigningScheme::Eip712);
    assert_eq!(decode_signing_scheme(1).unwrap(), SigningScheme::EthSign);
    assert_eq!(decode_signing_scheme(2).unwrap(), SigningScheme::Eip1271);
    assert_eq!(decode_signing_scheme(3).unwrap(), SigningScheme::PreSign);
    assert!(decode_signing_scheme(4).is_err());

    let magic = fixture_case("contracts-eip1271-magic-value");
    assert_eq!(
        EIP1271_MAGICVALUE,
        magic["expected"]["magic_value"].as_str().unwrap()
    );
    assert_eq!(
        function_magic_value("isValidSignature(bytes32,bytes)"),
        EIP1271_MAGICVALUE
    );
    assert_eq!(
        IERC1271::isValidSignatureCall::SELECTOR,
        [0x16, 0x26, 0xba, 0x7e]
    );
    assert_eq!(
        format!(
            "0x{}",
            hex::encode(IERC1271::isValidSignatureCall::SELECTOR)
        ),
        EIP1271_MAGICVALUE
    );
}

#[test]
fn eip1271_signature_payloads_roundtrip_with_variable_lengths() {
    let verifier = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();

    for signature in [
        "0x",
        "0x1234",
        "0x29a674dfc87f8c78fc2bfbcbe8ffdd435091a6a84bc7761db72a45da453d73ac41c5ce28eceb34be73fddc12a5d04af6e736405e41b613aeefeed3db8122420c1b",
    ] {
        let data = Eip1271SignatureData::new(verifier, signature.to_owned());

        let encoded = encode_eip1271_signature_data(&data).unwrap();
        let decoded = decode_eip1271_signature_data(&encoded).unwrap();
        assert_eq!(
            decoded.verifier.to_hex_string(),
            data.verifier.to_hex_string()
        );
        assert_eq!(decoded.signature, data.signature);
    }

    assert!(decode_eip1271_signature_data("0x1234").is_err());
}

#[test]
fn signature_helpers_preserve_public_contract_surface() {
    let signer = Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let ecdsa = Signature::Ecdsa {
        scheme: SigningScheme::Eip712,
        data: normalized_ecdsa_signature(
            "0x29A674DFC87F8C78FC2BFBCBE8FFDD435091A6A84BC7761DB72A45DA453D73AC41C5CE28ECEB34BE73FDDC12A5D04AF6E736405E41B613AEEFEED3DB8122420C1B",
        )
        .unwrap(),
    };
    let pre_sign = Signature::PreSign { owner: signer };
    let eip1271 = Signature::Eip1271 {
        data: Eip1271SignatureData::new(signer, "0x1234".to_owned()),
    };

    assert_eq!(ecdsa.scheme(), SigningScheme::Eip712);
    assert_eq!(pre_sign.scheme(), SigningScheme::PreSign);
    assert_eq!(eip1271.scheme(), SigningScheme::Eip1271);
    assert_eq!(ecdsa.declared_address(), None);
    assert_eq!(pre_sign.declared_address(), Some(&signer));
    assert_eq!(eip1271.declared_address(), Some(&signer));
    assert!(SigningScheme::Eip712.is_ecdsa());
    assert!(SigningScheme::EthSign.is_ecdsa());
    assert!(!SigningScheme::Eip1271.is_ecdsa());
}

#[test]
fn recover_ecdsa_address_recovers_eip712_prehash_signer() {
    let digest =
        Hash32::new("0x5eb4f5a33c621f32a8622d5f943b6b102994dfe4e5aebbefe69bb1b2aa0fc93e").unwrap();
    let signature = Signature::Ecdsa {
        scheme: SigningScheme::Eip712,
        data: "0x48b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c8041b".to_owned(),
    };

    let recovered = signature.recover_ecdsa_address(&digest).unwrap();

    assert_eq!(
        recovered,
        Address::new("0x0f65fe9276bc9a24ae7083ae28e2660ef72df99e").unwrap()
    );
}

#[test]
fn recover_ecdsa_address_eth_sign_recovers_signer_through_canonical_eip191_prehash() {
    let signing_key = deterministic_signing_key();
    let digest =
        Hash32::new("0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20").unwrap();
    let prehash = cow_eth_sign_prehash(&digest);
    let expected = expected_address_for_key(&signing_key);
    let signature = Signature::Ecdsa {
        scheme: SigningScheme::EthSign,
        data: ecdsa_signature_for_prehash(&signing_key, &prehash),
    };

    let recovered = signature.recover_ecdsa_address(&digest).unwrap();

    assert_eq!(recovered, expected);
}

#[test]
fn recover_ecdsa_address_rejects_non_ecdsa_variants() {
    let digest = Hash32::new(format!("0x{}", "11".repeat(32))).unwrap();
    let verifier = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();
    let eip1271 = Signature::Eip1271 {
        data: Eip1271SignatureData::new(verifier, "0x1234".to_owned()),
    };
    let pre_sign = Signature::PreSign { owner: verifier };

    for signature in [eip1271, pre_sign] {
        let error = signature.recover_ecdsa_address(&digest).unwrap_err();
        assert!(matches!(error, ContractsError::SignatureSchemeNotEcdsa));
    }
}

#[test]
fn normalized_ecdsa_signature_normalizes_hex_case_and_prefix() {
    let normalized = normalized_ecdsa_signature(
        "0x29A674DFC87F8C78FC2BFBCBE8FFDD435091A6A84BC7761DB72A45DA453D73AC41C5CE28ECEB34BE73FDDC12A5D04AF6E736405E41B613AEEFEED3DB8122420C1B",
    )
    .unwrap();
    assert_eq!(
        normalized,
        "0x29a674dfc87f8c78fc2bfbcbe8ffdd435091a6a84bc7761db72a45da453d73ac41c5ce28eceb34be73fddc12a5d04af6e736405e41b613aeefeed3db8122420c1b"
    );
}

#[test]
fn normalized_ecdsa_signature_canonicalizes_v_from_eip2_to_legacy() {
    let normalized_v0 = normalized_ecdsa_signature(&synthetic_signature_with_v(0)).unwrap();
    assert_eq!(normalized_v0, synthetic_signature_with_v(27));

    let normalized_v1 = normalized_ecdsa_signature(&synthetic_signature_with_v(1)).unwrap();
    assert_eq!(normalized_v1, synthetic_signature_with_v(28));
}

#[test]
fn normalized_ecdsa_signature_preserves_legacy_v_values() {
    let normalized_v27 = normalized_ecdsa_signature(&synthetic_signature_with_v(27)).unwrap();
    assert_eq!(normalized_v27, synthetic_signature_with_v(27));

    let normalized_v28 = normalized_ecdsa_signature(&synthetic_signature_with_v(28)).unwrap();
    assert_eq!(normalized_v28, synthetic_signature_with_v(28));
}

#[test]
fn normalized_ecdsa_signature_rejects_invalid_recovery_byte() {
    let invalid_two = normalized_ecdsa_signature(&synthetic_signature_with_v(2)).unwrap_err();
    assert!(matches!(
        invalid_two,
        ContractsError::InvalidSignatureRecoveryByte { value: 2 }
    ));

    let invalid_ff = normalized_ecdsa_signature(&synthetic_signature_with_v(0xff)).unwrap_err();
    assert!(matches!(
        invalid_ff,
        ContractsError::InvalidSignatureRecoveryByte { value: 0xff }
    ));
}

#[test]
fn normalized_ecdsa_signature_rejects_wrong_length() {
    let four_byte = normalized_ecdsa_signature("0xabababab").unwrap_err();
    assert!(matches!(
        four_byte,
        ContractsError::InvalidSignatureLength { actual: 4 }
    ));

    let missing_v = normalized_ecdsa_signature(&format!("0x{}", "a".repeat(128))).unwrap_err();
    assert!(matches!(
        missing_v,
        ContractsError::InvalidSignatureLength { actual: 64 }
    ));
}

#[test]
fn normalized_ecdsa_signature_rejects_invalid_hex() {
    let missing_prefix = normalized_ecdsa_signature("xyzzy").unwrap_err();
    assert!(matches!(
        missing_prefix,
        ContractsError::InvalidHexPrefix { field } if field == "signature"
    ));

    let invalid_hex = normalized_ecdsa_signature(&format!("0x{}", "z".repeat(130))).unwrap_err();
    assert!(matches!(
        invalid_hex,
        ContractsError::DecodeHex { field, source: _ } if field == "signature"
    ));
}

#[test]
fn eip1271_verification_reads_contract_code_and_magic_value() {
    let provider = MockProvider::new();
    let verifier = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();
    provider.set_code(Some("0x6001600055"));
    provider.set_response("\"0x1626ba7e\"");

    verify_eip1271_signature(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::new(format!("0x{}", "11".repeat(32))).unwrap(),
            HexData::new("0x1234").unwrap(),
        ),
    )
    .unwrap();

    let call = provider.calls.borrow().last().cloned().unwrap();
    assert_eq!(call.address, verifier);
    assert_eq!(call.method, "isValidSignature");
    assert!(call.abi_json.contains("\"bytes4\""));
    let args: Vec<String> = serde_json::from_str(&call.args_json).unwrap();
    assert_eq!(args[0], format!("0x{}", "11".repeat(32)));
    assert_eq!(args[1], "0x1234");
}

#[test]
fn eip1271_verification_fails_closed_for_missing_code_and_transport_errors() {
    let provider = MockProvider::new();
    let verifier = Address::new("0x1111111111111111111111111111111111111111").unwrap();

    let missing = verify_eip1271_signature(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::new(format!("0x{}", "22".repeat(32))).unwrap(),
            HexData::new("0x").unwrap(),
        ),
    )
    .unwrap_err();
    match &missing {
        ContractsError::UnsupportedEip1271Verifier { verifier: got } => {
            assert_eq!(got.to_hex_string(), verifier.to_hex_string());
        }
        other => panic!("expected UnsupportedEip1271Verifier, got {other:?}"),
    }

    provider.set_code(Some("0x6001600055"));
    provider.set_response_error(Some("rpc unavailable"));
    let transport = verify_eip1271_signature(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::new(format!("0x{}", "33".repeat(32))).unwrap(),
            HexData::new("0x1234").unwrap(),
        ),
    )
    .unwrap_err();
    match transport {
        ContractsError::Eip1271Provider { operation, message } => {
            assert_eq!(operation, "read_contract");
            assert_eq!(message.as_inner(), "rpc unavailable");
        }
        other => panic!("expected Eip1271Provider, got {other:?}"),
    }
}

#[test]
fn eip1271_verification_rejects_malformed_and_wrong_magic_responses() {
    let provider = MockProvider::new();
    let verifier = Address::new("0x2222222222222222222222222222222222222222").unwrap();
    provider.set_code(Some("0x6001600055"));

    provider.set_response("{\"unexpected\":true}");
    let malformed = verify_eip1271_signature(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::new(format!("0x{}", "44".repeat(32))).unwrap(),
            HexData::new("0x1234").unwrap(),
        ),
    )
    .unwrap_err();
    match &malformed {
        ContractsError::MalformedEip1271Response { response } => {
            assert_eq!(response.as_inner(), "{\"unexpected\":true}");
        }
        other => panic!("expected MalformedEip1271Response, got {other:?}"),
    }

    provider.set_response("\"0xffffffff\"");
    let mismatch = verify_eip1271_signature(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::new(format!("0x{}", "55".repeat(32))).unwrap(),
            HexData::new("0x1234").unwrap(),
        ),
    )
    .unwrap_err();
    match &mismatch {
        ContractsError::Eip1271MagicValueMismatch { expected, actual } => {
            assert_eq!(*expected, [0x16, 0x26, 0xba, 0x7e]);
            assert_eq!(*actual, [0xff, 0xff, 0xff, 0xff]);
        }
        other => panic!("expected Eip1271MagicValueMismatch, got {other:?}"),
    }
    assert_eq!(EIP1271_MAGICVALUE, "0x1626ba7e");
    assert_eq!(
        mismatch.to_string(),
        "unexpected EIP-1271 magic value: expected 0x1626ba7e, got 0xffffffff",
    );
}

#[tokio::test(flavor = "current_thread")]
async fn async_eip1271_verification_reads_contract_code_and_magic_value() {
    let provider = AsyncMockProvider::new();
    let verifier = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();
    provider.set_code(Some("0x6001600055"));
    provider.set_response("\"0x1626ba7e\"");

    verify_eip1271_signature_async(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::new(format!("0x{}", "11".repeat(32))).unwrap(),
            HexData::new("0x1234").unwrap(),
        ),
        &NoCache,
    )
    .await
    .unwrap();

    let call = provider.calls.borrow().last().cloned().unwrap();
    assert_eq!(call.address, verifier);
    assert_eq!(call.method, "isValidSignature");
}

#[tokio::test(flavor = "current_thread")]
async fn async_eip1271_cache_hit_false_fails_closed_without_provider_call() {
    let provider = AsyncMockProvider::new();
    let verifier = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();
    let cache = RecordingCache::with_hit(Some(false));

    let error = verify_eip1271_signature_async(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::from_bytes([0x66; 32]),
            HexData::new("0x1234").unwrap(),
        ),
        &cache,
    )
    .await
    .expect_err("cached invalid EIP-1271 result must fail closed");

    assert!(matches!(
        error,
        ContractsError::Eip1271MagicValueMismatch {
            expected: [0x16, 0x26, 0xba, 0x7e],
            actual: [0, 0, 0, 0],
        }
    ));
    assert!(
        provider.calls.borrow().is_empty(),
        "cache hits must not call the verifier contract",
    );
    assert!(
        cache.writes().is_empty(),
        "cache hits must not rewrite the cache entry",
    );
}

#[tokio::test(flavor = "current_thread")]
async fn async_eip1271_verification_caches_only_magic_value_outcomes_by_digest() {
    let provider = AsyncMockProvider::new();
    let verifier = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();
    provider.set_code(Some("0x6001600055"));
    let cache = RecordingCache::default();

    provider.set_response("\"0x1626ba7e\"");
    verify_eip1271_signature_async(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::from_bytes([0x77; 32]),
            HexData::new("0x1234").unwrap(),
        ),
        &cache,
    )
    .await
    .expect("valid magic value must verify");
    assert_eq!(cache.writes(), vec![(verifier, [0x77; 32], true)]);

    provider.set_response("\"0xffffffff\"");
    let mismatch = verify_eip1271_signature_async(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::from_bytes([0x78; 32]),
            HexData::new("0x1234").unwrap(),
        ),
        &cache,
    )
    .await
    .expect_err("wrong magic value must fail closed");
    assert!(matches!(
        mismatch,
        ContractsError::Eip1271MagicValueMismatch {
            expected: [0x16, 0x26, 0xba, 0x7e],
            actual: [0xff, 0xff, 0xff, 0xff],
        }
    ));
    assert_eq!(
        cache.writes(),
        vec![(verifier, [0x77; 32], true), (verifier, [0x78; 32], false)]
    );

    provider.set_response("{\"unexpected\":true}");
    let before_malformed = cache.writes();
    let malformed = verify_eip1271_signature_async(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::from_bytes([0x79; 32]),
            HexData::new("0x1234").unwrap(),
        ),
        &cache,
    )
    .await
    .expect_err("malformed responses must not be cached as verifier outcomes");
    assert!(matches!(
        malformed,
        ContractsError::MalformedEip1271Response { .. }
    ));
    assert_eq!(
        cache.writes(),
        before_malformed,
        "non-cacheable verifier errors must not be stored",
    );
}

#[tokio::test(flavor = "current_thread")]
async fn async_eip1271_verification_fails_closed_for_missing_code_and_transport_errors() {
    let provider = AsyncMockProvider::new();
    let verifier = Address::new("0x1111111111111111111111111111111111111111").unwrap();

    let missing = verify_eip1271_signature_async(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::new(format!("0x{}", "22".repeat(32))).unwrap(),
            HexData::new("0x1234").unwrap(),
        ),
        &NoCache,
    )
    .await
    .unwrap_err();
    match &missing {
        ContractsError::UnsupportedEip1271Verifier { verifier: got } => {
            assert_eq!(got.to_hex_string(), verifier.to_hex_string());
        }
        other => panic!("expected UnsupportedEip1271Verifier, got {other:?}"),
    }

    provider.set_code(Some("0x6001600055"));
    provider.set_response_error(Some("rpc unavailable"));
    let transport = verify_eip1271_signature_async(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::new(format!("0x{}", "33".repeat(32))).unwrap(),
            HexData::new("0x1234").unwrap(),
        ),
        &NoCache,
    )
    .await
    .unwrap_err();
    match &transport {
        ContractsError::Eip1271Provider { operation, message } => {
            assert_eq!(*operation, "read_contract");
            assert_eq!(message.as_inner(), "rpc unavailable");
        }
        other => panic!("expected Eip1271Provider, got {other:?}"),
    }

    provider.set_response_error(None);
    provider.set_code_error(Some("code lookup unavailable"));
    let code_error = verify_eip1271_signature_async(
        &provider,
        &Eip1271VerificationRequest::new(
            verifier,
            Hash32::new(format!("0x{}", "44".repeat(32))).unwrap(),
            HexData::new("0x1234").unwrap(),
        ),
        &NoCache,
    )
    .await
    .unwrap_err();
    match &code_error {
        ContractsError::Eip1271Provider { operation, message } => {
            assert_eq!(*operation, "get_code");
            assert_eq!(message.as_inner(), "code lookup unavailable");
        }
        other => panic!("expected Eip1271Provider, got {other:?}"),
    }
}
